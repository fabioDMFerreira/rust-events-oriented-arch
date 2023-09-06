use std::sync::Arc;
use std::time::{Duration, Instant};

use actix::prelude::*;
use actix::{Actor, AsyncContext, StreamHandler};
use actix_web_actors::ws;
use jsonwebtoken::{decode, DecodingKey, Validation};
use log::warn;

use crate::config::Config;
use crate::models::token_claims::TokenClaims;

use super::ws_server::{Connect, Disconnect, Message, Swap, WebsocketServer};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WebsocketSession {
    pub id: String,

    pub hb: Instant,

    pub addr: Addr<WebsocketServer>,

    pub authenticated: bool,

    pub config: Arc<Config>,
}

impl WebsocketSession {
    /// helper method that sends ping to client every 5 seconds (HEARTBEAT_INTERVAL).
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // notify chat server
                act.addr.do_send(Disconnect { id: act.id.clone() });

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for WebsocketSession {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    /// We register ws session with WebsocketServer
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        // register self in ws server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsChatSessionState, state is shared
        // across all routes within application
        let addr = ctx.address();
        self.addr
            .send(Connect {
                id: self.id.clone(),
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.addr.do_send(Disconnect {
            id: self.id.clone(),
        });
        Running::Stop
    }
}

impl Handler<Message> for WebsocketSession {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebsocketSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Text(text)) => {
                let msg = text.to_string();

                if msg.starts_with("/login") {
                    // if there is a token after the command /login check whether the token is valid
                    // if token is valid set the session as authenticated and set the session id with the user id
                    if let Some(token) = msg.splitn(2, ' ').nth(1).filter(|s| !s.is_empty()) {
                        match decode::<TokenClaims>(
                            &token,
                            &DecodingKey::from_secret(self.config.jwt_secret.as_ref()),
                            &Validation::default(),
                        ) {
                            Ok(c) => {
                                let new_id = c.claims.sub.to_string();
                                self.addr.do_send(Swap {
                                    id: self.id.clone(),
                                    new_id: new_id.clone(),
                                });
                                self.id = new_id;
                                self.authenticated = true
                            }
                            Err(_) => {
                                warn!("invalid authentication token {}", token);
                            }
                        };
                    }
                } else if self.authenticated {
                    self.hb = Instant::now();

                    if msg != "ping" {
                        ctx.text(format!("received text: {}", msg));
                    }
                }
            }
            _ => (),
        }
    }
}

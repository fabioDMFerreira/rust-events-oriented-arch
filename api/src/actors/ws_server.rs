use std::collections::HashMap;

use actix::prelude::*;
use actix::Actor;
use log::debug;
use log::warn;
use rand::{self, rngs::ThreadRng, Rng};

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Message)]
#[rtype(result = "String")]
pub struct Connect {
    pub id: String,
    pub addr: Recipient<Message>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Swap {
    pub id: String,
    pub new_id: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SessionMessage {
    pub id: String,
    pub message: String,
}

#[derive(Debug)]
pub struct WebsocketServer {
    sessions: HashMap<String, Recipient<Message>>,
    rng: ThreadRng,
}

impl WebsocketServer {
    pub fn new() -> WebsocketServer {
        WebsocketServer {
            sessions: HashMap::new(),
            rng: rand::thread_rng(),
        }
    }
}

impl Actor for WebsocketServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

impl Handler<Connect> for WebsocketServer {
    type Result = String;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> String {
        let id = self.rng.gen::<usize>().to_string();

        debug!("Someone joined: {}", id);

        self.sessions.insert(id.clone(), msg.addr);

        id
    }
}

impl Handler<SessionMessage> for WebsocketServer {
    type Result = ();

    fn handle(&mut self, msg: SessionMessage, _: &mut Context<Self>) {
        let session_result = self.sessions.get(&msg.id);

        if session_result.is_some() {
            session_result.unwrap().do_send(Message(msg.message));
        } else {
            warn!("server tried to send message to unknown session {}", msg.id);
        }
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for WebsocketServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        debug!("Someone disconnected: {}", msg.id);

        self.sessions.remove(&msg.id);
    }
}

/// Handler for Disconnect message.
impl Handler<Swap> for WebsocketServer {
    type Result = ();

    fn handle(&mut self, msg: Swap, _: &mut Context<Self>) {
        if let Some(recipient_addr) = self.sessions.remove(&msg.id) {
            debug!(
                "replacing socket identifier from {} to {}",
                &msg.id, &msg.new_id
            );

            self.sessions.insert(msg.new_id, recipient_addr);
        } else {
            warn!(
                "session with id {} does not exist, could not set session to id {}",
                &msg.id, &msg.new_id
            );
        }
    }
}

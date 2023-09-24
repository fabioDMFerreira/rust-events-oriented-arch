type LoginResponse = {
  token: string;
};

type UserResponse = {
  id: string;
  name: string;
};

export type Feeds = Array<{
  id: string;
  url: string;
  title: string;
  author: string;
  publish_date: string;
}>;

export type Subscription = {
  user_id: string;
  feed_id: string;
};

export type News = {
  title: string;
  author: string;
  url: string;
  publish_date: string;
  feed_id: string;
};

class API {
  token: string = '';

  _doRequest(
    input: RequestInfo | URL,
    init: RequestInit = {}
  ): Promise<Response> {
    return fetch(`/api${input}`, {
      ...init,
      mode: 'cors',
      headers: {
        Authorization: `Bearer ${this.token}`,
        'Content-Type': 'application/json',
      },
    }).then(async (resp) => {
      if (resp.status >= 200 && resp.status < 300) {
        const json = await resp.json();
        return json;
      } else if (resp.status === 401) {
        this._saveToken('');
      }
      const err = await resp.text();
      throw new Error(err);
    });
  }

  _init() {
    const authToken = window.localStorage.getItem('authToken');

    if (authToken) {
      this.token = authToken;
    }
  }

  _saveToken(token: string) {
    this.token = token;
    window.localStorage.setItem('authToken', token);
  }

  login(name: string, password: string): Promise<LoginResponse> {
    return this._doRequest('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ name, password }),
    })
      .then((resp: any) => {
        this._saveToken(resp.token);
        return resp;
      })
      .catch((err) => {
        console.log('caught error', err);
        const errorParsed = JSON.parse(err.message);

        if (errorParsed.message) {
          throw new Error(errorParsed.message);
        }
        console.log({ errorParsed });
        throw new Error('invalid username or password');
      }) as any;
  }

  logout(): Promise<any> {
    return this._doRequest('/auth/logout').then(() => {
      this._saveToken('');
    });
  }

  me(): Promise<UserResponse> {
    return this._doRequest('/auth/me') as any;
  }

  feeds(): Promise<Feeds> {
    return this._doRequest('/feeds') as any;
  }

  news(): Promise<News[]> {
    return this._doRequest('/news') as any;
  }

  subscriptions(): Promise<Subscription[]> {
    return this._doRequest('/subscriptions') as any;
  }

  subscribe(feedId: string): Promise<Subscription> {
    return this._doRequest('/subscriptions', {
      method: 'POST',
      body: JSON.stringify({ feed_id: feedId }),
    }) as any;
  }

  unsubscribe(feedId: string): Promise<number> {
    return this._doRequest(`/subscriptions?feed_id=${feedId}`, {
      method: 'DELETE',
    }) as any;
  }

  connectWs(onMessage: (event: MessageEvent<any>) => void) {
    const socket = new WebSocket('ws://localhost:8000/connect-ws');
    // const socket = new WebSocket(`ws://${window.location.host}/connect-ws`);

    let interval: string | number | NodeJS.Timer | undefined;

    // WebSocket event listeners
    socket.onopen = () => {
      console.log('WebSocket connection established.');
      socket.send('/login ' + this.token);

      interval = setInterval(() => {
        socket.send('ping');
      }, 1000);
    };

    socket.onmessage = onMessage;

    socket.onclose = () => {
      console.log('WebSocket connection closed.');
      clearInterval(interval);
    };

    return () => {
      socket.close();
    };
  }
}

const api = new API();

api._init();

export default api;

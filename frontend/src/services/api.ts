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

  login(name: string, password: string): Promise<LoginResponse> {
    return this._doRequest('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ name, password }),
    }).then((resp: any) => {
      window.localStorage.setItem('authToken', resp.token);
      return resp;
    }) as any;
  }

  me(): Promise<UserResponse> {
    return this._doRequest('/auth/me') as any;
  }

  feeds(): Promise<Feeds> {
    return this._doRequest('/feeds') as any;
  }

  connectWs(onMessage: (event: MessageEvent<any>) => void) {
    // const socket = new WebSocket('ws://localhost:8000/ws');
    const socket = new WebSocket(`ws://${window.location.host}/ws`);

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
    };

    return () => {
      socket.close();
      clearInterval(interval);
    };
  }
}

const api = new API();

// api.login = api.login.bind(api);
// api.me = api.me.bind(api);
// api.connectWs = api.connectWs.bind(api);

export default api;

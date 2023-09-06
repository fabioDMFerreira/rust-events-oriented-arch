import React, { useCallback, useState } from 'react';
import './App.css';
import WebSocketComponent from './components/WebsocketComponent';
import UserInfo from './components/UserInfo';

function App() {
  const [name, setName] = useState<string>();
  const [password, setPassword] = useState<string>();
  const [token, setToken] = useState<string>();
  const [isLoggedIn, setIsLoggedIn] = useState<boolean>();
  const [loginError, setLoginError] = useState<string>();

  const login = useCallback(() => {
    setLoginError('');

    fetch('http://localhost:8000/auth/login', {
      method: 'POST',
      mode: 'cors',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ name, password }),
    })
      .then(async (resp) => {
        if (resp.status === 200) {
          const json = await resp.json();
          setIsLoggedIn(true);
          setToken(json.token);
        } else {
          const err = await resp.text();
          setLoginError(err);
        }
      })
      .catch((err) => {
        console.log(err);
      });
  }, [name, password]);

  return (
    <div className="App">
      <div>
        {!isLoggedIn && (
          <form
            onSubmit={(e) => {
              e.preventDefault();
            }}
          >
            <input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Name"
            />
            <br />
            <input
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Password"
            />
            <br />
            <button onClick={login} disabled={!name || !password}>
              Login
            </button>
            <br />
            {loginError}
          </form>
        )}
        {isLoggedIn && token && (
          <div>
            <UserInfo token={token} />
            <WebSocketComponent token={token} />
          </div>
        )}
      </div>
    </div>
  );
}

export default App;

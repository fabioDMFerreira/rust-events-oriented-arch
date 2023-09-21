import React, { useCallback, useEffect, useState } from 'react';
import './App.css';
import WebSocketComponent from './components/WebsocketComponent';
import UserInfo from './components/UserInfo';
import api from './services/api';
import Feeds from './components/Feeds';

function App() {
  const [name, setName] = useState<string>();
  const [password, setPassword] = useState<string>();
  const [token, setToken] = useState<string>();
  const [isLoggedIn, setIsLoggedIn] = useState<boolean>();
  const [loginError, setLoginError] = useState<string>();

  useEffect(() => {
    if (api.token) {
      setIsLoggedIn(true);
    }
  }, []);

  const login = useCallback(() => {
    if (!name || !password) {
      return;
    }

    setLoginError('');

    api
      .login(name, password)
      .then((resp) => {
        setIsLoggedIn(true);
        setToken(resp.token);
      })
      .catch((err) => {
        setLoginError(err);
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
            <UserInfo />
            <Feeds />
            <WebSocketComponent />
          </div>
        )}
      </div>
    </div>
  );
}

export default App;

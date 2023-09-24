import React, { useCallback, useEffect, useState } from 'react';
import './App.css';
import WebSocketComponent from './components/WebsocketComponent';
import UserInfo from './components/UserInfo';
import api from './services/api';
import Feeds from './components/Feeds';
import Logout from './components/Logout';

function App() {
  const [name, setName] = useState<string>('');
  const [password, setPassword] = useState<string>('');
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
      .then(() => {
        setIsLoggedIn(true);
      })
      .catch((err) => {
        setLoginError(err.message);
      });
  }, [name, password]);

  const logout = useCallback(() => {
    api.logout().then(() => {
      setIsLoggedIn(false);
    });
  }, []);

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
        {isLoggedIn && (
          <div>
            <Logout logout={logout} />
            <UserInfo />
            <WebSocketComponent />
            <Feeds />
          </div>
        )}
      </div>
    </div>
  );
}

export default App;

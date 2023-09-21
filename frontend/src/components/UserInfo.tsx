import React, { useEffect, useState } from 'react';
import api from '../services/api';

interface Props {}

const UserInfo = (_: Props) => {
  const [user, setUser] = useState<object>();
  const [err, setError] = useState<Error | null>();

  useEffect(() => {
    setError(null);

    api
      .me()
      .then(async (user) => {
        setUser(user);
      })
      .catch((err) => setError(err));
  }, []);

  return <div>{user ? JSON.stringify(user) : err?.message}</div>;
};

export default UserInfo;

import React, { useEffect, useState } from 'react';

interface Props {
  token: string;
}

const UserInfo = ({ token }: Props) => {
  const [user, setUser] = useState<object>();
  const [err, setError] = useState<Error | null>();

  useEffect(() => {
    setError(null);

    fetch('http://localhost:8000/auth/me', {
      headers: {
        Authorization: `Bearer ${token}`,
      },
    })
      .then(async (resp) => {
        const user = await resp.json();
        setUser(user);
      })
      .catch((err) => setError(err));
  }, []);

  return <div>{user ? JSON.stringify(user) : err?.message}</div>;
};

export default UserInfo;

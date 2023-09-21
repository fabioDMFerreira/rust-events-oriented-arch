import React from 'react';

interface Props {
  logout: () => void;
}

const Logout = ({ logout }: Props) => {
  return (
    <div>
      <button onClick={logout}>Logout</button>
    </div>
  );
};

export default Logout;

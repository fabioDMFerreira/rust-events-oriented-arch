import React, { useEffect } from 'react';

interface Props {
  token: string;
}

const WebSocketComponent = ({ token }: Props) => {
  useEffect(() => {
    // Connect to WebSocket server
    const socket = new WebSocket('ws://localhost:8000/ws');

    let interval: string | number | NodeJS.Timer | undefined;

    // WebSocket event listeners
    socket.onopen = () => {
      console.log('WebSocket connection established.');
      socket.send('/login ' + token);

      interval = setInterval(() => {
        socket.send('ping');
      }, 1000);
    };

    socket.onmessage = (event) => {
      console.log('WebSocket message received:', event.data);
      // Handle the received message
    };

    socket.onclose = () => {
      console.log('WebSocket connection closed.');
    };

    // Clean up the WebSocket connection when the component unmounts
    return () => {
      socket.close();
      clearInterval(interval);
    };
  }, []);

  return <div>WebSocket Component</div>;
};

export default WebSocketComponent;

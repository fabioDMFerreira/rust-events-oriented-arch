import React, { useEffect } from 'react';
import api from '../services/api';

interface Props {}

const WebSocketComponent = (_: Props) => {
  useEffect(() => {
    const connDestruct = api.connectWs((event) => {
      console.log('WebSocket message received:', event.data);
    });

    // Clean up the WebSocket connection when the component unmounts
    return connDestruct;
  }, []);

  return <span aria-description="websocket-placeholder"></span>;
};

export default WebSocketComponent;

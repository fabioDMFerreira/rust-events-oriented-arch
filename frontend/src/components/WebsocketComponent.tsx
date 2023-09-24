import React, { useCallback, useEffect, useState } from 'react';
import api, { News } from '../services/api';

interface Props {}

const WebSocketComponent = (_: Props) => {
  const [lastNews, setLastNews] = useState<News[]>([]);

  const addLastNews = useCallback(
    (event: MessageEvent<any>) => {
      try {
        const news = JSON.parse(event.data);

        setLastNews([news, ...lastNews]);
      } catch (err) {
        console.log(err);
      }
      console.log('WebSocket message received:', event.data);
    },
    [lastNews]
  );

  useEffect(() => {
    const connDestruct = api.connectWs(addLastNews);

    // Clean up the WebSocket connection when the component unmounts
    return connDestruct;
  }, []);

  return (
    <>
      {lastNews.length !== 0 ? (
        <>
          <h2>Last News</h2>
          {lastNews.map((news) => {
            return (
              <p>
                {news.title} ({news.author}) {news.publish_date}
              </p>
            );
          })}
        </>
      ) : (
        <></>
      )}
    </>
  );
};

export default WebSocketComponent;

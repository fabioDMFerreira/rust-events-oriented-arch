import React, { useEffect, useState } from 'react';
import api, { Feeds } from '../services/api';

const FeedsComponent = () => {
  const [feeds, setFeeds] = useState<Feeds>();

  useEffect(() => {
    api.feeds().then(setFeeds).catch(console.log);
  }, []);

  return (
    <>
      {feeds
        ? feeds.map((feed) => {
            return (
              <div>
                {feed.title} <button>Subscribe</button>
              </div>
            );
          })
        : 'No feeds'}
    </>
  );
};

export default FeedsComponent;

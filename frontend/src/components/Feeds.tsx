import React, { useCallback, useEffect, useState } from 'react';
import api, { Feeds, News, Subscription } from '../services/api';

const FeedsComponent = () => {
  const [feeds, setFeeds] = useState<Feeds>();
  const [subscriptions, setSubscriptions] = useState<{
    [key: string]: boolean;
  }>({});
  const [news, setNews] = useState<News[]>();

  useEffect(() => {
    api.feeds().then(setFeeds).catch(console.log);
    api
      .subscriptions()
      .then((subscriptions) => {
        setSubscriptions(
          subscriptions.reduce((final, subscription: Subscription) => {
            final[subscription.feed_id] = true;
            return final;
          }, {} as { [key: string]: boolean })
        );
      })
      .catch(console.log);
  }, []);

  useEffect(() => {
    api.news().then(setNews).catch(console.log);
  }, [subscriptions]);

  const subscribe = useCallback(
    (feedId: string) => {
      api.subscribe(feedId).then(() => {
        setSubscriptions({
          ...subscriptions,
          [feedId]: true,
        });
      });
    },
    [subscriptions]
  );

  const unsubscribe = useCallback(
    (feedId: string) => {
      api.unsubscribe(feedId).then(() => {
        const newSubscriptions = { ...subscriptions };
        delete newSubscriptions[feedId];
        setSubscriptions(newSubscriptions);
      });
    },
    [subscriptions]
  );

  return (
    <>
      {feeds
        ? feeds.map((feed) => {
            return (
              <div>
                {feed.title}{' '}
                {subscriptions[feed.id] ? (
                  <button
                    onClick={() => {
                      unsubscribe(feed.id);
                    }}
                  >
                    Unsubscribe
                  </button>
                ) : (
                  <button
                    onClick={() => {
                      subscribe(feed.id);
                    }}
                  >
                    Subscribe
                  </button>
                )}
              </div>
            );
          })
        : 'No feeds'}
      <div>
        <h2>News</h2>
        <div>
          {news
            ? news.map((news) => {
                return (
                  <p>
                    {news.title} ({news.author}) {news.publish_date}
                  </p>
                );
              })
            : 'No news'}
        </div>
      </div>
    </>
  );
};

export default FeedsComponent;

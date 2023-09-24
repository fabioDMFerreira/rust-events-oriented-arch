const { createProxyMiddleware } = require('http-proxy-middleware');

module.exports = function (app) {
  app
    .use(
      '/api/news',
      createProxyMiddleware({
        target: 'http://news:8001',
        changeOrigin: true,
        pathRewrite: {
          '^/api/': '/',
        },
      })
    )
    .use(
      '/api/subscriptions',
      createProxyMiddleware({
        target: 'http://news:8001',
        changeOrigin: true,
        pathRewrite: {
          '^/api/': '/',
        },
      })
    )
    .use(
      '/api/feeds',
      createProxyMiddleware({
        target: 'http://news:8001',
        changeOrigin: true,
        pathRewrite: {
          '^/api/': '/',
        },
      })
    )
    .use(
      '/api/users',
      createProxyMiddleware({
        target: 'http://api:8000',
        changeOrigin: true,
        pathRewrite: {
          '^/api/': '/',
        },
      })
    )
    .use(
      '/api/auth',
      createProxyMiddleware({
        target: 'http://users:8000',
        changeOrigin: true,
        pathRewrite: {
          '^/api/': '/',
        },
      })
    )
    .use(
      '/connect-ws',
      createProxyMiddleware({
        ws: true,
        target: 'http://news:8001',
        changeOrigin: true,
      })
    );
};

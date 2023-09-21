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
        target: 'http://api:8000',
        changeOrigin: true,
        pathRewrite: {
          '^/api/': '/',
        },
      })
    )
    .use(
      '/ws',
      createProxyMiddleware({
        target: 'ws://api:8000',
        changeOrigin: true,
      })
    );
};

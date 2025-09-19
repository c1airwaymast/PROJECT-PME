export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    const targetUrl = 'http://airwaymast.org' + url.pathname + url.search;
    
    return fetch(targetUrl, {
      method: request.method,
      headers: request.headers,
      body: request.body
    });
  }
};
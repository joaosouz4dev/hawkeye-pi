// hawkeye-pi service worker minimo
// Nao faz cache offline: streaming e ao vivo, nao faz sentido cachear.
// Existe so pra o navegador aceitar como PWA "instalavel".
self.addEventListener('install', e => self.skipWaiting());
self.addEventListener('activate', e => e.waitUntil(self.clients.claim()));
self.addEventListener('fetch', e => { /* passa direto pra rede */ });

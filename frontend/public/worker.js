// service-worker.js
const CACHE_NAME = "zentrox-alerts-chache";
const OFFLINE_URL = "/alerts_offline.html";

// Precache the offline page
self.addEventListener("install", (event) => {
 event.waitUntil(
  caches.open(CACHE_NAME).then((cache) => {
   return cache.addAll([OFFLINE_URL]);
  }),
 );
});

// Handle fetch events
self.addEventListener("fetch", (event) => {
 event.respondWith(
  fetch(event.request).catch(() => {
   return caches.match(OFFLINE_URL); // Return the offline page if fetch fails
  }),
 );
});

// Cleanup old caches
self.addEventListener("activate", (event) => {
 const cacheWhitelist = [CACHE_NAME];
 event.waitUntil(
  caches.keys().then((cacheNames) => {
   return Promise.all(
    cacheNames.map((cacheName) => {
     if (cacheWhitelist.indexOf(cacheName) === -1) {
      return caches.delete(cacheName);
     }
    }),
   );
  }),
 );
});

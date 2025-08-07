const log = (...args) => {
  console.log('[LiveReload]', ...args);
};
var errorcount = 0;

function initLiveReload() {
  log('Initializing live reload...');
  const eventSource = new EventSource('/__livereload');

  eventSource.addEventListener('start', function() {
    log('Server restarted, reloading page...');
    setTimeout(() => window.location.reload(), 1000);
  });

  eventSource.addEventListener('heartbeat', function() {
    log('Live reload heartbeat');
  });

  eventSource.addEventListener('error', function() {
    log('Live reload connection error, retrying...');
    eventSource.close();
    if (errorcount > 20) {
      log('Too many errors, stopping live reload');
      return;
    }
    errorcount++;
    setTimeout(initLiveReload, 1000 + errorcount * 1000);
  });

  window.addEventListener('beforeunload', function() {
    eventSource.close();
    return true;
  });
};

setTimeout(initLiveReload, 1000);

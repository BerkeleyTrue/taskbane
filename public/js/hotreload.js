const log = (...args) => {
    console.log('[HotReload]', ...args);
};

function initHotReload() {
    log('Initializing hot reload...');
    const eventSource = new EventSource('/__hotreload');

    eventSource.addEventListener('start', function(event) {
        log('Server restarted, reloading page...');
        setTimeout(() => window.location.reload(), 1000);
    });

    eventSource.addEventListener('heartbeat', function(event) {
        log('Hot reload heartbeat');
    });

    eventSource.addEventListener('error', function(event) {
        log('Hot reload connection error, retrying...');
        eventSource.close();
        setTimeout(initHotReload, 1000);
    });
  
    eventSource.onerror = function(event) {
        log('Hot reload error:', event);
        eventSource.close();
        setTimeout(initHotReload, 1000);
    };

    window.addEventListener('beforeunload', function() {
        eventSource.close();
        return true;
    });
};

setTimeout(initHotReload, 1000);

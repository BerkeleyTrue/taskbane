(function() {
    'use strict';

    const eventSource = new EventSource('/__hotreload');

    eventSource.addEventListener('start', function(event) {
        console.log('Server restarted, reloading page...');
        window.location.reload();
    });

    eventSource.addEventListener('heartbeat', function(event) {
        console.log('Hot reload heartbeat');
    });

    eventSource.addEventListener('error', function(event) {
        console.log('Hot reload connection error, retrying...');
    });

    console.log('Hot reload script loaded');
})();
// HTMX loading bar
(function () {
  var inFlight = 0;
  var bar = document.getElementById('loading-bar');
  document.body.addEventListener('htmx:beforeRequest', function () {
    inFlight++;
    bar.classList.add('active');
  });
  document.body.addEventListener('htmx:afterRequest', function () {
    inFlight = Math.max(0, inFlight - 1);
    if (inFlight === 0) bar.classList.remove('active');
  });
})();

// Alert helper
function showAlert(level, message) {
  var container = document.getElementById('alert-container');
  var template = document.getElementById('alert-template');
  if (!container || !template) {
    return;
  }

  var el = template.content.cloneNode(true).querySelector('.alert');

  el.classList.add('alert-' + level);
  el.querySelector('[data-message]').textContent = message;

  el.querySelector('button').addEventListener('click', function () {
    el.classList.add('dismissing');
    el.addEventListener(
      'transitionend',
      function () {
        el.remove();
      },
      { once: true }
    );
  });

  container.appendChild(el);

  if (level !== 'error') {
    setTimeout(function () {
      el.classList.add('dismissing');
      el.addEventListener(
        'transitionend',
        function () {
          el.remove();
        },
        { once: true }
      );
    }, 5000);
  }
}

// HTMX error alerts
(function () {
  document.body.addEventListener('htmx:responseError', function (event) {
    var status = event.detail.xhr.status;
    if (status !== 400) return;
    var text = event.detail.xhr.statusText || 'Bad request';
    showAlert('error', status + ' ' + text);
  });

  document.body.addEventListener('htmx:sendError', function () {
    showAlert('error', 'Network error: could not reach the server');
  });
})();

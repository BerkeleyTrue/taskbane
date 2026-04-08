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

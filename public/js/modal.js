var isOpenClass = 'modal-is-open';
var isOpeningClass = 'modal-is-opening';
var isClosingClass = 'modal-is-closing';
var scrollbarWidthCssVar = '--pico-scrollbar-width';
var animationDuration = 400; // ms
var visibleModal = null;

var logger = (log) => {
  console.log('modal: ' + log);
};

function toggleModal(event) {
  event.preventDefault();
  const modal = document.getElementById(event.currentTarget.dataset.target);
  if (!modal) return;
  modal && (modal.open ? closeModal(modal) : openModal(modal));
}

function openModal(modal) {
  const { documentElement: html } = document;
  const scrollbarWidth = getScrollbarWidth();
  if (scrollbarWidth) {
    html.style.setProperty(scrollbarWidthCssVar, `${scrollbarWidth}px`);
  }
  html.classList.add(isOpenClass, isOpeningClass);
  setTimeout(() => {
    visibleModal = modal;
    html.classList.remove(isOpeningClass);
  }, animationDuration);
  modal.showModal();
}

function closeModal(modal) {
  visibleModal = null;
  const { documentElement: html } = document;
  html.classList.add(isClosingClass);
  setTimeout(() => {
    html.classList.remove(isClosingClass, isOpenClass);
    html.style.removeProperty(scrollbarWidthCssVar);
    modal.close();
  }, animationDuration);
}

document.addEventListener('htmx:beforeSwap', function(event) {
  if (!visibleModal) return;

  event.detail.shouldSwap = false;

  const response = event.detail.serverResponse;
  const target = event.detail.target;

  const replaceUrl = event.detail.xhr.getResponseHeader('HX-Replace-Url');

  closeModal(visibleModal);

  setTimeout(() => {
    htmx.swap(target, response, { swapStyle: 'outerHTML' });
    if (replaceUrl) {
      history.replaceState(null, '', replaceUrl);
    }
  }, animationDuration);
});

// Close with a click outside
document.addEventListener('click', (event) => {
  if (visibleModal === null) return;
  const modalContent = visibleModal.querySelector('article');
  const isClickInside = modalContent.contains(event.target);
  !isClickInside && closeModal(visibleModal);
});

// Close with Esc key
document.addEventListener('keydown', (event) => {
  if (event.key === 'Escape' && visibleModal) {
    closeModal(visibleModal);
  }
});

function getScrollbarWidth() {
  const scrollbarWidth =
    window.innerWidth - document.documentElement.clientWidth;
  return scrollbarWidth;
}

function isScrollbarVisible() {
  return document.body.scrollHeight > screen.height;
}

const isOpenClass = 'modal-is-open';
const isOpeningClass = 'modal-is-opening';
const isClosingClass = 'modal-is-closing';
const scrollbarWidthCssVar = '--pico-scrollbar-width';
const animationDuration = 400; // ms
var visibleModal = null;

var logger = (log) => {
  console.log("modal: " + log);
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

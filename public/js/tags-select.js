(function () {
  const widget      = document.getElementById('tags-widget');
  const input       = document.getElementById('tags-input');
  const pillsEl     = document.getElementById('tags-pills');
  const pillTemplate = document.getElementById('tag-pill-template');
  const selected    = new Set();

  function renderHiddenInputs() {
    widget.querySelectorAll('input[name="tags"]').forEach((el) => el.remove());
    for (const tag of selected) {
      const inp = document.createElement('input');
      inp.type = 'hidden';
      inp.name = 'tags';
      inp.value = tag;
      widget.appendChild(inp);
    }
  }

  function addTag(raw) {
    const tag = raw.trim().toLowerCase();
    if (!tag || selected.has(tag)) return;
    selected.add(tag);

    const pill = pillTemplate.content.cloneNode(true).querySelector('.dep-pill');
    pill.querySelector('span').textContent = tag;
    pill.querySelector('button').addEventListener('click', (e) => {
      e.stopPropagation();
      selected.delete(tag);
      pill.remove();
      renderHiddenInputs();
    });

    pillsEl.appendChild(pill);
    renderHiddenInputs();
  }

  input.addEventListener('keydown', (e) => {
    if (e.key === ',' || e.key === 'Enter') {
      e.preventDefault();
      addTag(input.value);
      input.value = '';
    } else if (e.key === 'Backspace' && input.value === '') {
      const last = [...selected].at(-1);
      if (last) {
        selected.delete(last);
        pillsEl.lastElementChild?.remove();
        renderHiddenInputs();
      }
    }
  });

  // also handle paste of comma-separated values
  input.addEventListener('paste', (e) => {
    e.preventDefault();
    const text = e.clipboardData.getData('text');
    text.split(',').forEach(addTag);
  });
})();

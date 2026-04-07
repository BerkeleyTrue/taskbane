(function () {
  const allTasks = window.__TASKS__ || [];

  const widget = document.getElementById('deps-widget');
  const search = document.getElementById('deps-search');
  const results = document.getElementById('deps-results');
  const pillsEl = document.getElementById('deps-pills');
  const pillTemplate = document.getElementById('dep-pill-template');
  // id -> { id, uuid, description }
  const selected = new Map();

  function renderHiddenInputs() {
    widget.querySelectorAll('input[name="deps"]').forEach((el) => el.remove());

    for (const [, task] of selected) {
      const inp = document.createElement('input');
      inp.type = 'hidden';
      inp.name = 'deps';
      inp.value = task.uuid;
      widget.appendChild(inp);
    }
  }

  function addPill(task) {
    if (selected.has(task.id)) return;
    selected.set(task.id, task);

    const pill = pillTemplate.content
      .cloneNode(true)
      .querySelector('.dep-pill');
    pill.dataset.id = task.id;

    const label = pill.querySelector('[data-tooltip]');
    label.dataset.tooltip = task.description;
    label.textContent = task.id;

    pill.querySelector('button').addEventListener('click', (e) => {
      e.stopPropagation();
      selected.delete(task.id);
      pill.remove();
      renderHiddenInputs();
    });

    pillsEl.appendChild(pill);

    renderHiddenInputs();
  }

  function buildResults(tasks) {
    results.innerHTML = '';

    tasks.forEach((task) => {
      const li = document.createElement('li');
      li.setAttribute('role', 'option');
      li.innerHTML = `<span class="task-id">${task.id}</span>${task.description}`;

      li.addEventListener('mousedown', (e) => {
        e.preventDefault();
        addPill(task);
        search.value = '';
        results.innerHTML = '';
      });

      results.appendChild(li);
    });
  }

  search.addEventListener('input', () => {
    const q = search.value.trim().toLowerCase();
    if (!q) {
      results.innerHTML = '';
      return;
    }

    const matches = allTasks.filter(
      (t) =>
        !selected.has(t.id) &&
        (String(t.id).includes(q) || t.description.toLowerCase().includes(q))
    );

    buildResults(matches);
  });

  document.addEventListener('click', (e) => {
    if (!widget.contains(e.target)) {
      results.innerHTML = '';
    }
  });

  search.addEventListener('keydown', (e) => {
    const items = [...results.querySelectorAll('li')];
    const cur = results.querySelector('[aria-selected="true"]');

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      const next = cur ? items[items.indexOf(cur) + 1] || items[0] : items[0];
      if (cur) {
        cur.removeAttribute('aria-selected');
      }

      if (next) {
        next.setAttribute('aria-selected', 'true');
      }
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      const prev = cur
        ? items[items.indexOf(cur) - 1] || items[items.length - 1]
        : items[items.length - 1];

      if (cur) {
        cur.removeAttribute('aria-selected');
      }
      if (prev) {
        prev.setAttribute('aria-selected', 'true');
      }
    } else if (e.key === 'Enter') {
      if (items.length > 0) {
        e.preventDefault();
      }
      if (cur) {
        cur.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
      }
    } else if (e.key === 'Escape') {
      results.innerHTML = '';
    }
  });
})();

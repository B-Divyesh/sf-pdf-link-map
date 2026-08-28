import './style.css';

type Specimen = {
  heading: string;
  stamp: string;
  tone: 'fail' | 'pass' | 'note';
  note: string;
  metrics: [string, string][];
  rows: [string, string, string, string][];
};

const specimens: Record<string, Specimen> = {
  broken: {
    heading: '2 routes need attention', stamp: 'FAIL', tone: 'fail',
    note: '↳ The printed contents still looks correct. These defects exist only in the annotation layer.',
    metrics: [['14', 'links'], ['9', 'valid internal'], ['3', 'external'], ['2', 'broken']],
    rows: [['p. 1', 'Installation → install', '✓ Valid', 'valid'], ['p. 1', 'Figure 4 → fig-04', '✕ Missing anchor', 'broken'], ['p. 2', 'Appendix → page 41', '✕ Page absent', 'broken'], ['p. 6', 'Support website', '↗ Recorded only', 'external']]
  },
  clean: {
    heading: 'All routes accounted for', stamp: 'PASS', tone: 'pass',
    note: '✓ Ready to attach to the release record. External addresses were not requested.',
    metrics: [['14', 'links'], ['11', 'valid internal'], ['3', 'external'], ['0', 'broken']],
    rows: [['p. 1', 'Installation → install', '✓ Valid', 'valid'], ['p. 1', 'Figure 4 → fig-04', '✓ Valid', 'valid'], ['p. 2', 'Appendix → page 38', '✓ Valid', 'valid'], ['p. 6', 'Support website', '↗ Recorded only', 'external']]
  },
  empty: {
    heading: 'No annotations found', stamp: 'CHECK', tone: 'note',
    note: '△ If the source document had a contents page, its converter probably dropped the navigation layer.',
    metrics: [['0', 'links'], ['0', 'valid internal'], ['0', 'external'], ['1', 'warning']], rows: []
  }
};

function renderSpecimen(name: string): void {
  const specimen = specimens[name] ?? specimens.broken;
  const heading = document.querySelector<HTMLElement>('#specimen-heading');
  const stamp = document.querySelector<HTMLElement>('#specimen-stamp');
  const metrics = document.querySelector<HTMLElement>('#specimen-metrics');
  const rows = document.querySelector<HTMLElement>('#specimen-rows');
  const note = document.querySelector<HTMLElement>('#specimen-note');
  if (!heading || !stamp || !metrics || !rows || !note) return;
  heading.textContent = specimen.heading;
  stamp.textContent = specimen.stamp;
  stamp.className = `stamp ${specimen.tone}`;
  metrics.replaceChildren(...specimen.metrics.map(([value, label]) => {
    const item = document.createElement('p');
    const strong = document.createElement('strong'); strong.textContent = value;
    item.append(strong, document.createTextNode(label)); return item;
  }));
  if (specimen.rows.length === 0) {
    const tr = document.createElement('tr'); const td = document.createElement('td');
    td.colSpan = 3; td.className = 'empty-row'; td.textContent = 'No link annotations to map.'; tr.append(td); rows.replaceChildren(tr);
  } else {
    rows.replaceChildren(...specimen.rows.map(([from, destination, result, kind]) => {
      const tr = document.createElement('tr');
      [from, destination, result].forEach((value, index) => { const td = document.createElement('td'); td.textContent = value; if (index === 2) td.className = `result ${kind}`; tr.append(td); });
      return tr;
    }));
  }
  note.textContent = specimen.note;
}

document.querySelectorAll<HTMLButtonElement>('[data-specimen]').forEach(button => button.addEventListener('click', () => {
  document.querySelectorAll<HTMLButtonElement>('[data-specimen]').forEach(item => { const active = item === button; item.classList.toggle('is-active', active); item.setAttribute('aria-pressed', String(active)); });
  renderSpecimen(button.dataset.specimen ?? 'broken');
}));
renderSpecimen('broken');

document.querySelectorAll<HTMLButtonElement>('[data-copy]').forEach(button => button.addEventListener('click', async () => {
  try { await navigator.clipboard.writeText(button.dataset.copy ?? ''); button.textContent = 'Copied'; }
  catch { button.textContent = 'Select the command above'; }
  window.setTimeout(() => { button.textContent = 'Copy command'; }, 1800);
}));

function showNetworkState(message?: string): void {
  const target = document.querySelector<HTMLElement>('#network-state');
  if (target) target.textContent = message ?? (navigator.onLine ? 'Works offline after first visit.' : 'Offline — docs and specimen remain available.');
}
window.addEventListener('online', () => showNetworkState()); window.addEventListener('offline', () => showNetworkState()); showNetworkState();
if ('serviceWorker' in navigator && import.meta.env.PROD) {
  // A module can run after `load` when restored from the back/forward cache or
  // injected by a host. Register now instead of waiting for an event we may have
  // missed; registration itself is safe to repeat on every visit.
  void navigator.serviceWorker.register('/sw.js', { scope: '/' }).catch(() => {
    showNetworkState('Offline cache could not start. The docs are still available online.');
  });
}

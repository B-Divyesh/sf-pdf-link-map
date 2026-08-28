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

const slug = 'pdf-link-map';
const apiBase = (import.meta.env.VITE_BILLING_API_BASE as string | undefined) ?? 'https://api.sociobot.in';
const licenseKey = `sb_license:${slug}`;
const verdictKey = `sb_license_verdict:${slug}`;
const day = 86_400_000;
type Verdict = { valid: boolean; checkedAt: number; reason?: string };

function readVerdict(): Verdict | null {
  try { return JSON.parse(localStorage.getItem(verdictKey) ?? 'null') as Verdict | null; } catch { return null; }
}
function setUnlocked(unlocked: boolean, message: string): void {
  const kit = document.querySelector<HTMLElement>('#team-kit'); const status = document.querySelector<HTMLElement>('#license-status');
  if (kit) kit.hidden = !unlocked; if (status) status.textContent = message;
}
async function verifyLicense(token: string, force = false): Promise<void> {
  const cached = readVerdict();
  if (cached?.valid) setUnlocked(true, navigator.onLine ? 'Team notebook unlocked.' : 'Team notebook unlocked from the last check; verification will resume online.');
  else if (cached) setUnlocked(false, 'License no longer active. You can buy a new unlock above.');
  if (!force && cached && Date.now() - cached.checkedAt < day) return;
  if (!navigator.onLine) { if (!cached?.valid) setUnlocked(false, 'Offline. Connect once to verify this license.'); return; }
  const status = document.querySelector<HTMLElement>('#license-status'); if (status) status.textContent = 'Checking license…';
  try {
    const response = await fetch(`${apiBase}/api/v1/products/${slug}/verify?license=${encodeURIComponent(token)}`);
    if (!response.ok) throw new Error('verification unavailable');
    const data = await response.json() as { valid: boolean; reason?: string };
    const verdict = { valid: data.valid, checkedAt: Date.now(), reason: data.reason }; localStorage.setItem(verdictKey, JSON.stringify(verdict));
    setUnlocked(data.valid, data.valid ? 'Team notebook unlocked.' : 'License no longer active. You can buy a new unlock above.');
  } catch { setUnlocked(Boolean(cached?.valid), cached?.valid ? 'Verification unavailable; using the last valid check.' : 'Could not verify right now. Your free tools still work.'); }
}

const query = new URLSearchParams(location.search); const returnedLicense = query.get('license');
if (returnedLicense) { localStorage.setItem(licenseKey, returnedLicense); query.delete('license'); history.replaceState({}, '', `${location.pathname}${query.size ? `?${query}` : ''}${location.hash}`); void verifyLicense(returnedLicense, true); }
else { const stored = localStorage.getItem(licenseKey); if (stored) void verifyLicense(stored); }

document.querySelector<HTMLFormElement>('#license-form')?.addEventListener('submit', event => {
  event.preventDefault(); const input = document.querySelector<HTMLInputElement>('#license-token'); const token = input?.value.trim();
  if (!token) return; localStorage.setItem(licenseKey, token); localStorage.removeItem(verdictKey); if (input) input.value = ''; void verifyLicense(token, true);
});

document.querySelector<HTMLButtonElement>('#generate-policy')?.addEventListener('click', () => {
  const provider = document.querySelector<HTMLSelectElement>('#ci-provider')?.value ?? 'github';
  const pdf = document.querySelector<HTMLInputElement>('#pdf-path')?.value.trim() || 'dist/handbook.pdf';
  const manifest = document.querySelector<HTMLInputElement>('#manifest-path')?.value.trim();
  const args = `pdf-link-map ${JSON.stringify(pdf)}${manifest ? ` --manifest ${JSON.stringify(manifest)}` : ''} --fail-on broken --json`;
  const recipe = provider === 'github' ? `- name: Audit PDF navigation\n  run: ${args}` : `# PDF navigation release gate\n${args}`;
  const output = document.querySelector<HTMLElement>('#policy-output'); if (output) output.textContent = recipe;
});

function showNetworkState(): void { const target = document.querySelector<HTMLElement>('#network-state'); if (target) target.textContent = navigator.onLine ? 'Works offline after first visit.' : 'Offline — docs and specimen remain available.'; }
window.addEventListener('online', showNetworkState); window.addEventListener('offline', showNetworkState); showNetworkState();
if ('serviceWorker' in navigator && import.meta.env.PROD) window.addEventListener('load', () => { void navigator.serviceWorker.register('/sw.js'); });

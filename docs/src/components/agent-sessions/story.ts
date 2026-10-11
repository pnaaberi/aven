// The scripted agent sessions on the landing page, and the aven queue they change.
// Command output follows real aven output; the queue follows the TUI queue view.

export type Result = string | { text: string; fx?: string };

export type Step =
  | { kind: 'earlier'; text: string }
  | { kind: 'context'; html: string }
  | { kind: 'user'; html: string; typed?: boolean }
  | { kind: 'tool'; name: string; cmd: string; body?: string[]; result?: Result[]; fx?: string; ms?: number }
  | { kind: 'agent'; html: string; cursor?: boolean }
  | { kind: 'proposal'; intro: string; items: string[] };

export interface Scene {
  name: string;
  line: string;
  title: string;
  /** Caption for a scene that starts a new session or continues the previous one. */
  start?: string;
  steps: Step[];
}

const ref = (r: string) => `<span class="violet">${r}</span>`;

export const scenes: Scene[] = [
  {
    name: 'Asides become tasks',
    line: 'A finding outside the change is filed with its repro and linked to the work it blocks.',
    title: '~/code/app',
    steps: [
      { kind: 'earlier', text: 'Earlier in this session · reviewing the login flow' },
      { kind: 'context', html: 'Review done. One finding is outside this change: token revocation shares the login rate limiter, so failed-login bursts delay revocations.' },
      { kind: 'user', html: 'Separate fix. File it with the repro, it blocks the public API launch.', typed: true },
      { kind: 'tool', name: 'Bash', cmd: 'aven search "revocation rate limit"', result: [''] },
      { kind: 'tool', name: 'Bash', cmd: 'aven search "public API"', fx: 'find',
        result: ['APP-M4R2 status=todo priority=high project=app labels= match=title score=412880 title="Open the public API"'] },
      { kind: 'tool', name: 'Bash', cmd: `aven add "Give token revocation its own rate limit" --status backlog --description-stdin <<'EOF'`, fx: 'add',
        body: ['Repro: exhaust the login limiter, then revoke a token.', 'The revocation waits for the login window to reset.', 'Fix: separate limiter in src/auth/limits.rs, plus a test.', 'EOF'],
        result: ['created APP-P8N3 ref=P8N3 project=app status=backlog priority=none title="Give token revocation its own rate limit"'] },
      { kind: 'tool', name: 'Bash', cmd: 'aven dep add APP-M4R2 APP-P8N3', fx: 'dep',
        result: ['dependency-added APP-M4R2 changed=yes depends_on=APP-P8N3'] },
      { kind: 'agent', html: `Filed ${ref('APP-P8N3')} with the repro. ${ref('APP-M4R2')} Open the public API now waits on it.` },
    ],
  },
  {
    name: 'Any agent picks it up',
    line: 'A new session in another worktree starts from the ref, records a decision, and ships it.',
    title: '~/code/app__worktrees/revocation-limit',
    start: 'Another worktree · new agent session',
    steps: [
      { kind: 'user', html: `Do ${ref('APP-P8N3')}.`, typed: true },
      { kind: 'tool', name: 'Bash', cmd: 'aven context APP-P8N3', fx: 'read', result: [
        'context APP-P8N3 suffix=P8N3 id=P8N3KMMQQ2NKP7FD status=backlog priority=none deleted=no epic=no blocked=no conflicts=no blocks_open=yes labels= title="Give token revocation its own rate limit"',
        'project=app prefix=APP', 'workspace=default', 'created=2026-10-11T09:14:02Z updated=2026-10-11T09:14:02Z',
        'description<<EOF', 'Repro: exhaust the login limiter, then revoke a token.', 'The revocation waits for the login window to reset.',
        'Fix: separate limiter in src/auth/limits.rs, plus a test.', 'EOF',
        'blocks open=1 total=1', '- APP-M4R2 status=todo unresolved=yes title="Open the public API"'] },
      { kind: 'tool', name: 'Bash', cmd: 'aven edit APP-P8N3 --status active', fx: 'active',
        result: ['updated APP-P8N3 changed=yes status=active priority=none title="Give token revocation its own rate limit"'] },
      { kind: 'agent', html: 'Revocations get their own budget. Per client, or one global reserve?' },
      { kind: 'user', html: 'Per client.' },
      { kind: 'tool', name: 'Bash', cmd: 'aven note APP-P8N3 "Decision: per-client revocation budget, separate from login."', fx: 'note',
        result: ['noted APP-P8N3 note=K4W9T2XQ7MB3RD5E'] },
      { kind: 'tool', name: 'Edit', cmd: 'src/auth/limits.rs', ms: 500, result: ['Updated src/auth/limits.rs with 38 additions and 6 removals'] },
      { kind: 'tool', name: 'Bash', cmd: 'cargo test limits', ms: 1400, result: ['test result: ok. 14 passed; 0 failed'] },
      { kind: 'tool', name: 'Bash', cmd: 'aven edit APP-P8N3 --status done', fx: 'done ship',
        result: ['updated APP-P8N3 changed=yes status=done priority=none title="Give token revocation its own rate limit"'] },
      { kind: 'agent', html: `${ref('APP-P8N3')} is done. ${ref('APP-M4R2')} was waiting on it and is ready now.`, cursor: true },
    ],
  },
  {
    name: 'Designs become epics',
    line: 'A design discussion turns into an epic with tasks and a dependency.',
    title: '~/code/app',
    start: 'New agent session',
    steps: [
      { kind: 'user', html: "How should we do API key rotation? It can't break running clients.", typed: true },
      { kind: 'tool', name: 'Read', cmd: 'src/auth/keys.rs', ms: 300, result: ['Read 142 lines'] },
      { kind: 'proposal', intro: 'Version the keys. Rotating issues a new key, the old one keeps working for a 7-day grace period, then a sweep expires it. Three parts:',
        items: ['1. An endpoint that issues the new key version', '2. A sweep that expires old keys (needs 1)', '3. Client docs for rotating without downtime'] },
      { kind: 'user', html: 'Sounds good. Make it an epic with tasks.' },
      { kind: 'tool', name: 'Bash', cmd: `aven add "API key rotation" --epic --description-stdin <<'EOF'`, fx: 'epic',
        body: ['Versioned keys: rotating issues a new version, the old', 'one stays valid for 7 days, then a sweep expires it.', 'EOF'],
        result: ['created APP-K1RT ref=K1RT project=app status=inbox priority=none title="API key rotation"'] },
      { kind: 'tool', name: 'Bash', cmd: `aven add "Add key rotation endpoint" --description-stdin <<'EOF'`, ms: 300,
        body: ['POST /keys/:id/rotate issues version n+1.', 'EOF',
          `aven add "Expire old keys after the grace period" --description-stdin <<'EOF'`, 'A sweep expires a key 7 days after it is replaced.', 'EOF',
          `aven add "Document key rotation" --description-stdin <<'EOF'`, 'Client guide: rotate, deploy the new key, let the old one lapse.', 'EOF'],
        result: [
          { text: 'created APP-R7W2 ref=R7W2 project=app status=inbox priority=none title="Add key rotation endpoint"', fx: 'kid1' },
          { text: 'created APP-R8E5 ref=R8E5 project=app status=inbox priority=none title="Expire old keys after the grace period"', fx: 'kid2' },
          { text: 'created APP-R9D3 ref=R9D3 project=app status=inbox priority=none title="Document key rotation"', fx: 'kid3' }] },
      { kind: 'tool', name: 'Bash', cmd: 'for r in R7W2 R8E5 R9D3; do aven epic add APP-$r APP-K1RT; done', fx: 'link',
        result: ['epic-added APP-R7W2 changed=yes epic=APP-K1RT', 'epic-added APP-R8E5 changed=yes epic=APP-K1RT', 'epic-added APP-R9D3 changed=yes epic=APP-K1RT'] },
      { kind: 'tool', name: 'Bash', cmd: 'aven dep add APP-R8E5 APP-R7W2', fx: 'block',
        result: ['dependency-added APP-R8E5 changed=yes depends_on=APP-R7W2'] },
      { kind: 'agent', html: `${ref('APP-K1RT')} has three tasks. Expiry waits on the endpoint.` },
    ],
  },
  {
    name: "What's next",
    line: 'The agent reads the ready list and explains its pick.',
    title: '~/code/app',
    start: 'Same session',
    steps: [
      { kind: 'user', html: "What's next?", typed: true },
      { kind: 'tool', name: 'Bash', cmd: 'aven list --ready', fx: 'next', result: [
        'APP-M4R2 status=todo priority=high labels= title="Open the public API"',
        'APP-7KQ9 status=todo priority=medium labels= title="Add scoped API tokens"',
        'APP-R7W2 status=inbox priority=none labels= blocks=1 title="Add key rotation endpoint"',
        'APP-R9D3 status=inbox priority=none labels= title="Document key rotation"'] },
      { kind: 'agent', html: `${ref('APP-M4R2')} Open the public API is the only high-priority task. In the epic, start with ${ref('APP-R7W2')}, since key expiry waits on it.`, cursor: true },
    ],
  },
];

/** The scene shown without motion: the pickup, the core of the story. */
export const STATIC_SCENE = 1;

// ---------- Queue model ----------

interface Task {
  ref: string;
  title: string;
  status: 'todo' | 'active' | 'inbox' | 'backlog' | 'done';
  pri?: 'high' | 'medium';
  deps?: string[];
  note?: boolean;
  epic?: boolean;
  parent?: string;
  band?: Band;
}

export interface Queue {
  tasks: Task[];
  /** The selected epic; the TUI marks its children with ↳. */
  sel: string | null;
}

/** Rows to highlight after a change: flash for writes, peek for reads. */
export interface Effects {
  flash?: string[];
  peek?: string[];
}

export const initialQueue = (): Queue => ({
  sel: null,
  tasks: [
    { ref: 'APP-M4R2', title: 'Open the public API', status: 'todo', pri: 'high' },
    { ref: 'APP-7KQ9', title: 'Add scoped API tokens', status: 'todo', pri: 'medium' },
  ],
});

const task = (q: Queue, r: string) => q.tasks.find((t) => t.ref === r)!;
const kid = (r: string, title: string) => (q: Queue): Effects => {
  q.tasks.push({ ref: r, title, status: 'inbox' });
  return { flash: [r] };
};

export const FX: Record<string, (q: Queue) => Effects> = {
  find: () => ({ peek: ['APP-M4R2'] }),
  add: (q) => {
    q.tasks.push({ ref: 'APP-P8N3', title: 'Give token revocation its own rate limit', status: 'backlog' });
    return { flash: ['APP-P8N3'] };
  },
  dep: (q) => { task(q, 'APP-M4R2').deps = ['APP-P8N3']; return { flash: ['APP-M4R2'] }; },
  read: () => ({ peek: ['APP-P8N3', 'APP-M4R2'] }),
  active: (q) => { task(q, 'APP-P8N3').status = 'active'; return { flash: ['APP-P8N3'] }; },
  note: (q) => { task(q, 'APP-P8N3').note = true; return {}; },
  done: (q) => { task(q, 'APP-P8N3').status = 'done'; return { flash: ['APP-P8N3'] }; },
  ship: (q) => { q.tasks = q.tasks.filter((t) => t.ref !== 'APP-P8N3'); return { flash: ['APP-M4R2'] }; },
  epic: (q) => {
    q.tasks.push({ ref: 'APP-K1RT', title: 'API key rotation', status: 'inbox', epic: true });
    return { flash: ['APP-K1RT'] };
  },
  kid1: kid('APP-R7W2', 'Add key rotation endpoint'),
  kid2: kid('APP-R8E5', 'Expire old keys after the grace period'),
  kid3: kid('APP-R9D3', 'Document key rotation'),
  link: (q) => {
    ['APP-R7W2', 'APP-R8E5', 'APP-R9D3'].forEach((r) => { task(q, r).parent = 'APP-K1RT'; });
    q.sel = 'APP-K1RT';
    return {};
  },
  block: (q) => { q.sel = null; task(q, 'APP-R8E5').deps = ['APP-R7W2']; return { flash: ['APP-R8E5'] }; },
  next: () => ({ peek: ['APP-M4R2', 'APP-R7W2'] }),
};

export const fxNames = (fx?: string) => (fx ?? '').split(' ').filter(Boolean);

/** Queue changes in the order a scene fires them: result lines first, then the tool's own. */
export function sceneFx(scene: Scene) {
  return scene.steps.flatMap((s) => (s.kind === 'tool'
    ? [...(s.result ?? []).flatMap((r) => (typeof r === 'string' ? [] : fxNames(r.fx))), ...fxNames(s.fx)]
    : []));
}

/** The queue as the scenes before `scene` left it. */
export function queueBefore(scene: number) {
  const q = initialQueue();
  scenes.slice(0, scene).flatMap(sceneFx).forEach((name) => FX[name](q));
  return q;
}

// ---------- Queue view ----------
// Bands and their order follow crates/aven-core/src/queue.rs.

type Band = 'focus' | 'soon' | 'triage' | 'blocked' | 'later' | 'epics';
const BANDS: Band[] = ['focus', 'soon', 'triage', 'blocked', 'later', 'epics'];

const blockers = (q: Queue, t: Task) => (t.deps ?? []).filter((r) => q.tasks.some((x) => x.ref === r)).length;
const dependents = (q: Queue, t: Task) => q.tasks.filter((x) => (x.deps ?? []).includes(t.ref)).length;

// A finished task keeps its band until it leaves the queue.
function bandOf(q: Queue, t: Task): Band {
  if (t.epic) return 'epics';
  if (t.status === 'done') return t.band!;
  if (blockers(q, t)) return 'blocked';
  if (t.status === 'active' || (t.status === 'todo' && t.pri === 'high')) return 'focus';
  if (t.status === 'todo' && t.pri === 'medium') return 'soon';
  if (t.status === 'inbox') return 'triage';
  return 'later';
}

const STATUS = { todo: '□ todo', active: '● active', inbox: '▣ inbox', backlog: '◌ back', done: '✓ done' };
const PRIORITY = { high: '<span class="high">●</span>', medium: '<span class="medium">◐</span>' };

function cells(q: Queue, t: Task) {
  const [project, suffix] = t.ref.split('-');
  const marks = [
    t.epic && '<span class="star">★</span>',
    q.sel && t.parent === q.sel && '<span class="child">↳</span>',
    blockers(q, t) && `←${blockers(q, t)}`,
    dependents(q, t) && `→${dependents(q, t)}`,
    t.note && '✎',
  ].filter(Boolean).join(' ');
  return [
    `<span class="ref-p">${project}</span><span class="ref-d">-</span><span class="ref-s">${suffix}</span>`,
    t.title,
    marks,
    `<span class="s-${t.status}">${STATUS[t.status]}</span>`,
    t.pri ? PRIORITY[t.pri] : '─',
  ];
}

export type Item = { key: string; text: string } | { key: string; cells: string[]; sel: boolean };

export function layout(q: Queue): Item[] {
  q.tasks.forEach((t) => { t.band = bandOf(q, t); });
  return BANDS.flatMap((b) => {
    const rows = q.tasks
      .filter((t) => t.band === b)
      .sort((a, c) => Number(c.status === 'active') - Number(a.status === 'active'));
    if (!rows.length) return [];
    return [
      { key: `band-${b}`, text: `▸ ${b.toUpperCase()} (${rows.length})` },
      ...rows.map((t) => ({ key: t.ref, cells: cells(q, t), sel: q.sel === t.ref })),
    ];
  });
}

export const CELL_CLASSES = ['', 't', 'mark', 'st', 'pri'];

/** Static markup for the queue rows, matching what the player renders. */
export function queueHTML(q: Queue) {
  return layout(q).map((item) => ('text' in item
    ? `<div class="q-sect" data-key="${item.key}">${item.text}</div>`
    : `<div class="q-row" data-key="${item.key}">${item.cells.map((c, i) => `<span class="${CELL_CLASSES[i]}">${c}</span>`).join('')}</div>`)).join('');
}

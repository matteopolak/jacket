// @ts-check
// The playground's panels (files, code, results) as a layout the reader can change: drag a panel's
// bar onto another panel's edge to dock it there, onto its middle to swap places, or onto the
// edge of the whole area to dock it there, and drag the gaps between panels to resize them.
// With no saved layout the page's own CSS arranges the panels.

/**
 * @typedef {{ kind: 'panel', id: string }} Leaf
 * @typedef {{ kind: 'split', dir: 'row' | 'col', items: { node: Node, size: number }[] }} Split
 * @typedef {Leaf | Split} Node
 * @typedef {'left' | 'right' | 'top' | 'bottom' | 'center'} Zone
 */

/** The smallest a panel gets when dragging a gap, in pixels. */
const MIN = 140;
/** How near the area's edge a drag has to be to dock against the whole edge, in pixels. */
const EDGE = 36;

/** @param {Node} node @returns {Node} */
const clone = (node) => JSON.parse(JSON.stringify(node));

/** The panels a tree holds, in order. @param {Node} node @returns {string[]} */
export function ids(node) {
  return node.kind === 'panel' ? [node.id] : node.items.flatMap((i) => ids(i.node));
}

/** The tree without a panel; splits left with one child become that child. @param {Node} node @param {string} id @returns {Node | null} */
export function remove(node, id) {
  if (node.kind === 'panel') return node.id === id ? null : node;
  const items = node.items
    .map((i) => ({ node: remove(i.node, id), size: i.size }))
    .filter(/** @returns {i is { node: Node, size: number }} */ (i) => i.node !== null);
  if (items.length === 0) return null;
  if (items.length === 1) return items[0].node;
  return { kind: 'split', dir: node.dir, items };
}

/**
 * The tree with `id` docked against `target`'s `zone`; `target` stays where it is for the middle.
 * @param {Node} tree @param {string} target @param {string} id @param {Zone} zone @returns {Node}
 */
export function dock(tree, target, id, zone) {
  if (zone === 'center') return swap(tree, target, id);
  const dir = zone === 'left' || zone === 'right' ? 'row' : 'col';
  const before = zone === 'left' || zone === 'top';
  /** @param {Node} node @returns {Node} */
  const walk = (node) => {
    if (node.kind === 'panel') {
      if (node.id !== target) return node;
      const mine = /** @type {Leaf} */ ({ kind: 'panel', id });
      const items = before
        ? [{ node: mine, size: 1 }, { node, size: 1 }]
        : [{ node, size: 1 }, { node: mine, size: 1 }];
      return { kind: 'split', dir, items };
    }
    const at = node.items.findIndex((i) => i.node.kind === 'panel' && i.node.id === target);
    if (at >= 0 && node.dir === dir) {
      const items = node.items.slice();
      const half = items[at].size / 2;
      items[at] = { node: items[at].node, size: half };
      items.splice(before ? at : at + 1, 0, { node: { kind: 'panel', id }, size: half });
      return { kind: 'split', dir, items };
    }
    return { kind: 'split', dir: node.dir, items: node.items.map((i) => ({ node: walk(i.node), size: i.size })) };
  };
  return walk(tree);
}

/** The tree with two panels' places exchanged. @param {Node} tree @param {string} a @param {string} b @returns {Node} */
export function swap(tree, a, b) {
  /** @param {Node} node @returns {Node} */
  const walk = (node) => {
    if (node.kind === 'panel') return node.id === a ? { kind: 'panel', id: b } : node.id === b ? { kind: 'panel', id: a } : node;
    return { kind: 'split', dir: node.dir, items: node.items.map((i) => ({ node: walk(i.node), size: i.size })) };
  };
  return walk(tree);
}

/** The tree with `id` docked against the whole area's edge. @param {Node} tree @param {string} id @param {Zone} edge @returns {Node} */
export function dockEdge(tree, id, edge) {
  const dir = edge === 'left' || edge === 'right' ? 'row' : 'col';
  const mine = /** @type {Leaf} */ ({ kind: 'panel', id });
  const items =
    edge === 'left' || edge === 'top'
      ? [{ node: mine, size: 0.28 }, { node: tree, size: 0.72 }]
      : [{ node: tree, size: 0.72 }, { node: mine, size: 0.28 }];
  return { kind: 'split', dir, items };
}

/**
 * Makes `body`'s panels rearrangeable.
 * @param {HTMLElement} body the panels' container
 * @param {Record<string, HTMLElement>} panels each panel by id, with its drag bar marked `data-dock-bar`
 * @param {{ main: HTMLElement, titles: Record<string, string>, around: (move: () => void) => void, changed: (tree: Node | null, free: boolean) => void }} hooks
 */
export function createLayout(body, panels, hooks) {
  /** @type {Node | null} */
  let tree = null;
  /** Whether a panel was docked to the area's edge, which lets the page use the whole window. */
  let free = false;
  /** The tree on screen: the reader's, or the page's own until they change it. @type {Node | null} */
  let shown = null;
  const narrow = matchMedia('(width < 720px)');
  const wide = matchMedia('(width >= 1180px)');

  /** The arrangement the page's CSS gives, as a tree to start changing from. @returns {Node} */
  function initial() {
    /** @param {string} id @returns {Leaf} */
    const leaf = (id) => ({ kind: 'panel', id });
    if (wide.matches)
      return { kind: 'split', dir: 'row', items: [{ node: leaf('files'), size: 0.18 }, { node: leaf('code'), size: 0.46 }, { node: leaf('results'), size: 0.36 }] };
    return {
      kind: 'split',
      dir: 'row',
      items: [
        { node: leaf('files'), size: 0.2 },
        { node: { kind: 'split', dir: 'col', items: [{ node: leaf('code'), size: 0.64 }, { node: leaf('results'), size: 0.36 }] }, size: 0.8 },
      ],
    };
  }

  /** @param {Node} node @returns {HTMLElement} */
  function build(node) {
    if (node.kind === 'panel') return panels[node.id];
    const el = document.createElement('div');
    el.className = 'pg-dock pg-' + node.dir;
    // Sizes are shares of the free space; a total under one would leave some of it empty.
    const sum = node.items.reduce((t, i) => t + i.size, 0) || 1;
    for (const item of node.items) item.size /= sum;
    node.items.forEach((item, n) => {
      const child = build(item.node);
      child.style.flex = `${item.size} 1 0`;
      if (n > 0) el.append(sash(node, n));
      el.append(child);
    });
    return el;
  }

  /** The gap between two neighbours, which resizes them when dragged. @param {Split} node @param {number} n */
  function sash(node, n) {
    const el = document.createElement('div');
    el.className = 'pg-sash pg-' + node.dir;
    el.setAttribute('role', 'separator');
    el.addEventListener('pointerdown', (e) => {
      e.preventDefault();
      el.setPointerCapture(e.pointerId);
      const a = /** @type {HTMLElement} */ (el.previousElementSibling);
      const b = /** @type {HTMLElement} */ (el.nextElementSibling);
      const row = node.dir === 'row';
      const sizeA = row ? a.getBoundingClientRect().width : a.getBoundingClientRect().height;
      const sizeB = row ? b.getBoundingClientRect().width : b.getBoundingClientRect().height;
      const start = row ? e.clientX : e.clientY;
      const total = node.items[n - 1].size + node.items[n].size;
      el.classList.add('dragging');
      /** @param {PointerEvent} m */
      const move = (m) => {
        const delta = (row ? m.clientX : m.clientY) - start;
        const next = Math.min(Math.max(sizeA + delta, Math.min(MIN, sizeA)), sizeA + sizeB - Math.min(MIN, sizeB));
        node.items[n - 1].size = (total * next) / (sizeA + sizeB);
        node.items[n].size = total - node.items[n - 1].size;
        a.style.flex = `${node.items[n - 1].size} 1 0`;
        b.style.flex = `${node.items[n].size} 1 0`;
      };
      const up = () => {
        el.classList.remove('dragging');
        el.removeEventListener('pointermove', move);
        el.removeEventListener('pointerup', up);
        el.removeEventListener('pointercancel', up);
        tree = shown;
        hooks.changed(tree, free);
      };
      el.addEventListener('pointermove', move);
      el.addEventListener('pointerup', up);
      el.addEventListener('pointercancel', up);
    });
    return el;
  }

  function render() {
    hooks.around(() => {
      for (const el of Object.values(panels)) el.style.removeProperty('flex');
      body.textContent = '';
      shown = tree ?? initial();
      body.classList.toggle('docked', !narrow.matches);
      if (!narrow.matches) body.append(build(shown));
      else {
        hooks.main.textContent = '';
        hooks.main.append(panels.code, panels.results);
        body.append(panels.files, hooks.main);
      }
    });
  }

  /** Slides each panel from where it was to where it is now. @param {Record<string, DOMRect>} before */
  function glide(before) {
    if (matchMedia('(prefers-reduced-motion: reduce)').matches) return;
    for (const [id, el] of Object.entries(panels)) {
      const was = before[id];
      const now = el.getBoundingClientRect();
      if (!was || !now.width || !now.height) continue;
      el.animate(
        [
          { transformOrigin: 'top left', transform: `translate(${was.left - now.left}px, ${was.top - now.top}px) scale(${was.width / now.width}, ${was.height / now.height})`, opacity: 0.7 },
          { transformOrigin: 'top left', transform: 'none', opacity: 1 },
        ],
        { duration: 200, easing: 'cubic-bezier(0.2, 0, 0, 1)' },
      );
    }
  }

  /** The panel under a point and where in it the point is, or null. @param {number} x @param {number} y @param {string} dragged @returns {{ id: string | null, zone: Zone, rect: DOMRect } | null} */
  function target(x, y, dragged) {
    const area = body.getBoundingClientRect();
    if (x < area.left || x > area.right || y < area.top || y > area.bottom) return null;
    const edges = /** @type {[Zone, number][]} */ ([
      ['left', x - area.left],
      ['right', area.right - x],
      ['top', y - area.top],
      ['bottom', area.bottom - y],
    ]).filter(([, d]) => d < EDGE).sort((p, q) => p[1] - q[1]);
    if (edges.length) {
      const zone = edges[0][0];
      const w = area.width * 0.28;
      const h = area.height * 0.28;
      const rect = new DOMRect(
        zone === 'right' ? area.right - w : area.left,
        zone === 'bottom' ? area.bottom - h : area.top,
        zone === 'left' || zone === 'right' ? w : area.width,
        zone === 'top' || zone === 'bottom' ? h : area.height,
      );
      return { id: null, zone, rect };
    }
    for (const [id, el] of Object.entries(panels)) {
      if (id === dragged) continue;
      const r = el.getBoundingClientRect();
      if (x < r.left || x > r.right || y < r.top || y > r.bottom) continue;
      const fx = (x - r.left) / r.width;
      const fy = (y - r.top) / r.height;
      /** @type {Zone} */
      const zone = fx < 0.25 ? 'left' : fx > 0.75 ? 'right' : fy < 0.25 ? 'top' : fy > 0.75 ? 'bottom' : 'center';
      const rect = new DOMRect(
        zone === 'right' ? r.left + r.width / 2 : r.left,
        zone === 'bottom' ? r.top + r.height / 2 : r.top,
        zone === 'left' || zone === 'right' ? r.width / 2 : r.width,
        zone === 'top' || zone === 'bottom' ? r.height / 2 : r.height,
      );
      return { id, zone, rect };
    }
    return null;
  }

  /** @param {string} id */
  function drag(id) {
    const bar = /** @type {HTMLElement} */ (panels[id].querySelector('[data-dock-bar]'));
    if (!bar) return;
    bar.classList.add('pg-grab');
    bar.addEventListener('pointerdown', (e) => {
      const hit = /** @type {HTMLElement} */ (e.target);
      if (e.button !== 0 || narrow.matches || hit.closest('button, input, select, a')) return;
      const x0 = e.clientX;
      const y0 = e.clientY;
      /** @type {HTMLElement | null} */
      let ghost = null;
      /** @type {HTMLElement | null} */
      let shade = null;
      /** @type {ReturnType<typeof target>} */
      let over = null;
      bar.setPointerCapture(e.pointerId);
      /** @param {PointerEvent} m */
      const move = (m) => {
        if (!ghost) {
          if (Math.hypot(m.clientX - x0, m.clientY - y0) < 6) return;
          ghost = document.createElement('div');
          ghost.className = 'pg-ghost';
          ghost.textContent = hooks.titles[id];
          shade = document.createElement('div');
          shade.className = 'pg-shade';
          document.body.append(ghost, shade);
        }
        ghost.style.left = m.clientX + 12 + 'px';
        ghost.style.top = m.clientY + 12 + 'px';
        over = target(m.clientX, m.clientY, id);
        const sh = /** @type {HTMLElement} */ (shade);
        sh.hidden = !over;
        if (over) {
          sh.style.left = over.rect.left + 'px';
          sh.style.top = over.rect.top + 'px';
          sh.style.width = over.rect.width + 'px';
          sh.style.height = over.rect.height + 'px';
        }
      };
      const up = () => {
        bar.removeEventListener('pointermove', move);
        bar.removeEventListener('pointerup', up);
        bar.removeEventListener('pointercancel', up);
        ghost?.remove();
        shade?.remove();
        if (!ghost || !over) return;
        const base = shown ?? initial();
        const rest = remove(base, id);
        if (!rest) return;
        if (over.id === null) free = true;
        tree = over.id === null ? dockEdge(rest, id, over.zone) : over.zone === 'center' ? swap(base, over.id, id) : dock(rest, over.id, id, over.zone);
        const before = Object.fromEntries(Object.entries(panels).map(([k, el]) => [k, el.getBoundingClientRect()]));
        render();
        glide(before);
        hooks.changed(tree, free);
      };
      bar.addEventListener('pointermove', move);
      bar.addEventListener('pointerup', up);
      bar.addEventListener('pointercancel', up);
    });
  }
  for (const id of Object.keys(panels)) drag(id);

  const again = () => render();
  narrow.addEventListener('change', again);
  wide.addEventListener('change', again);

  return {
    /** Shows a saved layout, or the page's own with `null`. @param {Node | null} saved @param {boolean} [wide] */
    apply(saved, wide = false) {
      tree = saved && ids(saved).sort().join() === Object.keys(panels).sort().join() ? clone(saved) : null;
      free = tree !== null && wide;
      render();
    },
    reset() {
      tree = null;
      free = false;
      render();
      hooks.changed(null, false);
    },
    destroy() {
      narrow.removeEventListener('change', again);
      wide.removeEventListener('change', again);
    },
  };
}

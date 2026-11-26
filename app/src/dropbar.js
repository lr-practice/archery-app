// dropbar.js — Active-Zones-Leiste am ganzen Mac (system.rs blendet sie ein,
// sobald irgendwo Dateien gezogen werden und ⌥ gehalten wird). Loslassen auf
// einer Zone sichert die Dateien dort: Dateien von außen werden importiert,
// Dateien aus der eigenen Bibliothek verschoben.
//
// 0.40.21: Die Karten stehen schon fertig im (unsichtbaren) Fenster, bevor
// gezogen wird. Beim Einblenden wird nichts mehr geladen; vorher kam die Leiste
// leer oder spät, und ein Drop auf die leere Leiste sicherte nichts.

(function () {
  const TAURI = window.__TAURI__;
  const invoke = (cmd, args) => TAURI.core.invoke(cmd, args);
  const { t, lang } = window.ArchiverI18n;
  const $ = (id) => document.getElementById(id);
  const HOT = 4;

  document.documentElement.lang = lang;

  const state = { snap: null, dirty: true, zones: [], target: null, visible: false };

  function el(tag, cls, text) {
    const node = document.createElement(tag);
    if (cls) node.className = cls;
    if (text != null) node.textContent = text;
    return node;
  }

  function pref(key, fallback) {
    try {
      const v = localStorage.getItem("archiver." + key);
      return v == null ? fallback : JSON.parse(v);
    } catch {
      return fallback;
    }
  }

  const zoneName = (path) => (path === "" ? (state.snap && state.snap.name) || "Archery" : path.split("/").join(" / "));

  // Reihenfolge wie in der App: die ersten vier sind die Active Zones
  function orderZones(zones) {
    zones = zones.filter((z) => z.count > 0 || !zones.some((o) => o.path.startsWith(z.path + "/")));
    const order = pref("zoneOrder", []);
    if (!order.length) return zones;
    const rank = (z) => {
      let i = order.indexOf(z.path);
      if (i < 0 && z.path) i = order.findIndex((p) => p.startsWith(z.path + "/"));
      return i < 0 ? order.length : i;
    };
    return zones.map((z, i) => [z, i]).sort((a, b) => rank(a[0]) - rank(b[0]) || a[1] - b[1]).map(([z]) => z);
  }

  async function load() {
    // Ecken wie in der App (Einstellungen › Darstellung), Werte aus palette.css
    const radius = pref("radius", "soft");
    if (radius && radius !== "round") document.documentElement.dataset.radius = radius;
    else delete document.documentElement.dataset.radius;
    if (!state.dirty && state.snap) return;
    try {
      state.snap = await invoke("snapshot");
    } catch {
      state.snap = null;
    }
    state.dirty = false;
    // Screenshots ist keine Zone und kein Ziel (0.48.0)
    state.zones = state.snap ? orderZones(state.snap.zones.filter((z) => z.path.split("/")[0] !== "Screenshots")).slice(0, HOT) : [];
  }

  function render() {
    const bar = $("bar");
    bar.classList.remove("leaving");
    bar.style.removeProperty("--leave-delay");
    bar.replaceChildren(
      ...state.zones.map((zone, i) => {
        const c = el("div", "db-hot");
        c.dataset.zone = zone.path;
        c.appendChild(el("span", "pal-mono pal-dim", String(i + 1).padStart(2, "0")));
        const text = el("div");
        text.appendChild(el("div", "db-name", zoneName(zone.path)));
        text.appendChild(el("div", "db-hint", t("hotRelease")));
        c.appendChild(text);
        return c;
      })
    );
  }

  // Karte unter dem Zeiger; in den Lücken und am Rand die Karte in derselben Spalte
  function cardAt(pos) {
    if (!pos) return null;
    // macOS meldet die Position schon in Punkten (wie app.js), nicht in Pixeln
    const x = pos.x;
    const hit = document.elementFromPoint(x, pos.y);
    const card = hit ? hit.closest(".db-hot") : null;
    if (card) return card;
    const cards = [...document.querySelectorAll(".db-hot")];
    let best = null;
    let dist = Infinity;
    for (const c of cards) {
      const r = c.getBoundingClientRect();
      const d = x < r.left ? r.left - x : x > r.right ? x - r.right : 0;
      if (d < dist) {
        dist = d;
        best = c;
      }
    }
    return best;
  }

  function mark(card) {
    for (const n of document.querySelectorAll(".drop-active")) n.classList.remove("drop-active");
    if (card) card.classList.add("drop-active");
    state.target = card;
  }

  const hold = (ms) => invoke("dropbar_hold", { ms }).catch(() => {});

  const FX = window.ArchiverDropFx;

  // Glas (0.40.24): echte macOS-Unschärfe hinter jeder Karte, wie das Glas
  // der App. Die Karten melden ihre Rechtecke, system.rs legt das Glas darunter.
  function glass() {
    const cards = [...document.querySelectorAll(".db-hot")].map((c) => {
      const r = c.getBoundingClientRect();
      return [r.left, r.top, r.width, r.height];
    });
    const first = document.querySelector(".db-hot");
    const radius = first ? parseFloat(getComputedStyle(first).borderTopLeftRadius) || 0 : 0;
    invoke("dropbar_glass", { cards, radius, light: document.documentElement.classList.contains("light") }).catch(() => {});
  }
  // nach dem Einblenden messen: das Fenster hat eben erst seine Höhe bekommen
  const glassSoon = () => requestAnimationFrame(() => requestAnimationFrame(glass));
  window.addEventListener("resize", () => state.visible && glassSoon());

  // Ausblenden: Webansicht und Glas zusammen (das Glas folgt keinem CSS)
  const FADE = 280;
  function fade(delay) {
    const bar = $("bar");
    bar.style.setProperty("--leave-delay", delay / 1000 + "s");
    bar.classList.add("leaving");
    setTimeout(() => invoke("dropbar_fade", { ms: FADE }).catch(() => {}), delay);
  }

  function leave(delay) {
    hold(delay + FADE + 100);
    fade(delay);
  }

  // „Gesichert“ erst, wenn es wirklich gesichert ist (0.31.0). Bis dahin bleibt
  // die Leiste stehen; ein Fehler bleibt lesbar stehen, bevor sie ausblendet.
  // Wie die Karte das Ankommen zeigt, regelt dropfx.js (Labor › Ablegen).
  async function drop(paths, card, pos) {
    mark(null);
    if (!card || !paths.length || !state.snap) return;
    const zone = card.dataset.zone;
    const hint = card.querySelector(".db-hint");
    const parts = { num: card.querySelector(".pal-mono"), name: card.querySelector(".db-name"), hint };
    hold(60000);
    card.classList.add("busy");
    hint.textContent = t("dropBusy");
    // Bild zuerst greifen: Eigene Dateien werden gleich verschoben
    const play = await FX.begin(card, parts, { paths, pos, invoke, src: TAURI.core.convertFileSrc });
    const root = state.snap.root + "/";
    const own = paths.filter((p) => p.startsWith(root)).map((p) => p.slice(root.length));
    const foreign = paths.filter((p) => !p.startsWith(root));
    try {
      if (own.length) await invoke("move_items", { paths: own, zone });
      if (foreign.length) await invoke("import_files", { paths: foreign, zone });
      card.classList.replace("busy", "landed");
      hint.textContent = t("dropSaved", { zone: zoneName(zone) });
      leave(await FX.land(play, zoneName(zone)));
    } catch (e) {
      invoke("log_error", { area: "dropbar", msg: String(e) }).catch(() => {});
      FX.fail(play);
      card.classList.replace("busy", "failed");
      hint.textContent = t("dropFailed", { msg: String(e) });
      hold(3000);
      fade(2500);
    }
  }

  TAURI.webview.getCurrentWebview().onDragDropEvent((event) => {
    const p = event.payload;
    if (p.type === "enter" || p.type === "over") return mark(cardAt(p.position));
    if (p.type === "leave") return mark(null);
    if (p.type === "drop") return drop(p.paths || [], cardAt(p.position) || state.target, p.position);
  });

  // Zonen im Hintergrund nachladen, solange die Leiste nicht zu sehen ist
  let refreshTimer = 0;
  function stale() {
    state.dirty = true;
    clearTimeout(refreshTimer);
    refreshTimer = setTimeout(() => {
      // steht die Leiste noch (Drop läuft, Bestätigung), später noch einmal
      if (state.visible || document.querySelector(".busy, .landed, .failed")) return stale();
      load().then(() => !state.visible && render());
    }, 400);
  }

  TAURI.event.listen("library-changed", stale);
  TAURI.event.listen("library-switched", stale);
  TAURI.event.listen("dropbar-show", () => {
    state.visible = true;
    // Stand von eben zeigen; nur wenn noch nie geladen wurde, jetzt laden
    // (render baut die Karten neu, ihr Einblenden läuft damit jedes Mal)
    if (!state.snap) load().then(render).then(glassSoon);
    else {
      render();
      glassSoon();
    }
  });
  TAURI.event.listen("dropbar-release", () => {
    // ohne Drop sofort weg
    if (!document.querySelector(".landed, .busy, .failed")) fade(0);
    // nach dem Ausblenden den neuen Stand vorbereiten (der Drop hat Zonen gefüllt)
    setTimeout(() => {
      state.visible = false;
      if (state.dirty) stale();
    }, 1500);
  });
  window.addEventListener("storage", (e) => {
    if (e.key === "archiver.zoneOrder" || e.key === "archiver.radius") stale();
  });

  load().then(render);
})();

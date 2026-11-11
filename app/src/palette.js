// palette.js — Palette am ganzen Mac (system.rs). Drei Zustände:
//   Start   Active Zones als Karten, darunter der Index (übrige Zonen)
//   Zone    Objekte einer Zone, oben die Active Zones als Reiter (1–4),
//           darüber ihre Unterordner als Karten (0.32)
//   Suche   passende Zonen als Karten, darunter passende Objekte
// Objekte zieht man heraus, z. B. in einen InDesign-Rahmen; währenddessen
// klappt die Palette auf eine Leiste oben zusammen und ist durchklickbar (0.37.2). ↵ kopiert, ⌘↵ zeigt im Finder.
// Leertaste zeigt das Objekt groß innerhalb der Palette, ⌘1–3 wechselt die Bibliothek.

(function () {
  const TAURI = window.__TAURI__;
  const invoke = (cmd, args) => TAURI.core.invoke(cmd, args);
  const fileSrc = (path) => TAURI.core.convertFileSrc(path);
  const { t, lang } = window.ArchiverI18n;
  const Media = window.ArchiverMedia;
  const $ = (id) => document.getElementById(id);

  document.documentElement.lang = lang;

  const HOT = 4;
  const SHOTS = "Screenshots"; // Ordner für Bildschirmfotos (0.47.0, library.rs SHOTS)
  const state = {
    snap: null,
    dirty: true,
    zones: [], // geordnet wie in der App
    view: "start", // start | zone | search
    zone: null,
    level: null, // Unterordner in der offenen Zone (0.32), null = ganze Zone
    sel: 0, // Index im aktuellen Fokusbereich
    area: "cards", // cards | objects (welcher Bereich die Pfeiltasten hat)
    cards: [], // sichtbare Karten (Zonen)
    objects: [], // sichtbare Objekte
    indesign: false, // aus InDesign geöffnet: ↵ und Doppelklick platzieren dort
    preview: null, // Objekt in der großen Ansicht (Leertaste)
    picked: new Set(), // 0.37.1: mehrere Objekte (⌘-Klick, ⇧-Klick), leer = nur das gewählte
  };

  // Textkarten wie in der App (app.js quoteTone): Grundfarbe, Schrift Ton in Ton
  const QUOTE_TONES = {
    red: ["#ff0000", "#5c0000", 0.7],
    green: ["#00b050", "#003d1c", 0.7],
    violet: ["#7030a0", "#e2c8f5", 0.75],
    cyan: ["#00b0f0", "#003d54", 0.7],
    yellow: ["#f2e14b", "#4d4500", 0.75],
    white: ["#ffffff", "#141414", 0.5],
    black: ["#141414", "#ededed", 0.5],
  };
  const RANDOM_TONES = ["red", "green", "violet", "cyan", "yellow"];
  function quoteTone(item) {
    if (QUOTE_TONES[item.tone]) return QUOTE_TONES[item.tone];
    if (item.format === "md") return QUOTE_TONES.white; // Notiz: Weiß
    const key = String(item.hash || item.path || "");
    let h = 0;
    for (let i = 0; i < key.length; i++) h = (h * 31 + key.charCodeAt(i)) >>> 0;
    return QUOTE_TONES[RANDOM_TONES[h % RANDOM_TONES.length]];
  }

  function textSheet(item, cls) {
    const node = el("div", "pal-text" + (cls ? " " + cls : ""));
    const [bg, fg] = quoteTone(item);
    node.style.setProperty("--ts-bg", bg);
    node.style.setProperty("--ts-fg", fg);
    // Notiz (.md): Titel ohne Markdown-Raute
    node.appendChild(el("div", "pal-text-body", item.format === "md" ? String(item.quote || "").replace(/^\s*#+\s*/, "") : item.quote));
    return node;
  }

  function el(tag, cls, text) {
    const node = document.createElement(tag);
    if (cls) node.className = cls;
    if (text != null) node.textContent = text;
    return node;
  }

  function mono(text, cls) {
    return el("span", "pal-mono" + (cls ? " " + cls : ""), text);
  }

  function pref(key, fallback) {
    try {
      const v = localStorage.getItem("archiver." + key);
      return v == null ? fallback : JSON.parse(v);
    } catch {
      return fallback;
    }
  }

  function toast(text) {
    const node = $("toast");
    node.textContent = text;
    node.classList.add("show");
    clearTimeout(toast.timer);
    toast.timer = setTimeout(() => node.classList.remove("show"), 1600);
  }

  const countText = (n) => (n === 1 ? t("countImage") : t("countImages", { n }));
  const zoneName = (path) => (path === "" ? (state.snap && state.snap.name) || "Archery" : path.split("/").join(" / "));

  // ---- Daten ----

  // Wie app.js: leere Elternordner ohne eigene Karte, Reihenfolge aus den Einstellungen
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
    if (!state.dirty && state.snap) return;
    try {
      state.snap = await invoke("snapshot");
    } catch {
      state.snap = null;
    }
    state.dirty = false;
    state.zones = state.snap ? orderZones(state.snap.zones) : [];
  }

  function itemsOf(zone) {
    const list = state.snap.items.filter((i) => i.zone === zone);
    const order = state.snap.order && state.snap.order[zone];
    if (!order || !order.length) return list;
    const pos = new Map(order.map((h, i) => [h, i]));
    const at = (item) => (pos.has(item.hash) ? pos.get(item.hash) : -1);
    return list.sort((a, b) => at(a) - at(b));
  }

  // Unterordner wie in der App (app.js subdirOf, zoneFolders): Ordnerpfad des
  // Objekts innerhalb seiner Zone, "" = direkt in der Zone
  function subdirOf(item) {
    const d = item.path.slice(0, Math.max(0, item.path.lastIndexOf("/")));
    const z = item.zone || "";
    if (!z) return d;
    return d === z ? "" : d.startsWith(z + "/") ? d.slice(z.length + 1) : "";
  }

  function inFolder(item, folder) {
    if (folder == null) return true;
    const d = subdirOf(item);
    return d === folder || d.startsWith(folder + "/");
  }

  // Ordner direkt unter `base` mit Anzahl (samt Unterordnern), auch leere
  function foldersOf(zone, items, base) {
    const pre = base ? base + "/" : "";
    const counts = new Map();
    for (const item of items) {
      const d = subdirOf(item);
      const rest = d.startsWith(pre) ? d.slice(pre.length) : "";
      if (!rest) continue; // lose Objekte dieser Ebene oder außerhalb
      const f = pre + rest.split("/")[0];
      counts.set(f, (counts.get(f) || 0) + 1);
    }
    const root = (zone ? zone + "/" : "") + pre;
    for (const d of state.snap.folders || []) {
      if (!d.startsWith(root) || d.length === root.length) continue;
      const f = pre + d.slice(root.length).split("/")[0];
      if (!counts.has(f)) counts.set(f, 0);
    }
    return [...counts].sort((a, b) => a[0].localeCompare(b[0], undefined, { numeric: true }));
  }

  function kindOf(item) {
    if (item.quote) return "text";
    return Media.kind(item.format || item.filename);
  }

  function thumbSrc(item) {
    if (item.thumb) return fileSrc(item.thumb);
    if (kindOf(item) === "audio") return Media.audioPlaceholder();
    if (kindOf(item) !== "image") return Media.placeholder((item.format || "").toUpperCase());
    return fileSrc(item.abs);
  }

  // Suche wie in der App (search.js): ganze Wörter, deutsche Wortformen,
  // sichere Motive; Treffer nach Relevanz
  const Find = window.ArchiverSearch;

  function search(q) {
    const words = Find.parse(q);
    const zones = state.zones.filter((z) => {
      const name = Find.tokens(zoneName(z.path));
      return words.every((w, i) => name.some((n) => n.startsWith(Find.stem(w)) || (i === words.length - 1 && n.startsWith(w))));
    });
    const items = state.snap.items
      .map((i) => [Find.score(i, words), i])
      .filter(([s]) => s > 0)
      .sort((a, b) => b[0] - a[0])
      .map(([, i]) => i);
    return { zones, items };
  }

  // ---- Darstellung ----

  function cover(items) {
    items = items.filter((it) => kindOf(it) !== "text");
    const trio = items.length >= 3;
    const node = el("div", "pal-cover" + (trio ? " trio" : ""));
    for (const item of items.slice(0, trio ? 3 : 1)) {
      const img = el("img");
      img.src = thumbSrc(item);
      img.loading = "lazy";
      img.draggable = false;
      node.appendChild(img);
    }
    return node;
  }

  function card(zone, i) {
    const node = el("article", "pal-card");
    node.appendChild(cover(itemsOf(zone.path)));
    if (i < HOT) {
      const key = el("div", "pal-key");
      key.appendChild(mono(String(i + 1).padStart(2, "0")));
      key.appendChild(mono(t("palActive")));
      node.appendChild(key);
    }
    const body = el("div", "pal-card-body");
    body.appendChild(el("div", "pal-card-name", zoneName(zone.path)));
    body.appendChild(el("div", "pal-card-meta", zone.count ? countText(zone.count) : t("emptyShort")));
    node.appendChild(body);
    node.addEventListener("click", () => openZone(zone.path));
    return node;
  }

  // Unterordner als Karte wie in der App (4.5): Cover, Name, Anzahl
  function folderCard(path, n) {
    const node = el("article", "pal-card");
    node.appendChild(cover(itemsOf(state.zone).filter((it) => inFolder(it, path))));
    const body = el("div", "pal-card-body");
    body.appendChild(el("div", "pal-card-name", path.split("/").pop()));
    body.appendChild(el("div", "pal-card-meta", n ? countText(n) : t("emptyShort")));
    node.appendChild(body);
    node.addEventListener("click", () => openFolder(path));
    return node;
  }

  function obj(item) {
    const node = el("div", "pal-obj");
    if (kindOf(item) === "text") node.appendChild(textSheet(item));
    else {
      const img = el("img");
      img.src = thumbSrc(item);
      img.loading = "lazy";
      img.draggable = false;
      node.appendChild(img);
    }
    node.title = item.filename;
    node.addEventListener("mousedown", (e) => pressObject(e, item));
    node.addEventListener("dblclick", () => placeItem(item));
    return node;
  }

  function label(text) {
    return mono(text, "pal-label");
  }

  function keys(list) {
    // 0.28: links der Name „Tray“, die Tasten rechts in einer Reihe
    const foot = $("keys");
    const cmds = el("span", "pal-keys-list");
    for (const [k, text] of list) {
      const s = el("span", "pal-mono");
      s.appendChild(el("span", "pal-kbd", k));
      s.appendChild(document.createTextNode(text));
      cmds.appendChild(s);
    }
    const esc = el("span", "pal-mono");
    esc.appendChild(el("span", "pal-kbd", "esc"));
    cmds.appendChild(esc);
    foot.replaceChildren(el("span", "pal-mono pal-name", "Tray"), cmds);
  }

  function render() {
    const body = $("body");
    body.replaceChildren();
    state.cards = [];
    state.objects = [];
    $("crumb").hidden = state.view !== "zone";
    $("hits").textContent = "";
    // 0.47.0: Knopf „Screenshots“, solange es den Ordner gibt (app.js hasShots)
    const shots = state.zones.find((z) => z.path === SHOTS);
    $("shots").hidden = !shots;
    $("shots").classList.toggle("on", state.view === "zone" && state.zone === SHOTS);
    if (shots) $("shots").lastElementChild.textContent = shots.count;

    if (!state.snap) {
      body.appendChild(el("div", "pal-empty", t("palNoLibrary")));
      keys([]);
      return;
    }

    if (state.view === "start") {
      const hot = state.zones.slice(0, HOT);
      const rest = state.zones.slice(HOT);
      body.appendChild(label("(" + t("hotLabel") + ") (" + hot.length + ")"));
      const row = el("div", "pal-cards");
      hot.forEach((z, i) => row.appendChild(card(z, i)));
      body.appendChild(row);
      if (rest.length) {
        body.appendChild(label(t("indexLabel") + " (" + rest.length + ")"));
        const more = el("div", "pal-cards");
        rest.forEach((z, i) => more.appendChild(card(z, HOT + i)));
        body.appendChild(more);
      }
      state.cards = state.zones.map((z) => z.path);
      state.area = "cards";
      $("hits").textContent = "⌥ SPACE";
      keys([["←→", t("palZone")], ["↵", t("palOpen")], ["1–4", t("palActive")], ["⌘1–3", t("palLibrary")]]);
    }

    if (state.view === "zone") {
      renderCrumb();
      const all = itemsOf(state.zone);
      const items = all.filter((it) => inFolder(it, state.level));
      const folders = foldersOf(state.zone, all, state.level);
      if (folders.length) {
        body.appendChild(label("(" + t("palFolders") + ") (" + folders.length + ")"));
        const row = el("div", "pal-cards");
        for (const [f, n] of folders) row.appendChild(folderCard(f, n));
        body.appendChild(row);
        if (items.length) body.appendChild(label("(" + t("palObjects") + ") (" + items.length + ")"));
      }
      if (!items.length && !folders.length) body.appendChild(el("div", "pal-empty", t("palZoneEmpty")));
      const grid = el("div", "pal-grid");
      for (const item of items) grid.appendChild(obj(item));
      body.appendChild(grid);
      state.cards = folders.map(([f]) => f);
      state.objects = items;
      if (state.area === "objects" && !items.length) state.area = "cards";
      if (state.area === "cards" && !state.cards.length) state.area = "objects";
      keys([["↵", enterLabel()], ["␣", t("palPreview")], ["⌘↵", t("palFinder")], ["1–4", t("palZone")], ["⌫", t("palBack")]]);
    }

    if (state.view === "search") {
      const { zones, items } = search($("q").value);
      $("hits").textContent = zones.length
        ? t("palHitsZones", { n: items.length, z: zones.length })
        : t("palHits", { n: items.length });
      if (zones.length) {
        body.appendChild(label("(" + t("palZones") + ")"));
        const row = el("div", "pal-cards");
        for (const z of zones.slice(0, 8)) row.appendChild(card(z, state.zones.indexOf(z)));
        body.appendChild(row);
      }
      if (items.length) {
        body.appendChild(label("(" + t("palObjects") + ")"));
        const grid = el("div", "pal-grid");
        for (const item of items.slice(0, 120)) grid.appendChild(obj(item));
        body.appendChild(grid);
      }
      if (!zones.length && !items.length) body.appendChild(el("div", "pal-empty", t("palEmpty")));
      state.cards = zones.slice(0, 8).map((z) => z.path);
      state.objects = items.slice(0, 120);
      if (state.area === "cards" && !state.cards.length) state.area = "objects";
      if (state.area === "objects" && !state.objects.length) state.area = "cards";
      keys([["↵", enterLabel()], ["⌘↵", t("palFinder")], ["tab", t("palSwitch")]]);
    }
    markSel();
  }

  function renderCrumb() {
    const crumb = $("crumb");
    crumb.replaceChildren();
    const head = el("div", "pal-crumb-head");
    const back = mono("← " + t("palZones"), "pal-crumb-back");
    back.addEventListener("click", goStart);
    head.appendChild(back);
    const name = el("span", "pal-crumb-name", zoneName(state.zone));
    // im Unterordner: „Zone / Ordner“, der Zonenname führt zurück auf die Zone
    if (state.level != null) {
      name.classList.add("pal-crumb-up");
      name.addEventListener("click", () => openFolder(null));
      head.appendChild(name);
      head.appendChild(el("span", "pal-crumb-name", "/ " + state.level.split("/").join(" / ")));
    } else head.appendChild(name);
    const zone = state.zones.find((z) => z.path === state.zone);
    const i = state.zones.indexOf(zone);
    const n = state.level != null ? itemsOf(state.zone).filter((it) => inFolder(it, state.level)).length : zone && zone.count;
    let meta = zone ? countText(n) : "";
    if (i >= 0 && i < HOT) meta += " · " + t("palActive") + " " + String(i + 1).padStart(2, "0");
    head.appendChild(el("span", "pal-crumb-meta", meta));
    crumb.appendChild(head);
    const tabs = el("div", "pal-tabs");
    state.zones.slice(0, HOT).forEach((z, n) => {
      const tab = el("div", "pal-tab" + (z.path === state.zone ? " on" : ""));
      tab.appendChild(mono(String(n + 1).padStart(2, "0")));
      tab.appendChild(document.createTextNode(zoneName(z.path)));
      tab.addEventListener("click", () => openZone(z.path));
      tabs.appendChild(tab);
    });
    crumb.appendChild(tabs);
  }

  function markSel() {
    for (const n of document.querySelectorAll(".sel")) n.classList.remove("sel");
    const nodes = document.querySelectorAll(state.area === "cards" ? ".pal-card" : ".pal-obj");
    if (!nodes.length) return;
    state.sel = Math.max(0, Math.min(state.sel, nodes.length - 1));
    // Mehrfachauswahl: jedes gewählte Objekt trägt den Ring (DESIGN.md 4.1)
    if (state.area === "objects" && state.picked.size)
      state.objects.forEach((item, i) => state.picked.has(item) && nodes[i] && nodes[i].classList.add("sel"));
    const node = nodes[state.sel];
    node.classList.add("sel");
    node.scrollIntoView({ block: "nearest" });
  }

  // Gewählte Objekte in Reihenfolge des Rasters; ohne Mehrfachauswahl nur das eine
  function picks(item) {
    if (state.picked.size > 1 && (!item || state.picked.has(item))) return state.objects.filter((i) => state.picked.has(i));
    return item ? [item] : [];
  }

  function clearPicked() {
    if (!state.picked.size) return false;
    state.picked.clear();
    return true;
  }

  // ---- Wechsel ----

  function goStart() {
    state.view = "start";
    state.zone = null;
    state.level = null;
    state.sel = 0;
    $("q").value = "";
    render();
    $("q").focus();
  }

  function openZone(path) {
    state.view = "zone";
    state.zone = path;
    state.level = null;
    state.sel = 0;
    state.area = "objects";
    $("q").value = "";
    render();
    $("q").focus();
  }

  function openFolder(path) {
    state.level = path;
    state.sel = 0;
    state.area = "objects";
    render();
    $("q").focus();
  }

  // ⌫ im Ordner: eine Ebene hoch, in der Zone: zurück zur Übersicht
  function goUp() {
    if (state.level == null) return goStart();
    const i = state.level.lastIndexOf("/");
    openFolder(i > 0 ? state.level.slice(0, i) : null);
  }

  function onInput() {
    const q = $("q").value.trim();
    if (!q) return goStart();
    state.view = "search";
    state.sel = 0;
    state.area = "cards";
    render();
  }

  // Spalten im aktuellen Bereich (für ↑↓)
  function columns() {
    if (state.area === "objects") return 6;
    return 4;
  }

  function move(dx, dy) {
    const total = state.area === "cards" ? state.cards.length : state.objects.length;
    if (!total) return;
    let next = state.sel + dx + dy * columns();
    if (state.view !== "start" && dy) {
      // ↓ aus den Karten (Zonen, Ordner) in die Objekte, ↑ zurück
      if (next >= total && state.area === "cards" && state.objects.length) {
        state.area = "objects";
        state.sel = 0;
        return markSel();
      }
      if (next < 0 && state.area === "objects" && state.cards.length) {
        state.area = "cards";
        state.sel = 0;
        return markSel();
      }
    }
    state.sel = Math.max(0, Math.min(total - 1, next));
    markSel();
  }

  function selected() {
    if (state.area === "cards") return { zone: state.cards[state.sel] };
    return { item: state.objects[state.sel] };
  }

  const enterLabel = () => (state.indesign ? t("palPlace") : t("palCopy"));

  // Aus InDesign: in den markierten Rahmen, sonst in den Platzieren-Cursor.
  // Sonst (oder wenn InDesign nicht mitspielt): kopieren.
  async function placeItem(item) {
    if (!state.indesign) return copyItem(item);
    try {
      const r = await invoke("palette_place", { path: item.abs });
      if (r === "none") return copyItem(item);
    } catch (e) {
      toast(String(e) === "indesign-denied" ? t("palDenied") : String(e));
    }
  }

  async function copyItem(item) {
    const list = picks(item);
    try {
      await invoke("copy_images", { paths: list.map((i) => i.abs), text: list.length === 1 ? item.quote || null : null });
      toast(t("palCopied"));
    } catch (e) {
      toast(String(e));
    }
  }

  async function pasteHere() {
    const zone = state.level ? state.zone + "/" + state.level : state.zone;
    try {
      const r = await invoke("paste_into", { zone });
      toast(r.imported ? t("pasted", { zone: zoneName(zone) }) : r.duplicates ? t("importDone", { imported: 0, duplicates: r.duplicates, sources: 0 }) : t("pasteEmpty"));
      state.dirty = true;
      await load();
      render();
    } catch (e) {
      toast(String(e));
    }
  }

  function hide() {
    invoke("palette_hide").catch(() => {});
  }

  // ---- Große Ansicht (Leertaste), innerhalb der Palette ----

  function openPreview(item) {
    closePreview();
    state.preview = item;
    const layer = el("div", "pal-preview");
    const stage = el("div", "pal-preview-stage");
    const kind = kindOf(item);
    if (kind === "text") stage.appendChild(textSheet(item, "large"));
    else if (kind === "video") {
      const v = el("video");
      v.src = fileSrc(item.abs);
      v.muted = true;
      v.loop = true;
      v.autoplay = true;
      v.playsInline = true;
      v.poster = item.thumb ? fileSrc(item.thumb) : "";
      stage.appendChild(v);
    } else {
      const img = el("img");
      img.draggable = false;
      // Bilder im Original, alles andere (PDF, Audio) als Vorschaubild
      img.src = kind === "image" ? fileSrc(item.abs) : thumbSrc(item);
      img.onerror = () => {
        img.onerror = null;
        img.src = thumbSrc(item);
      };
      img.addEventListener("mousedown", (e) => pressObject(e, item));
      stage.appendChild(img);
    }
    layer.appendChild(stage);
    const meta = el("div", "pal-preview-meta");
    meta.appendChild(el("span", "pal-preview-name", item.filename));
    const i = state.objects.indexOf(item);
    meta.appendChild(mono(i >= 0 ? i + 1 + " / " + state.objects.length : "", "pal-dim"));
    layer.appendChild(meta);
    layer.addEventListener("click", (e) => {
      if (e.target === layer || e.target === stage) closePreview();
    });
    const pal = document.querySelector(".pal");
    pal.insertBefore(layer, $("keys"));
    pal.classList.add("previewing");
    keys([["←→", t("palStep")], ["↵", enterLabel()], ["␣", t("palClose")]]);
  }

  function closePreview() {
    const layer = document.querySelector(".pal-preview");
    if (!layer) return false;
    const v = layer.querySelector("video");
    if (v) {
      v.pause();
      v.removeAttribute("src");
      v.load();
    }
    layer.remove();
    document.querySelector(".pal").classList.remove("previewing");
    state.preview = null;
    render();
    return true;
  }

  function previewKey(e) {
    if (e.key === " " || e.key === "Escape") {
      e.preventDefault();
      return closePreview();
    }
    if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
      e.preventDefault();
      const list = state.objects;
      const i = list.indexOf(state.preview);
      const next = list[Math.max(0, Math.min(list.length - 1, i + (e.key === "ArrowRight" ? 1 : -1)))];
      if (next && next !== state.preview) {
        state.sel = list.indexOf(next);
        openPreview(next);
      }
      return;
    }
    if (e.key === "Enter") {
      e.preventDefault();
      if (e.metaKey) return invoke("reveal", { path: state.preview.abs }).then(hide).catch(() => {});
      return placeItem(state.preview);
    }
    e.preventDefault();
  }

  // ⌘1–3: Bibliothek wechseln, wie in der App
  async function switchLibrary(i) {
    try {
      const slots = await invoke("libraries");
      const slot = slots[i];
      if (!slot || !slot.path) return toast(t("palNoSlot", { n: i + 1 }));
      if (!slot.active) {
        await invoke("open_library", { path: slot.path });
        state.dirty = true;
        await load();
      }
      goStart();
      toast(slot.name || slot.path.split("/").pop());
    } catch (e) {
      toast(String(e));
    }
  }

  function onKey(e) {
    if (state.preview) return previewKey(e);
    const q = $("q").value;
    // 0.32: ⌘C kopiert das gewählte Objekt, ⌘V legt die Zwischenablage in die
    // offene Zone. Steht etwas im Suchfeld, gehören beide dem Suchfeld.
    if (e.metaKey && !e.altKey && !e.shiftKey && !q && (e.key === "c" || e.key === "v")) {
      const s = selected();
      if (e.key === "c" && state.area === "objects" && s.item) {
        e.preventDefault();
        return copyItem(s.item);
      }
      if (e.key === "v" && state.view === "zone") {
        e.preventDefault();
        return pasteHere();
      }
    }
    if (e.metaKey && !e.altKey && !e.shiftKey && /^[1-4]$/.test(e.key)) {
      e.preventDefault();
      return switchLibrary(Number(e.key) - 1);
    }
    // Leertaste: großes Bild des gewählten Objekts (solange im Suchfeld nichts getippt ist)
    if (e.key === " " && !q && state.area === "objects") {
      const s = selected();
      if (s.item) {
        e.preventDefault();
        return openPreview(s.item);
      }
    }
    if (e.key === "Escape") {
      e.preventDefault();
      if (clearPicked()) return markSel();
      if (state.view !== "start") return goStart();
      return hide();
    }
    if (e.key === "Backspace" && !q && state.view === "zone") {
      e.preventDefault();
      return goUp();
    }
    // 1–4: Active Zone öffnen, solange nichts getippt ist
    if (!q && /^[1-4]$/.test(e.key) && !e.metaKey && !e.altKey) {
      const zone = state.zones[Number(e.key) - 1];
      if (zone) {
        e.preventDefault();
        return openZone(zone.path);
      }
    }
    if (e.key === "Tab" && state.view !== "start") {
      e.preventDefault();
      const other = state.area === "cards" ? "objects" : "cards";
      if ((other === "cards" ? state.cards : state.objects).length) {
        state.area = other;
        state.sel = 0;
        markSel();
      }
      return;
    }
    const arrows = { ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1] };
    if (arrows[e.key]) {
      // ←→ im Suchfeld bewegen den Cursor, solange Text drin ist
      if (q && (e.key === "ArrowLeft" || e.key === "ArrowRight")) return;
      e.preventDefault();
      // ⇧ + Pfeil nimmt die Objekte unterwegs dazu, sonst endet die Mehrfachauswahl
      const from = state.area === "objects" ? state.objects[state.sel] : null;
      if (e.shiftKey && from) state.picked.add(from);
      else clearPicked();
      move(...arrows[e.key]);
      if (e.shiftKey && state.area === "objects" && state.objects[state.sel]) {
        state.picked.add(state.objects[state.sel]);
        markSel();
      }
      return;
    }
    if (e.key === "Enter") {
      e.preventDefault();
      const s = selected();
      if (s.zone != null) return state.view === "zone" ? openFolder(s.zone) : openZone(s.zone);
      if (!s.item) return;
      if (e.metaKey) return invoke("reveal", { path: s.item.abs }).then(hide).catch(() => {});
      return placeItem(s.item);
    }
  }

  // ---- Herausziehen ----
  // Wie in der App über das Drag-Plugin (echte Datei, InDesign verknüpft sie).
  // Ab dem Losziehen ist die Palette fast unsichtbar und durchklickbar.

  function pressObject(e, item) {
    if (e.button !== 0) return;
    const index = state.objects.indexOf(item);
    // 0.37.1: ⌘-Klick nimmt dazu oder heraus, ⇧-Klick wählt bis hierher
    if (index >= 0 && (e.metaKey || e.shiftKey)) {
      const current = state.area === "objects" ? state.objects[state.sel] : null;
      if (e.shiftKey && current) {
        const [a, b] = [state.sel, index].sort((x, y) => x - y);
        state.objects.slice(a, b + 1).forEach((i) => state.picked.add(i));
      } else {
        if (!state.picked.size && current) state.picked.add(current);
        if (state.picked.has(item) && state.picked.size > 1) state.picked.delete(item);
        else state.picked.add(item);
      }
      state.area = "objects";
      state.sel = index;
      markSel();
      return;
    }
    // Klick auf ein Objekt der Auswahl hält sie fürs Ziehen; ohne Ziehen bleibt nur dieses
    const keep = state.picked.has(item) && state.picked.size > 1;
    if (index >= 0) {
      if (!keep) clearPicked();
      state.area = "objects";
      state.sel = index;
      markSel();
    }
    const x0 = e.clientX;
    const y0 = e.clientY;
    let dragged = false;
    const onMove = (m) => {
      if (Math.hypot(m.clientX - x0, m.clientY - y0) < 4) return;
      dragged = true;
      done();
      startDrag(item);
    };
    const done = () => {
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", done);
      if (!dragged && keep && clearPicked()) markSel();
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", done);
  }

  // Leiste beim Ziehen (V4): Objekt, Name, Zone, Hinweis. Liegt im Suchfeld,
  // auf das der Tray nach oben zusammenklappt (macos.rs panel_ghost).
  const DRAG_BAR = 72; // Suchfeld 44 + 14 oben + 14 unten
  function showDragBar(list) {
    const first = list[0];
    const bar = el("div", "pal-dragbar");
    bar.appendChild(el("span", "pal-mono pal-dim", "Tray"));
    const thumb = el("span", "pal-dragbar-thumb");
    const img = el("img");
    img.src = thumbSrc(first);
    img.draggable = false;
    thumb.appendChild(img);
    bar.appendChild(thumb);
    bar.appendChild(el("b", "pal-dragbar-name", list.length > 1 ? countText(list.length) : first.filename));
    if (list.length === 1) bar.appendChild(el("span", "pal-dim pal-dragbar-zone", zoneName(first.zone || "")));
    const hint = el("span", "pal-dragbar-hint pal-mono pal-dim", t(state.indesign ? "palDragPlace" : "palDragDrop"));
    const esc = el("span", "pal-mono pal-dim");
    esc.append(el("span", "pal-kbd", "esc"), document.createTextNode(t("palDragCancel")));
    bar.append(hint, esc);
    document.querySelector(".pal-search").appendChild(bar);
    document.body.classList.add("dragging");
  }
  function hideDragBar() {
    document.body.classList.remove("dragging");
    document.querySelectorAll(".pal-dragbar").forEach((b) => b.remove());
  }
  const ghost = (on) => {
    if (on) showDragBar(ghost.list);
    else hideDragBar();
    return invoke("palette_ghost", { on, bar: DRAG_BAR }).catch(() => {});
  };

  async function startDrag(item) {
    const channel = new TAURI.core.Channel();
    channel.onmessage = () => ghost(false);
    const icon =
      (await invoke("drag_icon", { hash: item.hash, path: item.thumb || item.abs, color: null }).catch(() => null)) ||
      item.thumb ||
      item.abs;
    const list = picks(item);
    ghost.list = list;
    ghost(true);
    invoke("plugin:drag|start_drag", { item: list.map((i) => i.abs), image: icon, onEvent: channel }).catch(() => ghost(false));
  }

  // ---- Start ----

  $("q").placeholder = t("palSearch");
  $("q").addEventListener("input", onInput);
  document.addEventListener("keydown", onKey);
  $("shots").addEventListener("click", () => (state.view === "zone" && state.zone === SHOTS ? goStart() : openZone(SHOTS)));

  TAURI.event.listen("library-changed", () => (state.dirty = true));
  TAURI.event.listen("library-switched", () => (state.dirty = true));
  TAURI.event.listen("palette-open", async (e) => {
    state.indesign = e && e.payload === true;
    state.preview = null;
    applyLook();
    await load();
    goStart();
  });
  window.addEventListener("storage", (e) => {
    if (e.key === "archiver.zoneOrder") state.dirty = true;
    if (e.key === "archiver.radius" || e.key === "archiver.theme") applyLook();
  });

  // 0.28: Aussehen aus der App (Hell/Dunkel, Ecken) gilt auch im Tray, samt Fensterecke
  const TRAY_RADIUS = { square: 0, soft: 6, round: 12, xl: 18 };
  function applyLook() {
    const id = pref("radius", "soft");
    const root = document.documentElement;
    if (id && id !== "round") root.dataset.radius = id;
    else delete root.dataset.radius;
    const r = TRAY_RADIUS[id] ?? 12;
    root.style.setProperty("--pal-radius", r + "px");
    window.ArchiverI18n.applyTheme();
    const light = root.classList.contains("light");
    invoke("set_appearance", { light, radius: r }).catch(() => {});
  }
  applyLook();

  load().then(goStart);
})();

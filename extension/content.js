// content.js
// Zeigt beim Ziehen eines Bildes eine frei konfigurierbare Anzahl Ablage-Zonen am
// unteren Bildschirmrand an. Eine Zone namens "Downloads" (Groß-/Kleinschreibung
// egal) speichert ohne Unterordner direkt im normalen Downloads-Ordner, alle
// anderen Zonennamen werden als (Unter-)Ordnerpfad verwendet. Ermittelt beim Start
// des Drags nach Möglichkeit die höchstaufgelöste Bildquelle statt der sichtbaren
// Thumbnail-URL und zeigt nach dem Download eine kurze Text-Bestätigung.
//
// Per Tastenkombination (eigene oder Chrome-Befehl) öffnet sich das Overlay:
// dieselbe Ansicht wie die Vollansicht (ui.js), nur halbtransparent über der
// Seite. Die Zonenleiste gibt es nur während eines Drags. Alles Sichtbare lebt
// in einem eigenen Shadow DOM, damit Seiten-CSS und unsere Styles sich nicht
// gegenseitig beeinflussen.

(function () {
  const { DEFAULT_ZONES, DEFAULT_SHORTCUT, isRootZoneName, loadZones, debugLog, currentPageSource, extractDroppedImage, t } =
    ArchiverShared;
  const UI = ArchiverUI;

  let zones = DEFAULT_ZONES.slice();
  // Leiste beim Ziehen: mit App die vier Hot Zones aus der Übersicht, sonst alle Zonen
  let barZones = zones;
  // Nach einem Update der Extension läuft dieses Skript in schon offenen Tabs
  // verwaist weiter (ohne chrome.*). Dann hält es still, statt eine alte
  // Leiste ohne Texte zu zeigen; die neue Fassung übernimmt.
  const gone = () => {
    try {
      return !chrome.runtime || !chrome.runtime.id;
    } catch {
      return true;
    }
  };
  const hotOf = (list, info) => (info && Array.isArray(info.hot) && info.hot.length ? info.hot.slice() : list);
  let shortcut = DEFAULT_SHORTCUT;
  let dragCounter = 0;
  let zonesOn = false; // Zonen für dieses Ziehen eingeblendet (3.31.3)
  let manualOpen = false; // true, wenn per Tastenkombination geöffnet (nicht durch Drag)
  let recordingShortcut = false; // true, während in den Einstellungen eine neue Kombination aufgenommen wird
  let fallbackImageUrl = null; // beim dragstart ermittelte (möglichst hochauflösende) Bildquelle
  let dragCaption = null; // Bildtext zum gezogenen Bild (0.29), siehe captionNear

  // ---- Shadow-DOM-Host ----
  // Wird beim ersten Bedarf angelegt (erster Drag, Tastenkombination oder
  // Bestätigung). Die Styles werden einmal geladen und als Constructable
  // Stylesheet übernommen; das umgeht auch strenge CSP-Regeln der Seite.

  let overlayReady = null;
  let shadow = null;
  let rootEl = null;

  let zonesBarEl = null;
  let zoneEls = [];
  let galleryPanelEl = null;
  let gallery = null;
  let panelHostEl = null;
  let panels = null;

  // Melange und Geist wie in der App. @font-face greift im Shadow DOM nicht, deshalb über
  // document.fonts; die Seite sieht nur die zusätzliche Schrift, ändert nichts.
  let fontsLoaded = false;
  function loadFonts() {
    if (fontsLoaded || !window.FontFace || !document.fonts) return;
    fontsLoaded = true;
    [
      ["Geist", "fonts/Geist-Variable.woff2", { weight: "100 900" }],
      ["Geist Mono", "fonts/GeistMono-Regular.woff2", { weight: "400" }],
      ["KMR Melange Grotesk", "fonts/melange/Melange-100.woff2", { weight: "100" }],
      ["KMR Melange Grotesk", "fonts/melange/Melange-300.woff2", { weight: "300" }],
      ["KMR Melange Grotesk", "fonts/melange/Melange-400.woff2", { weight: "400" }],
      ["KMR Melange Grotesk", "fonts/melange/Melange-400i.woff2", { weight: "400", style: "italic" }],
      ["KMR Melange Grotesk", "fonts/melange/Melange-500.woff2", { weight: "500" }],
      ["KMR Melange Grotesk", "fonts/melange/Melange-600.woff2", { weight: "600" }],
      ["KMR Melange Grotesk", "fonts/melange/Melange-700.woff2", { weight: "700" }],
    ].forEach(([family, file, desc]) => {
      const face = new FontFace(family, `url(${chrome.runtime.getURL(file)})`, desc);
      face.load().then((f) => document.fonts.add(f)).catch(() => {});
    });
  }

  // Ein-/Ausblenden der Zonenleiste als animierbare Zahl (siehe overlay.css)
  try {
    CSS.registerProperty({ name: "--archiver-zone-in", syntax: "<number>", inherits: true, initialValue: "1" });
  } catch (e) {
    // schon registriert (zweite Injektion) oder nicht unterstützt: Fallback 1 im CSS
  }

  // Styles vorab holen (siehe Ende der Datei), das Overlay selbst erst beim
  // ersten Gebrauch anlegen: die Seite bekommt bis dahin kein fremdes Element.
  let stylesReady = null;
  const loadStyles = () =>
    (stylesReady ||= Promise.all(
      ["gallery.css", "panel.css", "overlay.css"].map((file) => fetch(chrome.runtime.getURL(file)).then((r) => r.text()))
    ).catch((e) => {
      stylesReady = null; // beim nächsten Mal neu versuchen
      throw e;
    }));

  function ensureOverlay() {
    if (!overlayReady) {
      overlayReady = loadStyles().then((texts) => {
        const host = document.createElement("archiver-overlay");
        ["position:fixed", "top:0", "left:0", "width:0", "height:0", "display:block", "z-index:2147483647"].forEach((rule) => {
          const [prop, value] = rule.split(":");
          host.style.setProperty(prop, value, "important");
        });
        shadow = host.attachShadow({ mode: "closed" }); // geschlossen: die Seite kommt nicht an Vorschauen und Dateinamen
        UI.watchTheme(host);
        const sheet = new CSSStyleSheet();
        sheet.replaceSync(texts.join("\n"));
        shadow.adoptedStyleSheets = [sheet];
        rootEl = UI.el("div", "root");
        shadow.appendChild(rootEl);
        document.documentElement.appendChild(host);
        loadFonts();
      });
    }
    return overlayReady;
  }

  // ---- Einstellungen aus dem Speicher ----

  function loadShortcut(cb) {
    if (!chrome.storage || !chrome.storage.sync) {
      cb(DEFAULT_SHORTCUT);
      return;
    }
    chrome.storage.sync.get({ shortcut: null }, (data) => {
      const s = data.shortcut;
      cb(s && s.key ? s : DEFAULT_SHORTCUT);
    });
  }

  loadShortcut((s) => (shortcut = s));
  // Deckkraft der Zonen beim Ziehen, eingestellt in der App
  let zoneAlpha = null;
  const loadAlpha = () =>
    chrome.storage &&
    chrome.storage.local &&
    chrome.storage.local.get({ appZoneAlpha: null }, (d) => (zoneAlpha = typeof d.appZoneAlpha === "number" ? d.appZoneAlpha : null));
  loadAlpha();
  let zoneInfo = null;
  loadZones((list, info) => {
    zones = list;
    zoneInfo = info || null;
    barZones = hotOf(list, info);
  });

  if (chrome.storage && chrome.storage.onChanged) {
    chrome.storage.onChanged.addListener((changes, area) => {
      if (ArchiverShared.zonesChanged(changes, area)) {
        loadZones((list, info) => {
          zones = list;
          zoneInfo = info || null;
          barZones = hotOf(list, info);
          if (zonesBarEl) removeZones(); // Zonenanzahl kann sich geändert haben
          if (gallery) gallery.render(barZones, zoneInfo);
        });
      }
      if (area === "sync" && changes.shortcut) loadShortcut((s) => (shortcut = s));
      if (area === "local" && changes.appZoneAlpha) loadAlpha();
    });
  }

  // ---- Tastatur ----

  function matchesShortcut(e, combo) {
    if (!combo || !combo.key) return false;
    if (!!e.altKey !== !!combo.alt) return false;
    if (!!e.shiftKey !== !!combo.shift) return false;
    if (!!e.ctrlKey !== !!combo.ctrl) return false;
    if (!!e.metaKey !== !!combo.meta) return false;
    if (e.key.toLowerCase() === combo.key.toLowerCase()) return true;
    // macOS: Option verändert e.key (Option+Shift+A ergibt "Å"), daher
    // zusätzlich die physische Taste vergleichen.
    const code =
      combo.code ||
      (/^[a-z]$/i.test(combo.key) ? `Key${combo.key.toUpperCase()}` : /^[0-9]$/.test(combo.key) ? `Digit${combo.key}` : null);
    return !!code && e.code === code;
  }

  // composedPath statt activeElement, damit auch Eingabefelder im Shadow DOM zählen.
  function isTypingTarget(e) {
    const target = e.composedPath ? e.composedPath()[0] : e.target;
    if (!target || target.nodeType !== 1) return false;
    if (target.isContentEditable) return true;
    return target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.tagName === "SELECT";
  }

  document.addEventListener("keydown", (e) => {
    if (!e.isTrusted || gone()) return; // nur echte Eingaben, keine von der Seite erzeugten Events
    if (recordingShortcut) return; // nimmt das Einstellungen-Panel selbst entgegen

    if (manualOpen) {
      if (gallery && gallery.handleKey(e)) {
        e.preventDefault();
        return;
      }
      if (e.key === "Escape") {
        if (panels && panels.mode) panels.close();
        else closeManualOverlay();
        return;
      }
    }

    if (isTypingTarget(e)) return;

    if (matchesShortcut(e, shortcut)) {
      e.preventDefault();
      // Das Overlay lebt nur im Top-Frame; aus einem iframe heraus dorthin weiterreichen
      if (window === window.top) toggleManualOverlay();
      else chrome.runtime.sendMessage({ type: "toggle-overlay-top" }).catch(() => {});
    }
  });

  // ---- Nachrichten aus background.js ----

  chrome.runtime.onMessage.addListener((message) => {
    if (!message) return;
    if (message.type === "feedback") {
      ensureOverlay().then(() => UI.showToast(rootEl, message.text, message.ok));
    } else if (message.type === "toggle-overlay" && window === window.top) {
      toggleManualOverlay(); // Chrome-Befehl, kommt nur im Top-Frame an
    } else if (message.type === "context-save") {
      dropLibrary = null;
      contextSave(message);
    } else if (message.type === "capture-start" && window === window.top) {
      startAreaCapture();
    }
  });

  // ---- Rechtsklick „Ablegen in“ ----
  // Merkt sich, worauf zuletzt rechts geklickt wurde. Gesichert wird das Bild
  // dort (auch unter Overlays), sonst markierter Text oder ein Link als Karte,
  // sonst ein Bildschirmfoto der sichtbaren Seite.

  let lastContext = null;
  document.addEventListener(
    "contextmenu",
    (e) => {
      if (!e.isTrusted) return; // nur echte Eingaben, keine von der Seite erzeugten Events
      lastContext = { target: e.target, x: e.clientX, y: e.clientY };
    },
    true
  );

  function imageAt(x, y) {
    for (const el of document.elementsFromPoint(x, y)) {
      if (el === document.documentElement || el === document.body) continue;
      if (el.tagName === "VIDEO" && el.poster) return el.poster;
      const url = el.tagName === "IMG" || el.tagName === "PICTURE" ? findBestImageUrl(el) : bgImageOf(el);
      if (url) return url;
      if (el.querySelector) {
        const img = [...el.querySelectorAll("img")].find((i) => {
          const r = i.getBoundingClientRect();
          return x >= r.left && x <= r.right && y >= r.top && y <= r.bottom;
        });
        if (img) return findBestImageUrl(img);
      }
    }
    return null;
  }

  function bgImageOf(el) {
    const bg = getComputedStyle(el).backgroundImage;
    const m = bg && bg.match(/url\(["']?([^"')]+)["']?\)/);
    return m ? m[1] : null;
  }

  function saveCard(card, folder, source) {
    // Textkarten (mit Farbe) landen bei laufender App als Notiz (.md) in der Zone (background.js asTextFile)
    sendSave({ type: "download-image", url: card.dataUrl, suggestedName: card.name, folder, tone: card.tone, textCard: !!card.tone, ...source });
  }

  // Farben der Textkarten, gleiche Werte wie in der App (app.js QUOTE_TONES):
  // Hintergrund, Schrift Ton in Ton, Deckkraft für Kennung und Fuß
  const CARD_TONES = {
    red: ["#ff0000", "#5c0000", 0.7],
    green: ["#00b050", "#003d1c", 0.7],
    violet: ["#7030a0", "#e2c8f5", 0.75],
    cyan: ["#00b0f0", "#003d54", 0.7],
    yellow: ["#ffff00", "#5c5c00", 0.75],
  };
  // Die Farbe wird zu Beginn jeder Markierung zufällig bestimmt (nie zweimal
  // dieselbe hintereinander) und gilt, bis neu markiert wird
  let markTone = null;
  function newTone() {
    const keys = Object.keys(CARD_TONES).filter((k) => k !== markTone);
    markTone = keys[Math.floor(Math.random() * keys.length)];
    return markTone;
  }
  function toneFor() {
    return markTone || newTone();
  }

  // Markierung in der Farbe der späteren Textkarte, schon während des Markierens
  let toneStyle = null;
  function showTone() {
    if (!toneStyle) {
      toneStyle = document.createElement("style");
      toneStyle.setAttribute("data-archiver-tone", "");
      (document.head || document.documentElement).appendChild(toneStyle);
    }
    const [bg, fg] = CARD_TONES[toneFor()];
    toneStyle.textContent =
      `::selection{background:${bg}!important;color:${fg}!important}` +
      `::highlight(archiver-marks){background-color:${bg};color:${fg}}`;
  }

  // ---- PDF, Video und Audio ----
  // Links auf PDFs, Video- und Audiodateien werden als Datei gesichert, nicht als
  // Karte. YouTube und Vimeo lassen sich nicht laden: dort entsteht eine
  // Karte mit Vorschaubild, Titel und Domain.

  const MEDIA_FILE = /\.(pdf|mp4|m4v|mov|webm|wav|mp3|aif|aiff|m4a|flac)$/i;
  const AUDIO_FILE = /\.(wav|mp3|aif|aiff|m4a|flac)$/i;

  function isMediaFileUrl(url) {
    try {
      const u = new URL(url, location.href);
      return /^https?:$/.test(u.protocol) && MEDIA_FILE.test(u.pathname);
    } catch {
      return false;
    }
  }

  // Direkt geöffnete Datei (PDF oder Audio): als Datei sichern
  function isPdfPage() {
    const type = document.contentType || "";
    return type === "application/pdf" || /^audio\//.test(type) || /\.pdf$/i.test(location.pathname) || AUDIO_FILE.test(location.pathname);
  }

  // Adresse einer YouTube- oder Vimeo-Seite, sonst null
  function videoPageUrl(url) {
    try {
      const u = new URL(url, location.href);
      const host = u.hostname.replace(/^(www|m)\./, "");
      if (host === "youtube.com" && ((u.pathname === "/watch" && u.searchParams.get("v")) || /^\/shorts\/[\w-]+/.test(u.pathname))) return u.href;
      if (host === "youtu.be" && u.pathname.length > 1) return u.href;
      if (host === "vimeo.com" && /^\/\d+/.test(u.pathname)) return u.href;
    } catch {}
    return null;
  }

  // Bildtext zu einem Bild (0.29): nur Text, den man auf der Seite markiert
  // hat, während man das Bild sichert. Sonst kein Bildtext.
  function captionNear() {
    const text = String((window.getSelection && window.getSelection()) || "").replace(/\s+/g, " ").trim();
    return text ? text.slice(0, 2000) : null;
  }

  // Bilder in der Markierung (0.29.1): wer Bild und Text zusammen markiert,
  // sichert das Bild mit dem Text als Bildtext, nicht nur eine Textkarte.
  function selectedImages() {
    const sel = window.getSelection && window.getSelection();
    if (!sel || sel.isCollapsed || !sel.rangeCount) return [];
    const range = sel.getRangeAt(0);
    const c = range.commonAncestorContainer;
    const root = c.nodeType === 1 ? c : c.parentElement;
    if (!root) return [];
    const imgs = [root, ...root.querySelectorAll("img")].filter((n) => n.tagName === "IMG" && range.intersectsNode(n));
    return imgs.filter((i) => Math.max(i.naturalWidth || 0, i.width || 0) >= 80).slice(0, 12);
  }

  function saveSelectedImages(imgs, folder, caption = captionNear()) {
    const source = currentPageSource();
    for (const img of imgs) {
      const url = findBestImageUrl(img) || img.currentSrc || img.src;
      if (url) sendSave({ type: "download-image", url: resolveUrl(url), folder, caption, ...source });
    }
  }

  function saveMediaFile(url, folder, source) {
    sendSave({ type: "download-image", url: resolveUrl(url), folder, ...source });
  }

  async function saveVideoCard(url, folder, fallbackTitle) {
    const info = await chrome.runtime.sendMessage({ type: "video-info", url }).catch(() => null);
    const title = (info && info.title) || fallbackTitle || "";
    const card = await videoCard(title, url, info && info.thumb);
    saveCard(card, folder, { pageUrl: url, pageTitle: title || url });
  }

  // Link oder Adresse sichern: Datei, Video-Karte oder Link-Karte
  function saveLink(link, title, folder) {
    const video = videoPageUrl(link);
    if (isMediaFileUrl(link)) saveMediaFile(link, folder, currentPageSource());
    else if (video) saveVideoCard(video, folder, title);
    else saveCard(linkCard(title, link), folder, { pageUrl: link, pageTitle: title || link });
  }

  // Die geöffnete Seite selbst: PDF als Datei, Video als Karte, sonst Linkkarte
  // mit der ganzen Seite als MHTML dahinter (3.23.0, background.js archiveTab)
  function savePage(folder, source) {
    const video = videoPageUrl(location.href);
    if (isPdfPage()) saveMediaFile(location.href, folder, { pageUrl: location.href, pageTitle: document.title || null });
    else if (video) saveVideoCard(video, folder, document.title.replace(/ - YouTube$/, ""));
    else saveCard(linkCard(document.title.trim(), location.href), folder, { ...source, pageUrl: location.href, pageTitle: document.title || null, archivePage: true });
  }

  function contextSave(msg) {
    const source = currentPageSource();
    const ctx = lastContext;
    const folder = msg.folder;
    const selection = String(window.getSelection() || "").trim() || msg.selectionText;
    if (msg.videoUrl && /^https?:/.test(msg.videoUrl)) {
      saveMediaFile(msg.videoUrl, folder, source);
    } else if (msg.videoUrl && videoPageUrl(location.href)) {
      savePage(folder, source);
    } else if (msg.srcUrl) {
      const best = ctx && ctx.target && ctx.target.tagName === "IMG" ? findBestImageUrl(ctx.target) : null;
      const caption = captionNear();
      sendSave({ type: "download-image", url: resolveUrl(best || msg.srcUrl), folder, caption, ...source });
    } else if (selection && selectedImages().length) {
      saveSelectedImages(selectedImages(), folder);
    } else if (selection) {
      saveCard(textCard(selection, source), folder, { ...source, quote: selection });
    } else if (msg.linkUrl) {
      const a = ctx && ctx.target && ctx.target.closest && ctx.target.closest("a[href]");
      const title = ((a && (a.textContent || a.title)) || "").replace(/\s+/g, " ").trim();
      saveLink(msg.linkUrl, title, folder);
    } else if (isPdfPage() || videoPageUrl(location.href)) {
      savePage(folder, source);
    } else {
      const url = ctx ? imageAt(ctx.x, ctx.y) : null;
      if (url) sendSave({ type: "download-image", url: resolveUrl(url), folder, ...source });
      else sendSave({ type: "capture-visible", folder, ...source });
    }
  }

  // ---- Karten für Text und Links: typografisch gesetzt, als PNG ----
  // So bleiben auch Zitate und Links Objekte in der Zone, mit Quelle.

  const CARD_W = 1400;
  const CARD_PAD = 110;
  const SANS = '-apple-system, BlinkMacSystemFont, "Helvetica Neue", Arial, sans-serif';
  const MONO = 'ui-monospace, "SF Mono", Menlo, monospace';

  function wrapLines(ctx, text, width, maxLines) {
    const lines = [];
    for (const para of text.split(/\n+/)) {
      let line = "";
      for (const word of para.split(/\s+/).filter(Boolean)) {
        const test = line ? line + " " + word : word;
        if (ctx.measureText(test).width > width && line) {
          lines.push(line);
          line = word;
        } else line = test;
      }
      if (line) lines.push(line);
      if (lines.length >= maxLines) break;
    }
    if (lines.length > maxLines) lines.length = maxLines;
    const full = text.replace(/\s+/g, " ").trim();
    if (lines.join(" ").length < full.length - 1) lines[lines.length - 1] = lines[lines.length - 1].replace(/\s*\S*$/, "") + " …";
    return lines;
  }

  function hostOf(url) {
    try {
      return new URL(url).hostname.replace(/^www\./, "");
    } catch {
      return "";
    }
  }

  function renderCard(draw, height, bg) {
    const canvas = document.createElement("canvas");
    canvas.width = CARD_W;
    canvas.height = height;
    const ctx = canvas.getContext("2d");
    ctx.fillStyle = bg || "#f4f3ef";
    ctx.fillRect(0, 0, CARD_W, height);
    ctx.fillStyle = "#141414";
    ctx.textBaseline = "top";
    draw(ctx);
    return canvas.toDataURL("image/png");
  }

  function cardName(text, fallback) {
    const words = text.replace(/[\\/:*?"<>|\n\r]+/g, " ").trim().split(/\s+/).slice(0, 8).join(" ");
    return ((words || fallback).slice(0, 80).replace(/[.,;:!?…\s]+$/, "") || fallback) + ".png";
  }

  function textCard(text, source) {
    const tone = toneFor();
    const [bg, fg, dim] = CARD_TONES[tone];
    const measure = document.createElement("canvas").getContext("2d");
    const size = text.length < 140 ? 64 : text.length < 400 ? 46 : 36;
    measure.font = `500 ${size}px ${SANS}`;
    const lines = wrapLines(measure, text, CARD_W - CARD_PAD * 2, 22);
    const lh = Math.round(size * 1.28);
    const height = CARD_PAD * 2 + 80 + lines.length * lh + 60;
    const date = new Date().toLocaleDateString(document.documentElement.lang || undefined);
    const dataUrl = renderCard((ctx) => {
      ctx.fillStyle = fg;
      ctx.font = `400 22px ${MONO}`;
      ctx.globalAlpha = dim;
      ctx.fillText(t("cardText").toUpperCase(), CARD_PAD, CARD_PAD);
      ctx.globalAlpha = 1;
      ctx.font = `500 ${size}px ${SANS}`;
      lines.forEach((l, i) => ctx.fillText(l, CARD_PAD, CARD_PAD + 80 + i * lh));
      ctx.font = `400 22px ${MONO}`;
      ctx.globalAlpha = dim;
      ctx.fillText([hostOf(source.pageUrl), date].filter(Boolean).join("  ·  "), CARD_PAD, height - CARD_PAD + 10);
      ctx.globalAlpha = 1;
    }, height, bg);
    return { dataUrl, name: cardName(text, "Text"), tone };
  }

  function linkCard(title, url) {
    const measure = document.createElement("canvas").getContext("2d");
    measure.font = `600 72px ${SANS}`;
    const lines = wrapLines(measure, title || hostOf(url) || url, CARD_W - CARD_PAD * 2, 4);
    const height = Math.max(640, CARD_PAD * 2 + 80 + lines.length * 88 + 90);
    const dataUrl = renderCard((ctx) => {
      ctx.font = `400 22px ${MONO}`;
      ctx.fillStyle = "#8a8a85";
      ctx.fillText((t("cardLink") + "  ·  " + hostOf(url)).toUpperCase(), CARD_PAD, CARD_PAD);
      ctx.fillStyle = "#141414";
      ctx.font = `600 72px ${SANS}`;
      lines.forEach((l, i) => ctx.fillText(l, CARD_PAD, CARD_PAD + 80 + i * 88));
      ctx.font = `400 24px ${MONO}`;
      ctx.fillStyle = "#2d5aeb";
      const short = url.length > 90 ? url.slice(0, 89) + "…" : url;
      ctx.fillText(short, CARD_PAD, height - CARD_PAD + 6);
    }, height);
    return { dataUrl, name: cardName(title || hostOf(url), "Link") };
  }

  // Video-Karte: Vorschaubild oben (16:9) mit Abspiel-Zeichen, darunter
  // „VIDEO · youtube.com“ und der Titel.
  function videoCard(title, url, thumbDataUrl) {
    return new Promise((resolve) => {
      const draw = (img) => {
        const measure = document.createElement("canvas").getContext("2d");
        measure.font = `600 64px ${SANS}`;
        const lines = wrapLines(measure, title || hostOf(url) || url, CARD_W - CARD_PAD * 2, 3);
        const thumbH = Math.round((CARD_W * 9) / 16);
        const height = thumbH + CARD_PAD + 30 + lines.length * 78 + CARD_PAD - 10;
        const dataUrl = renderCard((ctx) => {
          ctx.fillStyle = "#1c1c1c";
          ctx.fillRect(0, 0, CARD_W, thumbH);
          if (img) {
            const s = Math.max(CARD_W / img.width, thumbH / img.height);
            const w = img.width * s;
            const h = img.height * s;
            ctx.drawImage(img, (CARD_W - w) / 2, (thumbH - h) / 2, w, h);
          }
          const cx = CARD_W / 2;
          const cy = thumbH / 2;
          ctx.fillStyle = "rgba(0, 0, 0, 0.55)";
          ctx.beginPath();
          ctx.arc(cx, cy, 64, 0, Math.PI * 2);
          ctx.fill();
          ctx.fillStyle = "#ffffff";
          ctx.beginPath();
          ctx.moveTo(cx - 20, cy - 32);
          ctx.lineTo(cx + 34, cy);
          ctx.lineTo(cx - 20, cy + 32);
          ctx.closePath();
          ctx.fill();
          ctx.font = `400 22px ${MONO}`;
          ctx.fillStyle = "#8a8a85";
          ctx.fillText((t("cardVideo") + "  ·  " + hostOf(url)).toUpperCase(), CARD_PAD, thumbH + CARD_PAD - 30);
          ctx.fillStyle = "#141414";
          ctx.font = `600 64px ${SANS}`;
          lines.forEach((l, i) => ctx.fillText(l, CARD_PAD, thumbH + CARD_PAD + 30 + i * 78));
        }, height);
        resolve({ dataUrl, name: cardName(title || hostOf(url), "Video") });
      };
      if (!thumbDataUrl) return draw(null);
      const img = new Image();
      img.onload = () => draw(img);
      img.onerror = () => draw(null);
      img.src = thumbDataUrl;
    });
  }

  // ---- Ausschnitt sichern: Rahmen aufziehen, dann Zone wählen ----

  let areaEl = null;

  function startAreaCapture() {
    if (areaEl) return;
    ensureOverlay().then(() => {
      if (manualOpen) closeManualOverlay();
      areaEl = UI.el("div", "area-layer");
      const hint = UI.el("div", "area-hint", t("areaHint"));
      const box = UI.el("div", "area-box");
      areaEl.append(hint, box);
      rootEl.appendChild(areaEl);
      let start = null;
      let rect = null;
      const draw = (r) => {
        box.style.cssText = `left:${r.x}px;top:${r.y}px;width:${r.w}px;height:${r.h}px;display:block`;
      };
      areaEl.addEventListener("mousedown", (e) => {
        if (e.button !== 0 || e.target.closest(".area-pick")) return;
        e.preventDefault();
        const old = areaEl.querySelector(".area-pick");
        if (old) old.remove();
        start = { x: e.clientX, y: e.clientY };
      });
      areaEl.addEventListener("mousemove", (e) => {
        if (!start) return;
        rect = {
          x: Math.min(start.x, e.clientX),
          y: Math.min(start.y, e.clientY),
          w: Math.abs(e.clientX - start.x),
          h: Math.abs(e.clientY - start.y),
        };
        draw(rect);
      });
      areaEl.addEventListener("mouseup", () => {
        if (!start) return;
        start = null;
        if (rect && rect.w > 8 && rect.h > 8) showAreaPicker(rect);
      });
      document.addEventListener("keydown", areaKey, true);
    });
  }

  function endAreaCapture() {
    document.removeEventListener("keydown", areaKey, true);
    if (areaEl) areaEl.remove();
    areaEl = null;
  }

  function areaKey(e) {
    if (!e.isTrusted) return; // nur echte Eingaben, keine von der Seite erzeugten Events
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      endAreaCapture();
      return;
    }
    const pick = areaEl && areaEl.querySelector(".area-pick");
    if (pick && /^[1-9]$/.test(e.key)) {
      const chip = pick.querySelectorAll(".area-chip")[Number(e.key) - 1];
      if (chip) {
        e.preventDefault();
        e.stopPropagation();
        chip.click();
      }
    }
  }

  function showAreaPicker(rect) {
    const pick = UI.el("div", "area-pick");
    pick.appendChild(UI.el("span", "area-pick-label", t("areaPick")));
    zones.forEach((name, i) => {
      const chip = UI.el("button", "area-chip");
      chip.type = "button";
      if (i < 9) chip.appendChild(UI.el("span", "area-key", String(i + 1)));
      chip.appendChild(document.createTextNode(String(name).split("/").pop()));
      chip.addEventListener("click", () => captureArea(rect, isRootZoneName(name) ? null : name));
      pick.appendChild(chip);
    });
    const below = rect.y + rect.h + 12;
    pick.style.left = Math.max(12, Math.min(rect.x, window.innerWidth - 420)) + "px";
    pick.style.top = (below + 54 < window.innerHeight ? below : Math.max(12, rect.y - 56)) + "px";
    areaEl.appendChild(pick);
  }

  function captureArea(rect, folder) {
    endAreaCapture();
    // zwei Frames warten, damit Rahmen und Leiste nicht mit aufs Foto kommen
    requestAnimationFrame(() =>
      requestAnimationFrame(() =>
        setTimeout(() => {
          chrome.runtime.sendMessage({
            type: "capture-visible",
            folder,
            rect: { ...rect, vw: window.innerWidth },
            ...currentPageSource(),
          });
        }, 60)
      )
    );
  }

  // ---- Manuell geöffnetes Overlay ----

  function toggleManualOverlay() {
    if (manualOpen) closeManualOverlay();
    else openManualOverlay();
  }

  function openManualOverlay() {
    if (manualOpen) return;
    manualOpen = true;
    ensureOverlay().then(() => {
      if (!manualOpen) return;
      removeZones();
      if (!galleryPanelEl) buildGalleryPanel();
      debugLog("Overlay manuell geöffnet");
    });
  }

  function closeManualOverlay() {
    manualOpen = false;
    removeGalleryPanel();
    debugLog("Overlay geschlossen");
  }

  // Wie die Vollansicht: Toolbar mit Filter, Größe, Einstellungen, Information,
  // darunter alle Zonen. Bilder vom Desktop lassen sich direkt auf eine Zone ziehen.
  function buildGalleryPanel() {
    galleryPanelEl = UI.el("div", "overlay-gallery");
    gallery = UI.createGallery({ mount: rootEl, root: shadow, acceptExternalDrops: true });

    const dashboardBtn = UI.button("mini-action", t("fullView"), () => chrome.runtime.sendMessage({ type: "open-dashboard" }));
    dashboardBtn.title = t("fullViewTitle");
    panelHostEl = UI.el("div", "panel-host");
    panels = UI.createPanelController(panelHostEl, {
      settingsHeaderExtras: [dashboardBtn],
      onRecordingChange: (on) => (recordingShortcut = on),
    });

    const toolbar = UI.buildToolbar(gallery, galleryPanelEl);
    toolbar.appendChild(UI.buildPanelButtons(panels));

    galleryPanelEl.appendChild(toolbar);
    galleryPanelEl.appendChild(toolbar.libTab);
    galleryPanelEl.appendChild(gallery.el);
    rootEl.appendChild(galleryPanelEl);
    rootEl.appendChild(panelHostEl);
    gallery.render(barZones, zoneInfo);
  }

  function removeGalleryPanel() {
    if (panels) panels.close();
    if (gallery) gallery.destroy();
    if (galleryPanelEl) galleryPanelEl.remove();
    if (panelHostEl) panelHostEl.remove();
    panels = gallery = galleryPanelEl = panelHostEl = null;
    recordingShortcut = false;
  }

  // ---- Zonenleiste (nur während eines Drags auf der Seite) ----

  function showZones() {
    refreshZones();
    ensureOverlay().then(() => {
      if (zonesBarEl || manualOpen || dragCounter <= 0) return; // schon da, Overlay offen oder Drag vorbei

      zonesBarEl = UI.el("div", "zones-bar");
      if (zoneAlpha != null) zonesBarEl.style.setProperty("--zone-alpha", String(zoneAlpha));
      zoneEls = barZones.map((name, i) => {
        const zone = UI.el("div", "zone" + (isRootZoneName(name) ? " zone-root" : ""));
        zone.dataset.index = String(i);
        // Schwebende Karte wie in der App (.p-hot): Nummer oben, Name in der Mitte, Hinweis unten, Unterzone mit grauem Elternteil
        zone.appendChild(UI.el("span", "zone-key", String(i + 1).padStart(2, "0")));
        const label = UI.el("div", "zone-label");
        const parts = String(name).split("/");
        if (parts.length > 1) label.appendChild(UI.el("span", "zone-parent", parts.slice(0, -1).join(" / ") + " / "));
        label.appendChild(document.createTextNode(parts[parts.length - 1]));
        zone.appendChild(label);
        zone.appendChild(UI.el("div", "zone-hint", t("dropRelease")));
        zonesBarEl.appendChild(zone);
        return zone;
      });
      rootEl.appendChild(zonesBarEl);
      showLibBar();
      debugLog("Zonen angezeigt:", barZones);
    });
  }

  // Zonen bei jedem Drag frisch holen (App kann inzwischen gestartet oder
  // eine Zone angelegt worden sein). Ändert sich etwas, wird die Leiste neu gebaut.
  let zonesRefreshedAt = 0;
  function refreshZones() {
    if (dropLibrary) return; // per ⇧ andere Bibliothek gewählt: deren Zonen stehen lassen
    if (Date.now() - zonesRefreshedAt < 2000) return;
    zonesRefreshedAt = Date.now();
    loadZones((list, info) => {
      const hot = hotOf(list, info);
      if (list.join("\n") === zones.join("\n") && hot.join("\n") === barZones.join("\n")) return;
      zones = list;
      barZones = hot;
      if (zonesBarEl) {
        zonesBarEl.remove();
        zonesBarEl = null;
        zoneEls = [];
        showZones();
      }
    });
  }

  function removeZones() {
    if (zonesBarEl) {
      zonesBarEl.remove();
      zonesBarEl = null;
      zoneEls = [];
    }
    removeLibBar();
    dragCounter = 0;
    zonesOn = false;
  }

  // Abgelegt (3.29.8, wie die App, DESIGN.md 4.13): Das Bild fliegt als Miniatur
  // vom Zeiger in die Ecke der Zielkarte, die Leiste steht 0.7 s und blendet aus.
  function landZones(idx, pos, src) {
    const bar = zonesBarEl;
    const zone = zoneEls[idx];
    if (!bar || !zone) return removeZones();
    zonesBarEl = null;
    zoneEls = [];
    dragCounter = 0;
    zonesOn = false;
    removeLibBar();
    zone.classList.add("active");
    if (src) stampZone(zone, src, pos);
    bar.classList.add("leaving");
    setTimeout(() => bar.remove(), 1000);
  }

  // Miniatur oben rechts in der Karte; lädt das Bild nicht in 250 ms, bleibt es weg
  function stampZone(zone, src, pos) {
    const t0 = Date.now();
    const img = new Image();
    img.className = "zone-stamp";
    img.onload = () => {
      if (Date.now() - t0 > 250) return;
      zone.appendChild(img);
      if (!pos || matchMedia("(prefers-reduced-motion: reduce)").matches) return;
      const r = img.getBoundingClientRect();
      const dx = pos.x - (r.left + r.width / 2);
      const dy = pos.y - (r.top + r.height / 2);
      img.animate([{ transform: `translate(${dx}px, ${dy}px) scale(2.4)` }, { transform: "none" }], {
        duration: 280,
        easing: "cubic-bezier(0.2, 0.7, 0.2, 1)",
      });
    };
    img.src = src;
  }

  // Zeiger während eines Ziehens (Browser-Ziehen, Text, Instagram): Zielkarte
  // markieren. Die Leiste steht in voller Höhe (3.29.7, vorher erst ein Streifen).
  function trackZones(x, y) {
    if (!zonesBarEl) return;
    trackLibs(x, y);
    const active = getZoneIndex(x, y);
    zoneEls.forEach((el, i) => el.classList.toggle("active", i === active));
  }

  // Liefert den Zonen-Index unter x/y, oder -1 wenn außerhalb aller Zonen
  function getZoneIndex(x, y) {
    if (!barZones.length || !zonesBarEl) return -1;
    if (y < zonesBarEl.getBoundingClientRect().top) return -1;
    const zoneWidth = window.innerWidth / barZones.length;
    const idx = Math.floor(x / zoneWidth);
    return Math.min(Math.max(idx, 0), barZones.length - 1);
  }

  // ---- Bildquelle ermitteln (High-Res statt Thumbnail) ----

  // srcset nach dem Verfahren der HTML-Spezifikation lesen: eine Adresse endet
  // erst an Leerraum, Kommas darin gehören dazu (Substack, Cloudinary:
  // ".../w_1456,c_limit,f_webp/..."). Ein Komma nach der Adresse oder nach den
  // Angaben trennt die Kandidaten.
  function parseSrcset(srcset) {
    const out = [];
    const s = String(srcset || "");
    let i = 0;
    while (i < s.length) {
      while (i < s.length && /[\s,]/.test(s[i])) i++;
      if (i >= s.length) break;
      let j = i;
      while (j < s.length && !/\s/.test(s[j])) j++;
      let url = s.slice(i, j);
      i = j;
      let desc = "";
      if (/,$/.test(url)) url = url.replace(/,+$/, "");
      else {
        while (i < s.length && s[i] !== ",") desc += s[i++];
        i++;
      }
      if (url) out.push({ url, desc: desc.trim() });
    }
    return out;
  }

  function parseSrcsetBest(srcset) {
    const candidates = parseSrcset(srcset).map(({ url, desc }) => {
      let score = 0;
      const d = desc.split(/\s+/)[0] || "";
      if (d.endsWith("w")) score = parseInt(d, 10) || 0;
      else if (d.endsWith("x")) score = (parseFloat(d) || 1) * 1000;
      return { url, score };
    });
    if (!candidates.length) return null;
    candidates.sort((a, b) => b.score - a.score);
    return candidates[0].url;
  }

  const LAZY_ATTRS = ["data-src", "data-original", "data-full", "data-large", "data-hi-res", "data-zoom-image", "data-large-src", "data-lazy-src"];

  function bestFromImg(img) {
    if (!img) return null;

    const picture = img.closest ? img.closest("picture") : null;
    if (picture) {
      let best = null;
      picture.querySelectorAll("source[srcset]").forEach((source) => {
        const url = parseSrcsetBest(source.getAttribute("srcset"));
        if (url) best = url; // im Markup steht die hochauflösendste Quelle meist zuletzt
      });
      if (best) return best;
    }

    const srcsetBest = parseSrcsetBest(img.getAttribute("srcset"));
    if (srcsetBest) return srcsetBest;

    for (const attr of LAZY_ATTRS) {
      const val = img.getAttribute(attr);
      if (val) return val;
    }

    return img.src || null;
  }

  function findBestImageUrl(el) {
    if (!el) return null;

    // Lightbox-Pattern: <a href="grosses-bild.jpg"><img src="thumbnail.jpg"></a>
    const anchor = el.closest ? el.closest("a[href]") : null;
    if (anchor) {
      const href = anchor.getAttribute("href");
      if (href && /\.(jpe?g|png|webp|gif|avif|svg)(\?.*)?$/i.test(href)) {
        return href;
      }
    }

    if (el.tagName === "IMG") {
      const best = bestFromImg(el);
      if (best) return best;
    }

    const nestedImg = el.querySelector && el.querySelector("img");
    if (nestedImg) {
      const best = bestFromImg(nestedImg);
      if (best) return best;
    }

    let node = el;
    for (let i = 0; i < 6 && node; i++) {
      const bg = getComputedStyle(node).backgroundImage;
      const match = bg && bg.match(/url\(["']?([^"')]+)["']?\)/);
      if (match) return match[1];
      node = node.parentElement;
    }

    const nestedSvgImage = el.querySelector && el.querySelector("image[href], image[*|href]");
    if (nestedSvgImage) {
      const href = nestedSvgImage.getAttribute("href") || nestedSvgImage.getAttribute("xlink:href");
      if (href) return href;
    }

    return null;
  }

  // ---- Mehrere Stellen markieren (⌥, seit 3.29.1; vorher ⌘) ----
  // Mit gedrückter ⌥-Taste kommen weitere Stellen dazu: Text wie gewohnt
  // markieren, Bilder anklicken (nochmal anklicken nimmt sie wieder raus).
  // Chrome kennt nur eine Markierung, deshalb merkt sich Archiver die früheren
  // Stücke selbst: Text bleibt in der Farbe der Markierung (CSS Highlight, die
  // Seite wird nicht verändert), Bilder tragen den gestrichelten Ring.
  // Ziehen eines Stücks nimmt alle mit:
  //   nur Text → eine Textkarte, Stücke als Absätze in Markierreihenfolge
  //   Text und Bild → jedes Bild mit allem Text als Bildtext
  //   nur Bilder → jedes Bild einzeln
  // Klick ohne ⌥ daneben oder Esc hebt alles auf.

  const marks = []; // { range, text } | { img }
  const MIN_MARK_IMG = 80;
  let markLayer = null;
  let markFrame = 0;
  const reduceMotion = window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)");

  const hasMarks = () => marks.length > 0;

  function bigImage(img) {
    return img && img.tagName === "IMG" && Math.max(img.naturalWidth || 0, img.width || 0) >= MIN_MARK_IMG;
  }

  function imageUnder(x, y) {
    for (const el of document.elementsFromPoint(x, y)) {
      if (el === document.documentElement || el === document.body) continue;
      if (el.tagName === "ARCHIVER-OVERLAY") return null;
      if (bigImage(el)) return el;
      // Bilder unter einer Fläche (Instagram u. a.)
      const img = el.querySelectorAll && [...el.querySelectorAll("img")].find((i) => {
        const r = i.getBoundingClientRect();
        return bigImage(i) && x >= r.left && x <= r.right && y >= r.top && y <= r.bottom;
      });
      if (img) return img;
    }
    return null;
  }

  function addImageMark(img) {
    if (!marks.some((m) => m.img === img)) marks.push({ img });
  }

  // Die laufende Markierung wird ein gemerktes Stück
  function keepLiveSelection() {
    const text = selectedText();
    if (!text) return false;
    const images = selectedImages();
    marks.push({ range: window.getSelection().getRangeAt(0).cloneRange(), text: captionNear() || text });
    images.forEach(addImageMark);
    return true;
  }

  function collectPieces() {
    const texts = [];
    const images = [];
    const addImg = (img) => !images.includes(img) && images.push(img);
    for (const m of marks) {
      if (m.img) addImg(m.img);
      else if (m.text) texts.push(m.text);
    }
    if (selectedText()) {
      texts.push(captionNear());
      selectedImages().forEach(addImg);
    }
    const text = texts.join("\n\n");
    return { text, caption: text ? text.slice(0, 2000) : null, images: images.slice(0, 12) };
  }

  function insideMarks(x, y) {
    const hit = (r, pad) => x >= r.left - pad && x <= r.right + pad && y >= r.top - pad && y <= r.bottom + pad;
    return marks.some((m) => (m.img ? hit(m.img.getBoundingClientRect(), 3) : [...m.range.getClientRects()].some((r) => hit(r, 2))));
  }

  function clearMarks() {
    if (!marks.length) return;
    marks.length = 0;
    paintMarks();
  }

  function paintMarks() {
    if (window.CSS && CSS.highlights && window.Highlight) {
      const ranges = marks.filter((m) => m.range).map((m) => m.range);
      if (ranges.length) CSS.highlights.set("archiver-marks", new Highlight(...ranges));
      else CSS.highlights.delete("archiver-marks");
    }
    showTone();
    scheduleMarkRings();
  }

  // Ring wie in der App (DESIGN 4.1): 1 px gestrichelt 4/4, 3 px Abstand,
  // eckig wie das Bild. Weiß mit feinem dunklen Schatten, damit er auf hellen
  // und dunklen Bildern steht.
  const MARK_RING =
    "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg'%3E%3Crect x='.5' y='.5' style='width:calc(100%25 - 1px);height:calc(100%25 - 1px)' fill='none' stroke='white' stroke-dasharray='4 4'/%3E%3C/svg%3E\") 0 0 / 100% 100% no-repeat";

  function placeMarkRings() {
    markFrame = 0;
    const imgs = marks.filter((m) => m.img).map((m) => m.img);
    if (!imgs.length && !markLayer) return;
    if (!markLayer) {
      markLayer = document.createElement("div");
      markLayer.setAttribute("data-archiver-marks", "");
      markLayer.style.cssText = "position:fixed;left:0;top:0;width:0;height:0;z-index:2147483000;pointer-events:none";
      document.documentElement.appendChild(markLayer);
    }
    while (markLayer.children.length > imgs.length) markLayer.lastChild.remove();
    while (markLayer.children.length < imgs.length) {
      const ring = document.createElement("div");
      ring.style.cssText =
        `position:fixed;left:0;top:0;background:${MARK_RING};filter:drop-shadow(0 0 1px rgba(0,0,0,.7));opacity:0;` +
        (reduceMotion && reduceMotion.matches ? "" : "transition:opacity .18s ease-out");
      markLayer.appendChild(ring);
      requestAnimationFrame(() => (ring.style.opacity = "1"));
    }
    imgs.forEach((img, i) => {
      const r = img.getBoundingClientRect();
      const ring = markLayer.children[i];
      ring.style.transform = `translate(${Math.round(r.left - 3)}px, ${Math.round(r.top - 3)}px)`;
      ring.style.width = Math.round(r.width + 6) + "px";
      ring.style.height = Math.round(r.height + 6) + "px";
    });
  }

  function scheduleMarkRings() {
    if (!markFrame) markFrame = requestAnimationFrame(placeMarkRings);
  }
  window.addEventListener("scroll", () => hasMarks() && scheduleMarkRings(), true);
  window.addEventListener("resize", () => hasMarks() && scheduleMarkRings());

  // Den Klick nach einer ⌥-Geste schlucken, sonst lädt Chrome den Link herunter
  function swallowNextClick() {
    const stop = (c) => {
      c.preventDefault();
      c.stopImmediatePropagation();
    };
    window.addEventListener("click", stop, { capture: true, once: true });
    setTimeout(() => window.removeEventListener("click", stop, true), 600);
  }

  let metaMark = null; // { x, y } während einer ⌥-Markierung

  // Seiten, die schon bei pointerdown/pointerup weiterspringen (Pinterest u. a.),
  // sehen ⌥-Klicks gar nicht erst. Nur stopPropagation: Archivers eigene
  // Listener am window laufen weiter, die Textauswahl bleibt möglich.
  const OWN_FIELDS = "input, textarea, select, [contenteditable=''], [contenteditable='true'], archiver-overlay";
  for (const type of ["pointerdown", "pointerup", "mouseup"]) {
    window.addEventListener(
      type,
      (e) => {
        if (!e.isTrusted || !e.altKey || e.button !== 0 || manualOpen || gone()) return;
        if (e.target instanceof Element && e.target.closest(OWN_FIELDS)) return;
        e.stopPropagation();
      },
      true
    );
  }
  window.addEventListener(
    "mousedown",
    (e) => {
      if (!e.isTrusted) return; // nur echte Eingaben, keine von der Seite erzeugten Events
      if (e.button !== 0 || manualOpen || textDrag || gone()) return;
      const t = e.target;
      if (t instanceof Element && t.closest("input, textarea, select, [contenteditable=''], [contenteditable='true'], archiver-overlay")) return;
      if (!e.altKey) {
        // Klick daneben hebt alle Stücke auf. Nicht auf einem Stück und nicht auf
        // der laufenden Markierung: von dort zieht das Ziehen unten alle Stücke mit.
        const onLive = !!selectedText() && insideSelection(e.clientX, e.clientY);
        if (hasMarks() && !onLive && !insideMarks(e.clientX, e.clientY)) clearMarks();
        return;
      }
      // ⌥-Klick gehört der Markierung: Die Seite bekommt ihn nicht, ein Link
      // führt nicht weiter und Chrome lädt ihn nicht herunter (3.29.3)
      e.stopPropagation();
      swallowNextClick();
      keepLiveSelection();
      const img = imageUnder(e.clientX, e.clientY);
      if (img) {
        // ⌥-Klick auf ein Bild: dazunehmen oder wieder herausnehmen
        e.preventDefault();
        e.stopImmediatePropagation();
        const i = marks.findIndex((m) => m.img === img);
        if (i === -1) marks.push({ img });
        else marks.splice(i, 1);
        window.getSelection().removeAllRanges();
        swallowNextClick();
      } else metaMark = { x: e.clientX, y: e.clientY };
      paintMarks();
    },
    true
  );
  window.addEventListener(
    "mouseup",
    (e) => {
      if (!e.isTrusted || !metaMark) return;
      const moved = Math.hypot(e.clientX - metaMark.x, e.clientY - metaMark.y) > TEXT_DRAG_THRESHOLD;
      metaMark = null;
      if (moved && selectedText()) swallowNextClick();
    },
    true
  );
  window.addEventListener(
    "keydown",
    (e) => {
      if (e.isTrusted && e.key === "Escape" && hasMarks() && !textDrag) clearMarks();
    },
    true
  );

  // ---- Markierten Text ziehen ----
  // Manche Seiten lassen eine Markierung nicht nativ ziehen (ein neuer Klick
  // markiert neu). Archiver übernimmt deshalb das Ziehen selbst: Drücken auf
  // die Markierung und Ziehen blendet die Zonen ein, Loslassen auf einer Zone
  // sichert den Text als Karte. Ein einfacher Klick hebt die Markierung auf.

  const TEXT_DRAG_THRESHOLD = 5;
  let textDrag = null; // { x, y, text, moving, ghost }

  function selectedText() {
    const sel = window.getSelection();
    if (!sel || sel.isCollapsed || !sel.rangeCount) return "";
    const node = sel.anchorNode && (sel.anchorNode.nodeType === 1 ? sel.anchorNode : sel.anchorNode.parentElement);
    if (node && node.closest && node.closest("input, textarea, [contenteditable=''], [contenteditable='true']")) return "";
    return String(sel).trim();
  }

  // ---- Markierung im Fokus ----
  // Sobald Text markiert ist (ab 3 Zeichen, nicht in Eingabefeldern), legt
  // sich eine gestrichelte Outline um den markierten Block, der Rest der Seite
  // wird dunkler (wie beim Bereich-Auswählen). Das Blau der Markierung
  // verschwindet. Der Block lässt sich als Ganzes in eine Zone ziehen.

  const SPOT_PAD = 8;
  let spotEl = null;
  let spotFrame = 0;
  let spotRect = null; // sichtbarer Block, für das Ziehen

  // Umriss aller Zeilen der Markierung
  function selectionBlock() {
    const sel = window.getSelection();
    if (!sel || !sel.rangeCount || selectedText().length < 3) return null;
    const rects = [...sel.getRangeAt(0).getClientRects()].filter((r) => r.width > 1 && r.height > 1);
    if (!rects.length) return null;
    const box = { left: Infinity, top: Infinity, right: -Infinity, bottom: -Infinity };
    for (const r of rects) {
      box.left = Math.min(box.left, r.left);
      box.top = Math.min(box.top, r.top);
      box.right = Math.max(box.right, r.right);
      box.bottom = Math.max(box.bottom, r.bottom);
    }
    box.left -= SPOT_PAD;
    box.top -= SPOT_PAD;
    box.right += SPOT_PAD;
    box.bottom += SPOT_PAD;
    return box;
  }

  function hideSpotlight() {
    spotRect = null;
    if (spotEl) spotEl.style.opacity = "0";
  }

  function updateSpotlight() {
    spotFrame = 0;
    if (window !== window.top && window.innerWidth < 200) return; // kleine Frames (Werbung) auslassen
    const box = selectionBlock();
    if (!box) return hideSpotlight();
    if (!spotEl) {
      spotEl = document.createElement("div");
      spotEl.setAttribute("data-archiver-spotlight", "");
      spotEl.style.cssText =
        "position:fixed;left:0;top:0;z-index:2147483000;pointer-events:none;" +
"opacity:0;";
      document.documentElement.appendChild(spotEl);
      spotEl.getBoundingClientRect();
    }
    spotEl.style.transform = `translate(${Math.round(box.left)}px, ${Math.round(box.top)}px)`;
    spotEl.style.width = Math.round(box.right - box.left) + "px";
    spotEl.style.height = Math.round(box.bottom - box.top) + "px";
    spotEl.style.opacity = "1";
    spotRect = box;
  }

  function scheduleSpotlight() {
    if (!spotFrame) spotFrame = requestAnimationFrame(updateSpotlight);
  }

  // Erst nach dem Loslassen der Maus einblenden, nicht während des Markierens
  let selecting = false;
  window.addEventListener("mousedown", (e) => {
    if (!e.isTrusted) return; // nur echte Eingaben, keine von der Seite erzeugten Events
    if (e.button === 0 && spotRect && insideBlock(e.clientX, e.clientY)) return; // Ziehen des Blocks
    selecting = true;
    hideSpotlight();
    // Neue Markierung beginnt: neue Farbe, sofort sichtbar. Mit ⌘ oder auf
    // einem gemerkten Stück bleibt die Farbe, alle Stücke werden eine Karte.
    if (e.button === 0 && !e.altKey && !(hasMarks() && insideMarks(e.clientX, e.clientY))) {
      newTone();
      showTone();
    }
  }, true);
  showTone();
  window.addEventListener("mouseup", () => {
    selecting = false;
    setTimeout(scheduleSpotlight, 0);
  }, true);
  document.addEventListener("selectionchange", () => {
    if (selecting || textDrag) return;
    if (!selectionBlock()) hideSpotlight();
    else scheduleSpotlight();
  });
  window.addEventListener("scroll", () => spotRect && scheduleSpotlight(), true);
  window.addEventListener("resize", () => spotRect && scheduleSpotlight());

  function insideBlock(x, y) {
    const r = spotRect;
    return !!r && x >= r.left && x <= r.right && y >= r.top && y <= r.bottom;
  }

  function insideSelection(x, y) {
    if (insideBlock(x, y)) return true;
    const sel = window.getSelection();
    if (!sel.rangeCount) return false;
    return [...sel.getRangeAt(0).getClientRects()].some(
      (r) => x >= r.left - 2 && x <= r.right + 2 && y >= r.top - 2 && y <= r.bottom + 2
    );
  }

  function endTextDrag() {
    if (!textDrag) return;
    if (textDrag.ghost) textDrag.ghost.remove();
    window.removeEventListener("mousemove", onTextDragMove, true);
    window.removeEventListener("mouseup", onTextDragUp, true);
    window.removeEventListener("keydown", onTextDragKey, true);
    if (textDrag.moving) removeZones();
    textDrag = null;
    scheduleSpotlight();
  }

  function onTextDragMove(e) {
    if (!e.isTrusted) return; // nur echte Eingaben, keine von der Seite erzeugten Events
    const d = textDrag;
    if (!d.moving) {
      if (Math.hypot(e.clientX - d.x, e.clientY - d.y) < TEXT_DRAG_THRESHOLD) return;
      d.moving = true;
      dragCounter = 1;
      loadDragLibs();
      showZones();
      ensureOverlay().then(() => {
        if (textDrag !== d) return;
        // in der Überlagerung, damit die Karte über den Zonen liegt
        d.ghost = d.images.length ? imageGhost(d.images[0], d.text) : cardGhost(d.text);
        rootEl.appendChild(d.ghost);
        moveGhost(d, d.lastX ?? e.clientX, d.lastY ?? e.clientY);
      });
    }
    e.preventDefault();
    d.lastX = e.clientX;
    d.lastY = e.clientY;
    moveGhost(d, e.clientX, e.clientY);
    trackZones(e.clientX, e.clientY);
  }

  // Am Zeiger hängt schon die spätere Textkarte: Farbe der Markierung,
  // Text in Kartenschrift, unten die Quelle.
  function cardGhost(text) {
    const [bg, fg, dim] = CARD_TONES[toneFor()];
    const g = UI.el("div", "text-ghost card");
    g.style.background = bg;
    g.style.color = fg;
    g.appendChild(UI.el("div", "text-ghost-text", text.length > 220 ? text.slice(0, 220).trim() + " …" : text));
    const foot = UI.el("div", "text-ghost-foot", hostOf(location.href));
    foot.style.opacity = String(dim);
    g.appendChild(foot);
    return g;
  }

  // Bild mit Bildtext: am Zeiger das Bild, darunter der Anfang des Textes
  function imageGhost(img, text) {
    const g = UI.el("div", "text-ghost card");
    const pic = document.createElement("img");
    pic.src = img.currentSrc || img.src;
    pic.style.cssText = "display:block;width:100%;max-height:180px;object-fit:cover;margin-bottom:8px";
    g.appendChild(pic);
    if (text) g.appendChild(UI.el("div", "text-ghost-text", text.length > 120 ? text.slice(0, 120).trim() + " …" : text));
    return g;
  }

  function moveGhost(d, x, y) {
    if (!d.ghost) return;
    // am rechten und unteren Rand auf die andere Seite des Zeigers
    const w = d.ghost.offsetWidth;
    const h = d.ghost.offsetHeight;
    const left = x + 14 + w > window.innerWidth ? x - 14 - w : x + 14;
    const top = y + 14 + h > window.innerHeight ? y - 14 - h : y + 14;
    d.ghost.style.transform = `translate(${left}px, ${top}px)`;
  }

  function onTextDragUp(e) {
    if (!e.isTrusted) return; // nur echte Eingaben, keine von der Seite erzeugten Events
    const d = textDrag;
    if (!d.moving) {
      // nur geklickt: wie gewohnt Markierung aufheben, auch die gemerkten Stücke
      window.getSelection().removeAllRanges();
      clearMarks();
    } else {
      e.preventDefault();
      e.stopPropagation();
      const idx = getZoneIndex(e.clientX, e.clientY);
      if (idx !== -1) {
        const name = barZones[idx];
        landZones(idx, { x: e.clientX, y: e.clientY }, d.images.length ? findBestImageUrl(d.images[0]) : null);
        const folder = isRootZoneName(name) ? null : name;
        if (d.images.length) saveSelectedImages(d.images, folder, d.caption);
        else {
          const source = currentPageSource();
          saveCard(textCard(d.text, source), folder, { ...source, quote: d.text });
        }
        clearMarks();
        window.getSelection().removeAllRanges();
      }
    }
    endTextDrag();
  }

  function onTextDragKey(e) {
    if (!e.isTrusted) return; // nur echte Eingaben, keine von der Seite erzeugten Events
    if (e.key !== "Escape") return;
    e.preventDefault();
    endTextDrag();
  }

  window.addEventListener(
    "mousedown",
    (e) => {
      if (!e.isTrusted) return; // nur echte Eingaben, keine von der Seite erzeugten Events
      if (e.button !== 0 || e.altKey || manualOpen || textDrag || gone()) return;
      const text = selectedText();
      const onLive = !!text && insideSelection(e.clientX, e.clientY);
      if (!onLive && !(hasMarks() && insideMarks(e.clientX, e.clientY))) return;
      // Kein neues Markieren und kein natives Ziehen: Archiver zieht selbst.
      e.preventDefault();
      e.stopImmediatePropagation();
      const p = hasMarks() ? collectPieces() : { text, caption: captionNear(), images: selectedImages() };
      textDrag = { x: e.clientX, y: e.clientY, text: p.text, caption: p.caption, images: p.images, moving: false, ghost: null, rect: spotRect ? { ...spotRect } : null };
      window.addEventListener("mousemove", onTextDragMove, true);
      window.addEventListener("mouseup", onTextDragUp, true);
      window.addEventListener("keydown", onTextDragKey, true);
    },
    true
  );

  window.addEventListener(
    "dragstart",
    (e) => {
      if (!e.isTrusted) return; // nur echte Eingaben, keine von der Seite erzeugten Events
      fallbackImageUrl = findBestImageUrl(e.target);
      dragCaption = captionNear();
      debugLog("dragstart, target:", e.target, "erkannte Quelle:", fallbackImageUrl);
    },
    true
  );

  // ---- Ziehen auf Seiten, die Bilder unter einer Fläche verstecken (Instagram u. a.) ----
  // Beim Drücken auf etwas Nicht-Ziehbares (Fläche über dem Bild, Bild mit
  // draggable="false", Karussell, Video): liegt darunter ein Bild oder Video,
  // wird das gedrückte Element kurz ziehbar. Bild: trägt die Bildadresse.
  // Video ohne Standbild: beim Ablegen wird das aktuelle Bild abfotografiert.
  let hiddenDrag = null;

  function clearHiddenDrag() {
    if (!hiddenDrag) return;
    if (hiddenDrag.prev == null) hiddenDrag.el.removeAttribute("draggable");
    else hiddenDrag.el.setAttribute("draggable", hiddenDrag.prev);
    hiddenDrag = null;
  }

  function videoAt(x, y) {
    for (const el of document.elementsFromPoint(x, y)) {
      const v = el.tagName === "VIDEO" ? el : el.querySelector && [...el.querySelectorAll("video")].find((n) => {
        const r = n.getBoundingClientRect();
        return x >= r.left && x <= r.right && y >= r.top && y <= r.bottom;
      });
      if (v) return v;
    }
    return null;
  }

  function bigControl(el) {
    const r = el.getBoundingClientRect();
    return r.width >= 160 && r.height >= 160;
  }

  const SLIDER_ROLES = "[role='slider'], [role='scrollbar'], [role='progressbar'], [role='spinbutton']";
  const CONTROL_ROLES =
    "a[href], label, summary, [role='button'], [role='link'], [role='menuitem'], [role='tab'], [role='checkbox'], [role='switch'], [role='option']";

  // Bild unter der gedrückten Fläche, wenn es sie wirklich deckt (3.31.3). Nein
  // bei Karten aus vielen Kacheln (Google Maps, OpenStreetMap), bei Text und
  // Zeichenflächen auf einem Hintergrundbild und bei schmalen Griffen über einem
  // Bild (Vorher/Nachher-Regler): Dort gehört das Ziehen der Seite.
  function coveredImage(t, x, y) {
    for (const el of document.elementsFromPoint(x, y)) {
      if (el === document.documentElement || el === document.body) continue;
      let src = null;
      if (el.tagName === "IMG" || el.tagName === "PICTURE" || (el.tagName === "VIDEO" && el.poster)) src = el;
      else if (bgImageOf(el) && !(el !== t && el.contains(t))) src = el; // Hintergrund einer Hülle zählt nicht
      else if (el.querySelector)
        src = [...el.querySelectorAll("img")].find((i) => {
          const r = i.getBoundingClientRect();
          return x >= r.left && x <= r.right && y >= r.top && y <= r.bottom;
        });
      if (!src) continue;
      if (!covers(t, src)) return null;
      return src === el && el.tagName === "VIDEO" ? el.poster : src.tagName === "IMG" || src.tagName === "PICTURE" ? findBestImageUrl(src) : bgImageOf(src);
    }
    return null;
  }

  // Sichtbare Schnittfläche: mindestens die Hälfte der Fläche und des Bildes
  function covers(a, b) {
    const ra = a.getBoundingClientRect();
    const rb = b.getBoundingClientRect();
    const vis = (l, t, r, btm) => Math.max(0, Math.min(r, innerWidth) - Math.max(l, 0)) * Math.max(0, Math.min(btm, innerHeight) - Math.max(t, 0));
    const both = vis(Math.max(ra.left, rb.left), Math.max(ra.top, rb.top), Math.min(ra.right, rb.right), Math.min(ra.bottom, rb.bottom));
    return both > 0 && both >= 0.5 * vis(ra.left, ra.top, ra.right, ra.bottom) && both >= 0.5 * vis(rb.left, rb.top, rb.right, rb.bottom);
  }

  function armHiddenDrag(e) {
    if (!e.isTrusted) return; // nur echte Eingaben, keine von der Seite erzeugten Events
    clearHiddenDrag();
    if (e.button !== 0 || e.altKey || manualOpen || textDrag || gone()) return;
    if (hasMarks() && insideMarks(e.clientX, e.clientY)) return; // zieht alle Stücke (Mehrfachmarkierung)
    const t = e.target;
    if (!(t instanceof Element)) return;
    const ctl = t.closest("input, textarea, select, button, [contenteditable=''], [contenteditable='true']");
    // Instagram-Feed: über dem Video liegt ein großer Knopf („Ton stummschalten“).
    // Den wie eine Fläche behandeln, kleine Bedienknöpfe bleiben Knöpfe.
    if (ctl && !(IG && ctl.tagName === "BUTTON" && bigControl(ctl))) return;
    // Regler und Bedienelemente der Seite bleiben bedienbar (3.29.5): Lautstärke,
    // Zeitleiste, Knöpfe. Links und Knöpfe ohne <button> ebenso, außer auf
    // Instagram, wo ein Link über dem Video liegt.
    if (t.closest(SLIDER_ROLES)) return;
    if (!IG && t.closest(CONTROL_ROLES)) return;
    // Kleine Flächen (Reglerknopf, Symbol) sind keine Hülle über einem Bild
    const tr = t.getBoundingClientRect();
    if (tr.width < 80 || tr.height < 80) return;
    // Echt ziehbare Bilder und Links laufen wie bisher
    if (t.closest("[draggable='true']") || (t.tagName === "IMG" && t.getAttribute("draggable") !== "false")) return;
    if (selectedText() && insideSelection(e.clientX, e.clientY)) return;
    const video = videoAt(e.clientX, e.clientY);
    const url = video && video.poster && !IG ? video.poster : video ? null : IG ? imageAt(e.clientX, e.clientY) : coveredImage(t, e.clientX, e.clientY);
    if (!url && !video) return;
    if (IG) return startMediaDrag(e, t, url, url ? null : video);
    hiddenDrag = { el: t, prev: t.getAttribute("draggable"), url, video: url ? null : video };
    t.setAttribute("draggable", "true");
    // Karussells verhindern sonst das Ziehen (eigene Wisch-Geste beim Drücken)
    e.stopImmediatePropagation();
  }

  window.addEventListener("pointerdown", armHiddenDrag, true);
  window.addEventListener(
    "mousedown",
    (e) => {
      if (!e.isTrusted) return; // nur echte Eingaben, keine von der Seite erzeugten Events
      if (hiddenDrag && hiddenDrag.el === e.target) e.stopImmediatePropagation();
    },
    true
  );
  window.addEventListener("mouseup", () => setTimeout(clearHiddenDrag, 0), true);
  window.addEventListener(
    "dragend",
    () =>
      setTimeout(() => {
        clearHiddenDrag();
        dragSource = null;
        dragStarted = false;
        pageDrag = false;
      }, 0),
    true
  );

  window.addEventListener(
    "dragstart",
    (e) => {
      if (!e.isTrusted) return; // nur echte Eingaben, keine von der Seite erzeugten Events
      dragSource = e.target instanceof Element ? e.target : null;
      dragStarted = true;
      // Ein Element, das die Seite selbst ziehbar gemacht hat (Kanban-Karte,
      // sortierbare Liste, Datei in einer Web-App): Das Ziehen gehört der Seite (3.31.3)
      const own = dragSource && dragSource.closest("[draggable='true']");
      pageDrag = !!own && dragSource.tagName !== "IMG" && !(hiddenDrag && own === hiddenDrag.el);
      if (!hiddenDrag || !e.dataTransfer || !(e.target === hiddenDrag.el || hiddenDrag.el.contains(e.target))) return;
      e.dataTransfer.effectAllowed = "copy";
      if (hiddenDrag.video) {
        fallbackImageUrl = null;
        e.dataTransfer.setData("application/x-archiver-video", "1");
        e.dataTransfer.setData("text/plain", location.href);
        // Vorschau beim Ziehen: das sichtbare Videobild statt der grauen Fläche
        const frame = videoFrame(hiddenDrag.video);
        if (frame) {
          frame.style.cssText += "position:fixed;left:-10000px;top:0;pointer-events:none";
          document.documentElement.appendChild(frame);
          e.dataTransfer.setDragImage(frame, frame.width / 2, frame.height / 2);
          setTimeout(() => frame.remove(), 0);
        }
        return;
      }
      const url = resolveUrl(hiddenDrag.url);
      fallbackImageUrl = url;
      e.dataTransfer.setData("text/uri-list", url);
      e.dataTransfer.setData("text/plain", url);
      e.dataTransfer.setData("text/html", `<img src="${url.replace(/"/g, "&quot;")}">`);
      // Vorschau beim Ziehen: das Bild selbst statt der leeren Fläche
      const img = [...document.elementsFromPoint(e.clientX, e.clientY)].find((n) => n.tagName === "IMG");
      if (img) cappedDragImage(e, img);
    },
    true
  );

  // Chrome zeichnet ein <img> als Ziehbild in voller Pixelgröße (große WebP
  // wurden riesig). Stattdessen eine verkleinerte Kopie, höchstens 220 px.
  function cappedDragImage(e, img) {
    const r = img.getBoundingClientRect();
    const nw = img.naturalWidth || r.width;
    const nh = img.naturalHeight || r.height;
    const scale = Math.min(1, 220 / Math.max(nw, nh, 1));
    const w = Math.max(1, Math.round(nw * scale));
    const h = Math.max(1, Math.round(nh * scale));
    const canvas = document.createElement("canvas");
    canvas.width = w;
    canvas.height = h;
    canvas.style.cssText = `position:fixed;left:-10000px;top:0;width:${w}px;height:${h}px;pointer-events:none`;
    try {
      canvas.getContext("2d").drawImage(img, 0, 0, w, h);
    } catch {}
    document.documentElement.appendChild(canvas);
    const fx = r.width ? Math.min(1, Math.max(0, (e.clientX - r.left) / r.width)) : 0.5;
    const fy = r.height ? Math.min(1, Math.max(0, (e.clientY - r.top) / r.height)) : 0.5;
    e.dataTransfer.setDragImage(canvas, Math.round(w * fx), Math.round(h * fy));
    setTimeout(() => canvas.remove(), 0);
  }

  // Ziehbild für Bild und Video (3.28.0): das Motiv im eigenen Seitenverhältnis,
  // ohne Rundung, höchstens DRAG_PX an der langen Seite (wie cappedDragImage).
  // Video: das gerade sichtbare Bild, sonst das Poster.
  const DRAG_PX = 220;
  const GHOST_CSS =
    "position:fixed;left:0;top:0;z-index:2147483647;pointer-events:none;border-radius:0;display:block;" +
    "box-shadow:0 12px 32px rgba(0,0,0,.4);opacity:.92;";

  function fitDrag(w, h) {
    const scale = Math.min(1, DRAG_PX / Math.max(w, h, 1));
    return [Math.max(1, Math.round(w * scale)), Math.max(1, Math.round(h * scale))];
  }

  function videoFrame(video) {
    if (!video || video.readyState < 2 || !video.videoWidth) return null;
    const [w, h] = fitDrag(video.videoWidth, video.videoHeight);
    const canvas = document.createElement("canvas");
    canvas.width = w;
    canvas.height = h;
    canvas.style.cssText = `width:${w}px;height:${h}px;`;
    try {
      // Fremde Videos lassen die Fläche „tainted“; zeichnen und zeigen geht trotzdem
      canvas.getContext("2d").drawImage(video, 0, 0, w, h);
    } catch {
      return null;
    }
    return canvas;
  }

  function mediaGhost(d) {
    const frame = d.video ? videoFrame(d.video) : null;
    if (frame) {
      frame.style.cssText += GHOST_CSS;
      return frame;
    }
    const src = d.url || (d.video && d.video.poster);
    const r = (d.video || d.el).getBoundingClientRect();
    const [w, h] = fitDrag(r.width || DRAG_PX, r.height || DRAG_PX);
    if (src) {
      const img = document.createElement("img");
      img.style.cssText = GHOST_CSS + `width:${w}px;height:${h}px;object-fit:cover;background:#222;`;
      img.onload = () => {
        const [nw, nh] = fitDrag(img.naturalWidth, img.naturalHeight);
        img.style.width = nw + "px";
        img.style.height = nh + "px";
      };
      img.src = src;
      return img;
    }
    const box = document.createElement("div");
    box.style.cssText = GHOST_CSS + `width:${w}px;height:${h}px;background:#222;`;
    return box;
  }

  // Instagram: eigenes Ziehen mit Zeiger-Ereignissen statt des Browser-Ziehens,
  // das Instagram unterbindet. Erst ab ein paar Pixeln Bewegung, sonst bleibt
  // es ein normaler Klick.
  let mediaDrag = null;

  // Instagram legt im Feed einen Link (mit „Ton stummgeschaltet“) über das
  // Video. Chrome würde sonst diesen Link ziehen statt des Videos: solange
  // Archiver zieht, sind Link und gedrücktes Element nicht nativ ziehbar.
  function blockNativeDrag(el) {
    const els = [el, el.closest && el.closest("a[href]")].filter((n, i, a) => n && a.indexOf(n) === i);
    return els.map((n) => ({ n, prev: n.getAttribute("draggable") })).map((x) => (x.n.setAttribute("draggable", "false"), x));
  }

  function restoreNativeDrag(list) {
    for (const { n, prev } of list || []) prev == null ? n.removeAttribute("draggable") : n.setAttribute("draggable", prev);
  }

  function stopNativeDrag(e) {
    if (mediaDrag) e.preventDefault();
  }

  function startMediaDrag(e, el, url, video) {
    mediaDrag = { x: e.clientX, y: e.clientY, el, url, video, moving: false, ghost: null, blocked: blockNativeDrag(el) };
    window.addEventListener("dragstart", stopNativeDrag, true);
    window.addEventListener("pointermove", onMediaDragMove, true);
    window.addEventListener("pointerup", onMediaDragUp, true);
    window.addEventListener("keydown", onMediaDragKey, true);
  }

  function endMediaDrag() {
    if (!mediaDrag) return;
    if (mediaDrag.ghost) mediaDrag.ghost.remove();
    restoreNativeDrag(mediaDrag.blocked);
    window.removeEventListener("dragstart", stopNativeDrag, true);
    window.removeEventListener("pointermove", onMediaDragMove, true);
    window.removeEventListener("pointerup", onMediaDragUp, true);
    window.removeEventListener("keydown", onMediaDragKey, true);
    if (mediaDrag.moving) removeZones();
    mediaDrag = null;
  }

  function onMediaDragMove(e) {
    if (!e.isTrusted) return; // nur echte Eingaben, keine von der Seite erzeugten Events
    const d = mediaDrag;
    if (!d.moving) {
      if (Math.hypot(e.clientX - d.x, e.clientY - d.y) < 6) return;
      d.moving = true;
      dragCounter = 1;
      loadDragLibs();
      showZones();
      d.ghost = mediaGhost(d);
      document.documentElement.appendChild(d.ghost);
      // in die Überlagerung, damit das Bild über den Zonen liegt (wie beim Text)
      ensureOverlay().then(() => {
        if (mediaDrag === d && d.ghost) rootEl.appendChild(d.ghost);
      });
      window.getSelection && window.getSelection().removeAllRanges();
    }
    e.preventDefault();
    e.stopImmediatePropagation();
    d.ghost.style.transform = `translate(${e.clientX + 12}px, ${e.clientY + 12}px)`;
    trackZones(e.clientX, e.clientY);
  }

  function onMediaDragUp(e) {
    if (!e.isTrusted) return; // nur echte Eingaben, keine von der Seite erzeugten Events
    const d = mediaDrag;
    if (!d.moving) return endMediaDrag();
    e.preventDefault();
    e.stopImmediatePropagation();
    // Den Klick nach dem Ziehen schlucken, sonst öffnet oder liked Instagram
    window.addEventListener(
      "click",
      (c) => {
        c.preventDefault();
        c.stopImmediatePropagation();
      },
      { capture: true, once: true }
    );
    const idx = getZoneIndex(e.clientX, e.clientY);
    if (idx !== -1) landZones(idx, { x: e.clientX, y: e.clientY }, d.video ? null : d.url);
    endMediaDrag();
    if (idx === -1) return;
    const name = barZones[idx];
    dropInstagram(d.el, d.video, d.url, isRootZoneName(name) ? null : name);
  }

  function onMediaDragKey(e) {
    if (!e.isTrusted) return; // nur echte Eingaben, keine von der Seite erzeugten Events
    if (e.key !== "Escape") return;
    e.preventDefault();
    endMediaDrag();
  }

  // Video abgelegt: hat das Video eine echte Dateiadresse, wird die Datei
  // gesichert. Instagram streamt (blob:): dort nimmt die Extension die MP4
  // mit Ton aus den Beitragsdaten, die Instagram selbst geladen hat (siehe
  // dropInstagram). Klappt das nicht, wird das aktuelle Bild fotografiert.
  async function dropHiddenVideo(folder) {
    const video = hiddenDrag.video;
    const src = video.currentSrc || video.src || "";
    clearHiddenDrag();
    removeZones();
    const source = currentPageSource();
    if (/^https?:/.test(src)) return saveMediaFile(src, folder, source);
    photographVideo(video, folder);
  }

  function photographVideo(video, folder) {
    const r = video.getBoundingClientRect();
    const x = Math.max(0, r.left);
    const y = Math.max(0, r.top);
    captureArea({ x, y, w: Math.min(window.innerWidth, r.right) - x, h: Math.min(window.innerHeight, r.bottom) - y }, folder);
  }

  // ---- Instagram: Bilder und Videos in Originalgröße aus den Beitragsdaten ----
  // Archiver fragt Instagram nicht selbst ab. ig-page.js liest mit, was
  // Instagram ohnehin in den Browser lädt (Feed, Beitrag, Reels, Stories),
  // und meldet die Medien hierher. Bei Karussells wird das gezogene Bild über
  // den Dateinamen, ein Video über seine Länge zugeordnet. Fehlt ein Beitrag,
  // gilt das angezeigte Bild in der größten Variante der Seite (srcset).
  const IG = /(^|\.)instagram\.com$/.test(location.hostname);
  let dragSource = null;
  let dragStarted = false; // das Ziehen hat auf dieser Seite begonnen
  let pageDrag = false; // ... mit einem Element, das die Seite selbst ziehbar gemacht hat

  // ---- Pinterest: Ziehen aus der Übersicht (3.28.4) ----
  // Jeder Pin ist ein Link um das Bild. Chrome zog darum den Link (Adresse am
  // Mauszeiger), und Pinterest hielt das Ziehen für einen Upload und öffnete
  // „Bild zur Suche hochladen“. Jetzt ist es ein Bild-Drag, und solange ein Drag
  // aus der Seite selbst läuft, bekommt Pinterest die Drag-Ereignisse nicht.
  // Dateien aus dem Finder (kein dragSource) erreichen die Seite wie bisher.
  const PIN = /(^|\.)pinterest\.[a-z.]+$/.test(location.hostname);
  if (PIN) {
    window.addEventListener(
      "dragstart",
      (e) => {
        if (!e.isTrusted || !e.dataTransfer || hiddenDrag || !(e.target instanceof Element)) return;
        const img = e.target.tagName === "IMG" ? e.target : e.target.querySelector && e.target.querySelector("img");
        if (!img || !e.target.closest("a[href]")) return;
        const url = resolveUrl(fallbackImageUrl || findBestImageUrl(img));
        if (!url) return;
        e.dataTransfer.clearData();
        e.dataTransfer.effectAllowed = "copy";
        e.dataTransfer.setData("text/uri-list", url);
        e.dataTransfer.setData("text/plain", url);
        e.dataTransfer.setData("text/html", `<img src="${url.replace(/"/g, "&quot;")}">`);
        cappedDragImage(e, img);
      },
      true
    );
    for (const type of ["dragenter", "dragover", "dragleave", "drop"]) {
      window.addEventListener(
        type,
        (e) => {
          if (e.isTrusted && dragSource && !manualOpen) e.stopPropagation(); // offenes Overlay: Karten nehmen selbst an
        },
        true
      );
    }
  }
  const igMedia = new Map();
  const IG_MEDIA_MAX = 3000;

  // Nur Adressen von Instagrams Bildservern übernehmen; die Seite selbst
  // könnte das Ereignis sonst mit fremden Adressen füttern.
  function igCdnUrl(u) {
    try {
      const url = new URL(u);
      return url.protocol === "https:" && /(^|\.)(cdninstagram\.com|fbcdn\.net)$/.test(url.hostname);
    } catch {
      return false;
    }
  }

  function igClean(m) {
    if (!m || typeof m !== "object") return null;
    const urls = (list) => (Array.isArray(list) ? list.filter((v) => v && igCdnUrl(v.url)).map((v) => ({ url: v.url })) : []);
    const out = {
      code: typeof m.code === "string" ? m.code : null,
      pk: typeof m.pk === "string" ? m.pk : null,
      product_type: typeof m.product_type === "string" ? m.product_type : null,
      video_duration: typeof m.video_duration === "number" ? m.video_duration : null,
      video_versions: urls(m.video_versions),
      image_versions2: { candidates: urls(m.image_versions2 && m.image_versions2.candidates) },
      user: m.user && typeof m.user.username === "string" ? { username: m.user.username } : null,
      carousel_media: Array.isArray(m.carousel_media) ? m.carousel_media.map(igClean).filter(Boolean) : [],
    };
    if (!out.video_versions.length) out.video_versions = null;
    return out;
  }

  function igRemember(key, item) {
    if (!key) return;
    const prev = igMedia.get(key);
    // Ein vollständigerer Eintrag (mit Video oder Karussell) gewinnt
    if (prev && !item.video_versions && !item.carousel_media.length && (prev.video_versions || prev.carousel_media.length)) return;
    igMedia.delete(key);
    igMedia.set(key, item);
    if (igMedia.size > IG_MEDIA_MAX) igMedia.delete(igMedia.keys().next().value);
  }

  if (IG) {
    document.addEventListener("archiver:ig-media", (e) => {
      let list;
      try {
        list = JSON.parse(e.detail);
      } catch {
        return;
      }
      if (!Array.isArray(list)) return;
      for (const raw of list) {
        const item = igClean(raw);
        if (!item) continue;
        igRemember(item.code, item);
        igRemember(item.pk, item);
      }
    });
    document.dispatchEvent(new CustomEvent("archiver:ig-hello"));
  }

  function postCodeNear(el) {
    const loc = location.pathname.match(/\/(?:p|reel|reels|tv)\/([\w-]+)/);
    if (loc && !(el.closest && el.closest("article"))) return loc[1];
    for (let n = el; n && n !== document.body; n = n.parentElement) {
      const a = n.querySelector && n.querySelector('a[href*="/p/"], a[href*="/reel/"]');
      const hit = a && a.getAttribute("href").match(/\/(?:p|reel)\/([\w-]+)/);
      if (hit) return hit[1];
      if (n.tagName === "ARTICLE") break;
    }
    return loc ? loc[1] : null;
  }

  function mediaIdOf(code) {
    const abc = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let id = 0n;
    for (const ch of code.slice(0, 11)) id = id * 64n + BigInt(abc.indexOf(ch));
    return id.toString();
  }

  // Story: /stories/<name>/<Medien-ID>/ trägt die ID schon in der Adresse
  function storyOfPage() {
    const m = location.pathname.match(/\/stories\/([\w.]+)\/(\d+)/);
    return m && m[1] !== "highlights" ? { user: m[1], id: m[2] } : null;
  }

  const igItem = (code) => igMedia.get(code) || igMedia.get(mediaIdOf(code)) || null;

  // Kontoname aus dem Kopf des Beitrags, falls Instagram den Beitrag nicht mitgeliefert hat
  function igUserNear(el) {
    const art = el && el.closest ? el.closest("article") : null;
    if (!art) return null;
    for (const a of art.querySelectorAll("header a[href], a[href]")) {
      const m = (a.getAttribute("href") || "").match(/^\/([\w.]+)\/$/);
      if (m && !["explore", "reels", "stories", "p", "reel"].includes(m[1])) return m[1];
    }
    return null;
  }

  const fileNameOf = (url) => {
    try {
      return new URL(url, location.href).pathname.split("/").pop();
    } catch {
      return "";
    }
  };

  function igPick(item, video, imageUrl) {
    const list = item.carousel_media && item.carousel_media.length ? item.carousel_media : [item];
    const vid = (m) => (m.video_versions && m.video_versions.length ? m.video_versions[0].url : null);
    const img = (m) => m.image_versions2 && m.image_versions2.candidates && m.image_versions2.candidates[0] && m.image_versions2.candidates[0].url;
    if (video) {
      const vids = list.filter(vid);
      const match = vids.find((m) => m.video_duration && isFinite(video.duration) && Math.abs(m.video_duration - video.duration) < 0.6);
      const m = match || (vids.length === 1 ? vids[0] : null);
      if (m) return { url: vid(m), ext: "mp4" };
      // Foto-Story mit einem vorgeladenen Video daneben: das Foto nehmen
      if (vids.length || list.length !== 1 || !img(list[0])) return null;
      return { url: img(list[0]), ext: "jpg" };
    }
    const name = fileNameOf(imageUrl || "");
    const m =
      (name && list.find((x) => ((x.image_versions2 && x.image_versions2.candidates) || []).some((c) => fileNameOf(c.url) === name))) ||
      (list.length === 1 ? list[0] : null);
    return m && img(m) ? { url: img(m), ext: "jpg" } : null;
  }

  // Quelle ist der Beitrag selbst, nicht der Feed, auf dem gezogen wurde.
  function igSource(code, item, el) {
    const reel = item ? item.product_type === "clips" : /\/reels?\//.test(location.pathname);
    const user = (item && item.user && item.user.username) || igUserNear(el);
    return {
      pageUrl: `https://www.instagram.com/${reel ? "reel" : "p"}/${code}/`,
      pageTitle: user ? `@${user} · Instagram` : "Instagram",
    };
  }

  async function dropInstagram(el, video, imageUrl, folder) {
    clearHiddenDrag();
    removeZones();
    fallbackImageUrl = null;
    // Stories: Original aus den mitgelesenen Mediendaten statt eines Standbilds
    const story = storyOfPage();
    const code = story ? null : el ? postCodeNear(el) : null;
    const item = story ? igMedia.get(story.id) || null : code ? igItem(code) : null;
    const source = story
      ? { pageUrl: `https://www.instagram.com/stories/${story.user}/${story.id}/`, pageTitle: `@${story.user} · Instagram` }
      : code
        ? igSource(code, item, el)
        : currentPageSource();
    const hit = item ? igPick(item, video, imageUrl) : null;
    if (hit) {
      sendSave({ type: "download-image", url: hit.url, suggestedName: `instagram-${story ? story.id : code}-${fileNameOf(hit.url).slice(0, 40)}`.replace(/\.(jpe?g|mp4|webp|heic)$/i, "") + "." + hit.ext, folder, toDownloads: shiftDrop, ...source });
    } else if (video) {
      photographVideo(video, folder);
    } else if (imageUrl) {
      sendSave({ type: "download-image", url: resolveUrl(imageUrl), folder, toDownloads: shiftDrop, ...source });
    }
  }

  // ---- Drag & Drop auf der Seite ----

  function looksLikeImageDrop(dt) {
    if (dt.files && [...dt.files].some((f) => f.type.startsWith("image/"))) return true;
    if (/<img[\s>]/i.test(dt.getData("text/html") || "")) return true;
    const uri = dt.getData("text/uri-list") || dt.getData("text/plain") || "";
    return /\.(png|jpe?g|gif|webp|svg|bmp|avif)(\?|#|$)/i.test(uri.trim());
  }

  function resolveUrl(url) {
    try {
      return new URL(url, location.href).href;
    } catch {
      return url;
    }
  }

  // Priorität: beim dragstart ermittelte (möglichst hochauflösende) Quelle,
  // sonst was der Drop selbst mitbringt.
  function extractImageUrl(dataTransfer) {
    if (fallbackImageUrl) return resolveUrl(fallbackImageUrl);
    const result = extractDroppedImage(dataTransfer);
    return typeof result === "string" ? resolveUrl(result) : result;
  }

  // Shift beim Loslassen: Bild landet in ~/Downloads statt in der Zone.
  // Gibt es in der App mehrere Bibliotheken, wechselt ⇧ stattdessen die Bibliothek.
  let shiftDrop = false;

  // ---- Bibliothek beim Ziehen wechseln (3.29.0) ----
  // Gibt es mehrere Bibliotheken, steht beim Ziehen eine Pille über dem
  // Streifen. Einen Namen anfahren und kurz verweilen wechselt die Bibliothek,
  // die Zonen zeigen dann ihre Active Zones. ⇧ springt weiter zur nächsten
  // (gelesen aus dragover, Tastenereignisse gibt es während eines Drags nicht).
  const LIB_DWELL_MS = 400;
  let dragLibs = []; // [{ name, path, active }]
  let libIdx = -1;
  let shiftHeld = false;
  let dropLibrary = null; // Pfad der gewählten Bibliothek, null = aktive
  let libBarEl = null;
  let libTarget = -1; // angefahrener Name
  let libTimer = null;

  // Nach einem Update der Extension ist dieses Skript von ihr abgeschnitten:
  // Speichern geht dann ins Leere. Statt still nichts zu tun, sagen wir es (3.24.0).
  function sendSave(message) {
    const lost = () => {
      const own = t("extensionReloaded");
      const text = own !== "extensionReloaded" ? own : /^de/i.test(navigator.language) ? "Archery wurde aktualisiert. Bitte lade die Seite neu." : "Archery was updated. Please reload the page.";
      if (rootEl) UI.showToast(rootEl, text, false);
    };
    try {
      return chrome.runtime.sendMessage(dropLibrary ? { ...message, library: dropLibrary } : message).catch(lost);
    } catch {
      lost();
      return Promise.resolve();
    }
  }

  function loadDragLibs() {
    dropLibrary = null;
    libIdx = -1;
    shiftHeld = false;
    setLibTarget(-1);
    try {
      chrome.runtime.sendMessage({ type: "library-zones", library: null }, (r) => {
        if (chrome.runtime.lastError || !r || !r.app) return;
        dragLibs = Array.isArray(r.libraries) ? r.libraries : [];
        libIdx = Math.max(0, dragLibs.findIndex((l) => l.active));
        if (zonesBarEl) showLibBar();
      });
    } catch {} // verwaistes Skript nach einem Update: ohne Bibliotheken weiter
  }

  function selectLibrary(i) {
    if (i < 0 || i >= dragLibs.length || i === libIdx) return;
    libIdx = i;
    const lib = dragLibs[i];
    dropLibrary = lib.active ? null : lib.path;
    showLibBar();
    chrome.runtime.sendMessage({ type: "library-zones", library: lib.path }, (r) => {
      if (chrome.runtime.lastError || !r || !r.app || dragLibs[libIdx] !== lib) return;
      barZones = r.hot && r.hot.length ? r.hot : r.zones.slice(0, 4);
      relabelZones();
    });
  }

  function cycleLibrary() {
    if (dragLibs.length < 2) return;
    setLibTarget(-1);
    selectLibrary((libIdx + 1) % dragLibs.length);
  }

  function showLibBar() {
    if (!rootEl || !zonesBarEl || dragLibs.length < 2) return;
    if (!libBarEl) {
      libBarEl = UI.el("div", "lib-bar");
      rootEl.appendChild(libBarEl);
    }
    libBarEl.replaceChildren(
      ...dragLibs.map((l, i) => UI.el("span", "lib" + (i === libIdx ? " on" : "") + (i === libTarget ? " target" : ""), l.name))
    );
    libBarEl.hidden = false;
  }

  function removeLibBar() {
    setLibTarget(-1);
    if (libBarEl) libBarEl.remove();
    libBarEl = null;
  }

  // Angefahrener Name: Ring, nach LIB_DWELL_MS Wechsel
  function setLibTarget(i) {
    if (i === libTarget) return;
    libTarget = i;
    clearTimeout(libTimer);
    libTimer = null;
    if (libBarEl) [...libBarEl.children].forEach((el, j) => el.classList.toggle("target", j === i));
    if (i < 0 || i === libIdx) return;
    libTimer = setTimeout(() => {
      libTimer = null;
      if (libTarget !== i) return;
      libTarget = -1;
      selectLibrary(i);
    }, LIB_DWELL_MS);
  }

  // Zeiger über der Pille? Liefert true, solange er darauf steht.
  function trackLibs(x, y) {
    if (!libBarEl || libBarEl.hidden) return false;
    const box = libBarEl.getBoundingClientRect();
    if (x < box.left || x > box.right || y < box.top || y > box.bottom) {
      setLibTarget(-1);
      return false;
    }
    const i = [...libBarEl.children].findIndex((el) => {
      const r = el.getBoundingClientRect();
      return x >= r.left && x <= r.right;
    });
    setLibTarget(i === libIdx ? -1 : i);
    return true;
  }

  // Karten behalten, nur Namen tauschen (Leiste bleibt ruhig stehen)
  function relabelZones() {
    zoneEls.forEach((zone, i) => {
      const name = barZones[i];
      zone.hidden = name == null;
      if (name == null) return;
      const label = zone.querySelector(".zone-label");
      label.replaceChildren();
      const parts = String(name).split("/");
      if (parts.length > 1) label.appendChild(UI.el("span", "zone-parent", parts.slice(0, -1).join(" / ") + " / "));
      label.appendChild(document.createTextNode(parts[parts.length - 1]));
    });
  }

  // Dateien von außen (Finder, Schreibtisch) in den Browser gezogen: keine
  // Zonen, die Seite bekommt das Ablegen selbst (Upload-Felder). Zonen nur,
  // wenn das Ziehen auf dieser Seite begonnen hat (3.29.5).
  function foreignDrag(e) {
    return !dragSource && !hiddenDrag && !!e.dataTransfer && [...e.dataTransfer.types].includes("Files");
  }

  // Zonen von selbst nur für das, was Archery sichert und was auf der Seite
  // begonnen hat: Bild, Link, markierter Text (3.31.3). Ziehen, das der Seite
  // gehört, und Text oder Links von außen bekommen sie nur mit ⌥, wie die
  // Leiste am ganzen Mac (DESIGN.md 4.13). ⌥ darf auch erst beim Ziehen kommen.
  function openDragZones(e) {
    if (zonesOn || !(e.altKey || hiddenDrag || (dragStarted && !pageDrag))) return;
    zonesOn = true;
    loadDragLibs();
    showZones();
  }

  window.addEventListener(
    "dragenter",
    (e) => {
      if (!e.isTrusted) return; // nur echte Eingaben, keine von der Seite erzeugten Events
      if (gone()) return removeZones();
      if (!e.dataTransfer || manualOpen) return; // im offenen Overlay übernehmen die Zonen-Karten
      if (foreignDrag(e)) return;
      dragCounter++;
      openDragZones(e);
    },
    true
  );

  window.addEventListener(
    "dragover",
    (e) => {
      if (!e.isTrusted) return; // nur echte Eingaben, keine von der Seite erzeugten Events
      if (gone()) return;
      if (!e.dataTransfer || manualOpen || foreignDrag(e)) return;
      openDragZones(e);
      if (!zonesOn) return; // ohne Zonen bleibt das Ablegen ganz bei der Seite
      e.preventDefault(); // notwendig, damit "drop" überhaupt feuert
      e.dataTransfer.dropEffect = "copy";
      // Mehrere Bibliotheken: jeder Druck auf ⇧ springt zur nächsten
      if (dragLibs.length > 1) {
        if (e.shiftKey && !shiftHeld) {
          shiftHeld = true;
          cycleLibrary();
        } else if (!e.shiftKey) shiftHeld = false;
      } else if (zonesBarEl && zonesBarEl.classList.contains("to-downloads") !== e.shiftKey) {
        // Nur eine Bibliothek: mit Shift geht das Bild am Archiv vorbei in den Download-Ordner
        zonesBarEl.classList.toggle("to-downloads", e.shiftKey);
        const hint = t(e.shiftKey ? "dropDownloads" : "dropRelease");
        zoneEls.forEach((el) => (el.querySelector(".zone-hint").textContent = hint));
      }
      trackZones(e.clientX, e.clientY);
    },
    true
  );

  window.addEventListener(
    "dragleave",
    (e) => {
      if (manualOpen || foreignDrag(e)) return;
      dragCounter--;
      if (dragCounter <= 0) removeZones();
    },
    true
  );

  window.addEventListener(
    "drop",
    (e) => {
      if (!e.isTrusted) return; // nur echte Eingaben, keine von der Seite erzeugten Events
      if (gone()) return;
      if (manualOpen || foreignDrag(e) || !zonesOn) return;
      const zoneIdx = getZoneIndex(e.clientX, e.clientY);
      shiftDrop = dragLibs.length > 1 ? false : e.shiftKey;
      removeLibBar();
      debugLog("drop, zoneIdx:", zoneIdx, "types:", e.dataTransfer.types);

      if (zoneIdx === -1) {
        debugLog("Drop außerhalb aller Zonen, ignoriert.");
        removeZones();
        fallbackImageUrl = null;
        return;
      }

      const zoneName = barZones[zoneIdx];
      const folder = isRootZoneName(zoneName) ? null : zoneName;
      const shown = hiddenDrag ? (hiddenDrag.video ? null : hiddenDrag.url) : extractImageUrl(e.dataTransfer);
      landZones(zoneIdx, { x: e.clientX, y: e.clientY }, typeof shown === "string" ? shown : shown && URL.createObjectURL(shown.file));

      if (IG && (hiddenDrag || dragSource)) {
        const el = hiddenDrag ? hiddenDrag.el : dragSource;
        const video = hiddenDrag && hiddenDrag.video;
        const img = video ? null : fallbackImageUrl || (hiddenDrag && hiddenDrag.url);
        if (video || img) {
          e.preventDefault();
          e.stopPropagation();
          dropInstagram(el, video, img, folder);
          return;
        }
      }
      if (hiddenDrag && hiddenDrag.video) {
        e.preventDefault();
        e.stopPropagation();
        dropHiddenVideo(folder);
        return;
      }

      // Markierung mit Bild: Bild sichern, Text wird Bildtext
      const marked = selectedImages();
      const fromMark = !fallbackImageUrl || marked.some((i) => findBestImageUrl(i) === fallbackImageUrl);
      if (marked.length && fromMark && e.dataTransfer.getData("text/plain").trim()) {
        e.preventDefault();
        e.stopPropagation();
        saveSelectedImages(marked, folder);
        removeZones();
        fallbackImageUrl = null;
        return;
      }

      // Kein Bild im Spiel: Link, Adresszeile oder markierter Text
      if (!fallbackImageUrl && !looksLikeImageDrop(e.dataTransfer)) {
        const uri = (e.dataTransfer.getData("text/uri-list") || "").split("\n").find((l) => l && !l.startsWith("#"));
        const text = e.dataTransfer.getData("text/plain").trim();
        const link = (uri || (/^https?:\/\/\S+$/.test(text) ? text : "")).trim();
        if (link || text) {
          e.preventDefault();
          e.stopPropagation();
          const source = currentPageSource();
          if (link && link.split("#")[0] === location.href.split("#")[0]) {
            savePage(folder, source);
          } else if (link) {
            const html = e.dataTransfer.getData("text/html");
            const title = (html ? new DOMParser().parseFromString(html, "text/html").body.textContent : "").replace(/\s+/g, " ").trim();
            saveLink(link, title && title !== link ? title : "", folder);
          } else {
            saveCard(textCard(text, source), folder, { ...source, quote: text });
          }
          removeZones();
          return;
        }
      }

      const result = extractImageUrl(e.dataTransfer);
      debugLog("erkanntes Ergebnis:", result, "Ordner:", folder || "(Downloads-Root)");

      if (result) {
        e.preventDefault();
        e.stopPropagation();

        if (result.file) {
          const reader = new FileReader();
          reader.onload = () => {
            sendSave({ type: "download-image", url: reader.result, suggestedName: result.file.name, folder, toDownloads: shiftDrop });
          };
          reader.readAsDataURL(result.file);
        } else {
          sendSave({ type: "download-image", url: result, folder, toDownloads: shiftDrop, caption: dragCaption, ...currentPageSource() });
        }
      }
      removeZones();
      fallbackImageUrl = null;
    },
    true
  );

  window.addEventListener(
    "dragend",
    () => {
      fallbackImageUrl = null;
    },
    true
  );
  // Styles vorab holen (nur im Hauptfenster), damit die Leiste schon beim
  // ersten Ziehen sofort steht und nicht erst nachlädt
  if (window === window.top) (window.requestIdleCallback || setTimeout)(() => loadStyles().catch(() => {}), { timeout: 2000 });
})();

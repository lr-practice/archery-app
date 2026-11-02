// shared.js
// Gemeinsame Helfer für alle drei Oberflächen (Seiten-Overlay, Toolbar-Popup,
// Vollansicht). Wird im Manifest vor content.js als Content-Script geladen und
// in popup.html / dashboard.html vor dem jeweiligen Skript eingebunden.
// Stellt ein einziges globales Objekt bereit: ArchiverShared.

(function (global) {
  if (global.ArchiverShared) return; // in Frames nicht doppelt definieren

  const DEFAULT_ZONES = ["Projekt 1", "Projekt 2", "Projekt 3", "Downloads"];
  const DEFAULT_SHORTCUT = { alt: true, shift: true, ctrl: false, meta: false, key: "a" };

  // Übersetzter Text aus _locales (Fallback: der Schlüssel selbst).
  function t(key, subs) {
    try {
      const msg = chrome.i18n && chrome.i18n.getMessage(key, subs);
      if (msg) return msg;
    } catch {}
    return key;
  }

  // Diagnose-Ausgaben in der Konsole. Für Releases aus, damit Archiver auf
  // fremden Seiten nichts in deren Konsole schreibt.
  const DEBUG = false;
  function debugLog(...args) {
    if (DEBUG) console.log("[Archery]", ...args);
  }

  // Quelle der aktuellen Seite. In Frames wird, wenn erlaubt, die Top-Seite
  // genommen, sonst der Referrer (meist die einbettende Seite), sonst der Frame.
  function currentPageSource() {
    let pageUrl = location.href;
    let pageTitle = document.title || "";
    if (window.top !== window) {
      try {
        pageUrl = window.top.location.href;
        pageTitle = window.top.document.title || pageTitle;
      } catch {
        if (document.referrer) pageUrl = document.referrer;
      }
    }
    return { pageUrl, pageTitle };
  }

  // Bildunterschrift der Lightbox: Dateiname, Position, Link zur Quellseite.
  function renderLightboxCaption(el, item, index, count, linkClass) {
    el.textContent = "";
    const filename = (item.filename || "").split(/[\\/]/).pop() || "";
    el.appendChild(document.createTextNode(count > 1 ? `${filename} — ${index + 1}/${count}` : filename));
    if (!item.pageUrl) return;
    let host = item.pageUrl;
    try {
      host = new URL(item.pageUrl).hostname.replace(/^www\./, "");
    } catch {}
    const link = document.createElement("a");
    link.className = linkClass;
    link.href = item.pageUrl;
    link.target = "_blank";
    link.rel = "noopener noreferrer";
    link.title = item.pageTitle || item.pageUrl;
    link.textContent = host;
    link.addEventListener("click", (e) => e.stopPropagation());
    el.appendChild(document.createTextNode(" · "));
    el.appendChild(link);
  }

  // Mit der Desktop-App verbunden? Dann kommen die Zonen aus deren Bibliothek
  // und Bilder landen direkt dort (siehe background.js, Abschnitt Brücke).
  let appMode = false;

  // Zone "Downloads" (Groß-/Kleinschreibung egal) speichert ohne Unterordner.
  // Mit App gibt es diese Sonderzone nicht.
  function isRootZoneName(name) {
    return !appMode && String(name || "").trim().toLowerCase() === "downloads";
  }

  function folderKeyForName(name) {
    return isRootZoneName(name) ? "" : name;
  }

  // Zwei Initialen bei mehreren Wörtern, sonst die ersten zwei Zeichen.
  function monogram(name) {
    const words = String(name || "").trim().split(/\s+/).filter(Boolean);
    if (words.length >= 2) return (words[0][0] + words[1][0]).toUpperCase();
    return String(name || "?").slice(0, 2).toUpperCase();
  }

  // Zonenliste: mit App die Zonen der Bibliothek, sonst chrome.storage.sync
  // (Fallback: alter Key "projects"). cb(list, info), info.app = verbunden.
  function loadZones(cb) {
    if (chrome.runtime && chrome.runtime.sendMessage) {
      try {
        chrome.runtime.sendMessage({ type: "get-zones" }, (resp) => {
          void chrome.runtime.lastError;
          if (resp && resp.app && Array.isArray(resp.zones)) {
            appMode = true;
            cb(resp.zones.slice(), resp);
          } else {
            appMode = false;
            loadStoredZones(cb);
          }
        });
        return;
      } catch {}
    }
    loadStoredZones(cb);
  }

  function loadStoredZones(cb) {
    if (!chrome.storage || !chrome.storage.sync) {
      cb(DEFAULT_ZONES.slice());
      return;
    }
    chrome.storage.sync.get({ zones: null, projects: null }, (data) => {
      let list = data.zones;
      if (!Array.isArray(list) || !list.length) {
        list = Array.isArray(data.projects) && data.projects.length ? data.projects.filter(Boolean) : DEFAULT_ZONES.slice();
      }
      cb(
        list.map((z, i) => (z && String(z).trim() ? String(z).trim() : `Zone ${i + 1}`)),
        { app: false }
      );
    });
  }

  // Neue Reihenfolge der Zonen merken (mit App getrennt von der eigenen Liste).
  function saveZoneOrder(order) {
    // mit App legt die Übersicht der App die Reihenfolge fest
    if (!appMode) chrome.storage.sync.set({ zones: order });
  }

  // Ändert sich etwas an den Zonen (eigene Liste, App-Zonen, Reihenfolge)?
  function zonesChanged(changes, area) {
    return (area === "sync" && !!changes.zones) || (area === "local" && !!(changes.appZones || changes.appZoneOrder));
  }

  function formatShortcutLabel(combo) {
    const parts = [];
    if (combo.ctrl) parts.push("Ctrl");
    if (combo.alt) parts.push("Alt");
    if (combo.shift) parts.push("Shift");
    if (combo.meta) parts.push(navigator.platform.toLowerCase().includes("mac") ? "Cmd" : "Win");
    parts.push(combo.key.length === 1 ? combo.key.toUpperCase() : combo.key);
    return parts.join("+");
  }

  // Generisches Sortieren-per-Ziehen für Listen, deren Zeilen data-idx tragen.
  // handleSelector: CSS-Selektor des Griffs (im Overlay präfixiert, sonst ".drag-handle").
  // Nutzt Maus-Events statt der nativen HTML5-Drag-API, damit es sich nicht mit
  // dem Bild-Drag&Drop in die Quere kommt.
  function attachSortable(listEl, order, renderFn, handleSelector = ".drag-handle") {
    listEl.addEventListener("mousedown", (e) => {
      const handle = e.target.closest(handleSelector);
      if (!handle) return;
      const row = handle.closest("[data-idx]");
      if (!row) return;

      e.preventDefault();
      const draggedIndex = parseInt(row.dataset.idx, 10);
      let currentIndex = draggedIndex;
      const rows = Array.from(listEl.children);
      const rowHeight = row.getBoundingClientRect().height + 6; // + Zeilenabstand
      const startY = e.clientY;
      const prevUserSelect = document.body.style.userSelect;
      document.body.style.userSelect = "none";
      row.classList.add("dragging");

      function updateSiblingOffsets() {
        rows.forEach((r, i) => {
          if (r === row) return;
          let shift = 0;
          if (draggedIndex < currentIndex && i > draggedIndex && i <= currentIndex) shift = -rowHeight;
          else if (draggedIndex > currentIndex && i >= currentIndex && i < draggedIndex) shift = rowHeight;
          r.style.transform = shift ? `translateY(${shift}px)` : "";
        });
      }

      function onMove(ev) {
        const delta = ev.clientY - startY;
        row.style.transform = `translateY(${delta}px)`;
        let newIndex = draggedIndex + Math.round(delta / rowHeight);
        newIndex = Math.max(0, Math.min(order.length - 1, newIndex));
        if (newIndex !== currentIndex) {
          currentIndex = newIndex;
          updateSiblingOffsets();
        }
      }

      function onUp() {
        document.removeEventListener("mousemove", onMove);
        document.removeEventListener("mouseup", onUp);
        document.body.style.userSelect = prevUserSelect;
        if (currentIndex !== draggedIndex) {
          const [moved] = order.splice(draggedIndex, 1);
          order.splice(currentIndex, 0, moved);
        }
        renderFn();
      }

      document.addEventListener("mousemove", onMove);
      document.addEventListener("mouseup", onUp);
    });
  }

  // Bild aus einem Drop lesen: URL (uri-list, <img> im HTML, nackte Bild-URL)
  // oder { file } für eine vom Desktop gezogene Bilddatei.
  function extractDroppedImage(dataTransfer) {
    const uriList = dataTransfer.getData("text/uri-list");
    if (uriList) {
      const url = uriList.split("\n").find((l) => l && !l.startsWith("#"));
      if (url) return url.trim();
    }

    const html = dataTransfer.getData("text/html");
    if (html) {
      const match = html.match(/<img[^>]+src=["']([^"']+)["']/i);
      if (match) return match[1];
    }

    const plain = dataTransfer.getData("text/plain");
    if (plain && /^https?:\/\/.+\.(png|jpe?g|gif|webp|svg|bmp|avif)(\?.*)?$/i.test(plain.trim())) {
      return plain.trim();
    }

    if (dataTransfer.files && dataTransfer.files.length > 0) {
      const file = dataTransfer.files[0];
      if (file.type.startsWith("image/")) return { file };
    }

    return null;
  }

  // Statische Texte in HTML-Seiten übersetzen: data-i18n (Text),
  // data-i18n-title, data-i18n-placeholder.
  function applyI18n(root) {
    root.querySelectorAll("[data-i18n]").forEach((el) => (el.textContent = t(el.dataset.i18n)));
    root.querySelectorAll("[data-i18n-title]").forEach((el) => (el.title = t(el.dataset.i18nTitle)));
    root.querySelectorAll("[data-i18n-placeholder]").forEach((el) => (el.placeholder = t(el.dataset.i18nPlaceholder)));
  }

  // Einheitliche Fehlermeldung für Verschieben-Antworten aus background.js.
  function moveErrorText(resp) {
    return (resp && resp.error) || t("moveFailed");
  }

  global.ArchiverShared = {
    DEFAULT_ZONES,
    DEFAULT_SHORTCUT,
    isRootZoneName,
    folderKeyForName,
    monogram,
    loadZones,
    saveZoneOrder,
    zonesChanged,
    isAppMode: () => appMode,
    formatShortcutLabel,
    attachSortable,
    moveErrorText,
    debugLog,
    currentPageSource,
    renderLightboxCaption,
    t,
    extractDroppedImage,
    applyI18n,
  };
})(typeof globalThis !== "undefined" ? globalThis : window);

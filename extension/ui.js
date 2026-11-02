// ui.js
// Gemeinsame Oberflächen-Bausteine für Seiten-Overlay (im Shadow DOM),
// Vollansicht und Popup, damit alle drei identisch aussehen und sich gleich
// verhalten:
//   - createGallery: alle Zonen als Karten mit ihren Bildern. Klick = Lightbox,
//     Shift/Strg/Cmd-Klick = Auswahl, Ziehen = Verschieben, "+" = im Finder zeigen.
//   - buildToolbar: Filter und Größenregler über der Galerie.
//   - createPanelController / buildPanelButtons: Einstellungen und Information
//     als schwebendes Panel.
//   - buildSettingsPanel / buildInfoPanel: Inhalt dieser Panels (auch im Popup).
// Styles: gallery.css und panel.css.
//
// Das Verschieben zwischen Karten läuft bewusst über Maus-Events statt über die
// native Drag-API, weil sich native Bild-Drags nicht zuverlässig mit eigenen
// Nutzdaten kombinieren lassen. Stellt ein globales Objekt bereit: ArchiverUI.

(function (global) {
  if (global.ArchiverUI) return; // in Frames nicht doppelt definieren

  const {
    DEFAULT_SHORTCUT,
    folderKeyForName,
    monogram,
    loadZones,
    formatShortcutLabel,
    attachSortable,
    moveErrorText,
    renderLightboxCaption,
    extractDroppedImage,
    t,
  } = global.ArchiverShared;

  function el(tag, className, text) {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (text != null) node.textContent = text;
    return node;
  }

  function button(className, text, onClick) {
    const btn = el("button", className, text);
    btn.type = "button";
    if (onClick) btn.addEventListener("click", onClick);
    return btn;
  }

  // Zonenname; bei Unterzonen ("Kunden/Studio B") ist der Elternteil grau.
  function zoneNameEl(name) {
    const span = el("span", "zone-card-name");
    const parts = String(name).split("/");
    if (parts.length > 1) span.appendChild(el("span", "zone-parent", parts.slice(0, -1).join(" / ") + " / "));
    span.appendChild(document.createTextNode(parts[parts.length - 1]));
    return span;
  }

  // „heute“, „gestern“, sonst „10. Okt.“ wie in der App
  function dayText(ts) {
    const d = new Date(ts);
    const today = new Date();
    const start = (x) => new Date(x.getFullYear(), x.getMonth(), x.getDate()).getTime();
    const days = Math.round((start(today) - start(d)) / 864e5);
    if (days === 0) return t("dayToday");
    if (days === 1) return t("dayYesterday");
    const lang = (chrome.i18n && chrome.i18n.getUILanguage && chrome.i18n.getUILanguage()) || "de";
    return d.toLocaleDateString(lang.startsWith("de") ? "de-DE" : "en-GB", { day: "numeric", month: "short" }).replace(/\.\.$/, ".");
  }

  function setBodyUserSelect(value) {
    if (document.body) document.body.style.userSelect = value;
  }

  // ---- Toast ----

  function showToast(mount, text, ok) {
    const toast = el("div", "gallery-toast" + (ok === false ? " error" : ""), text);
    mount.appendChild(toast);
    requestAnimationFrame(() => toast.classList.add("visible"));
    setTimeout(() => {
      toast.classList.remove("visible");
      setTimeout(() => toast.remove(), 300);
    }, 2200);
  }

  // ---- Galerie ----
  // mount: Element für Lightbox, Auswahl-Hinweis, Drag-Ghost und Toasts.
  // root: document oder ShadowRoot (für elementFromPoint).
  // acceptExternalDrops: Bilder aus anderen Fenstern direkt auf Karten ablegen.

  function createGallery({ mount, root = document, acceptExternalDrops = false }) {
    const gridEl = el("div", "zones-grid");
    const hintEl = el("div", "selection-hint");
    mount.appendChild(hintEl);

    let zones = [];
    let cardRefs = {}; // folderKey -> { cardGrid, countEl }
    const selected = new Map(); // id -> { folderKey, cell }
    let query = "";
    let lightbox = null; // { el, img, caption, items, index }
    let focusKey = null; // folderKey der geöffneten Zone, null = alle Zonen
    let libName = null; // Name der Bibliothek in der App (null = ohne App)

    function render(list, meta) {
      if (list) zones = list.slice();
      if (meta !== undefined) libName = meta && meta.app ? meta.library || null : null;
      if (focusKey !== null && !zones.some((name) => folderKeyForName(name) === focusKey)) focusKey = null;
      clearSelection();
      gridEl.innerHTML = "";
      // Wie die App: Übersicht als Karten, eine geöffnete Zone als Raster.
      gridEl.classList.toggle("focused", focusKey !== null);
      gridEl.classList.toggle("cards", focusKey === null);
      cardRefs = {};
      if (focusKey === null) {
        // Kopf wie in der App: „Active Zones in Referenzen. …“, darunter das Etikett
        const head = el("h1", "x-head");
        head.appendChild(el("span", "x-head-title", t("activeTitle")));
        head.appendChild(el("span", "x-head-text", " " + (libName ? t("activeLead", [libName]) : t("activeLeadNoLib"))));
        gridEl.appendChild(head);
        const bar = el("div", "x-bar");
        bar.appendChild(el("span", "x-mono", t("activeLabel") + " (" + zones.length + ")"));
        gridEl.appendChild(bar);
      }
      zones.forEach((name, i) => {
        if (focusKey === null || folderKeyForName(name) === focusKey) gridEl.appendChild(buildCard(name, i));
      });
      focusListeners.forEach((fn) => fn(focusKey === null ? null : zones.find((n) => folderKeyForName(n) === focusKey)));
    }

    const focusListeners = [];
    function onFocusChange(fn) {
      focusListeners.push(fn);
    }

    // Klick auf den Kopf einer Zone zeigt nur diese Zone, "Alle Zonen" führt zurück.
    function setFocus(folderKey) {
      focusKey = folderKey;
      render();
    }

    function refreshFolder(folderKey) {
      const refs = cardRefs[folderKey];
      if (refs) loadFolder(folderKey, refs.cardGrid, refs.countEl, refs.cover);
    }

    function setFilter(value) {
      query = String(value || "").trim().toLowerCase();
      applyFilter();
    }

    function applyFilter() {
      gridEl.querySelectorAll(".zone-card-item").forEach((cell) => {
        const match = !query || cell.title.toLowerCase().includes(query);
        cell.classList.toggle("filtered-out", !match);
      });
    }

    // ---- Mehrfachauswahl ----

    function toggleSelect(item, cell, folderKey) {
      if (selected.has(item.id)) {
        selected.delete(item.id);
        cell.classList.remove("selected");
      } else {
        selected.set(item.id, { folderKey, cell });
        cell.classList.add("selected");
      }
      updateHint();
    }

    function clearSelection() {
      selected.forEach(({ cell }) => cell.classList.remove("selected"));
      selected.clear();
      updateHint();
    }

    function updateHint() {
      // Auswahl im Fokus wie in der App: die übrigen Bilder treten zurück
      gridEl.classList.toggle("has-selection", selected.size > 0);
      if (selected.size > 1) {
        hintEl.textContent = t("selectionHint", [String(selected.size)]);
        hintEl.classList.add("visible");
      } else {
        hintEl.classList.remove("visible");
      }
    }

    // ---- Zonen-Karte ----

    function buildCard(name, index) {
      const folderKey = folderKeyForName(name);

      const card = el("div", "zone-card");
      card.dataset.zoneIndex = String(index);
      card.dataset.folderKey = folderKey;

      const focused = focusKey !== null;
      const header = el("div", "zone-card-header");
      const handle = el("div", "zone-card-handle", "⋮⋮");
      handle.title = t("dragToReorderZone");
      handle.draggable = true;
      const title = el("div", "zone-card-title");
      title.appendChild(zoneNameEl(name));
      const countEl = el("div", "zone-card-count");
      title.appendChild(countEl);
      header.appendChild(title);
      if (!focused) header.appendChild(handle);

      // Übersicht: Titelbild (bis zu drei Bilder) über Name und Anzahl; Klick öffnet die Zone.
      const cover = focused ? null : el("div", "zone-card-cover");
      const cardGrid = el("div", "zone-card-grid");
      if (cover) {
        // Nummer (Taste 1–4) und Etikett über dem Cover, wie die Active Zones der App
        const key = el("div", "x-key");
        key.appendChild(el("span", "x-mono", String(index + 1).padStart(2, "0")));
        key.appendChild(el("span", "x-mono", t("activeTag")));
        card.appendChild(key);
        card.appendChild(cover);
        card.title = t("openZone");
        card.addEventListener("click", (e) => {
          if (!e.target.closest(".zone-card-handle")) setFocus(folderKey);
        });
      }
      card.appendChild(header);
      if (focused) card.appendChild(cardGrid);

      cardRefs[folderKey] = { cardGrid, countEl, cover };
      loadFolder(folderKey, cardGrid, countEl, cover);

      // Karten-Reihenfolge per nativer Drag-API am Griff (nur eine Zahl als Nutzlast).
      // Die neue Reihenfolge landet im Speicher, alle Oberflächen zeichnen daraufhin neu.
      const isCardDrag = (e) => e.dataTransfer.types.includes("application/x-archery-card");

      handle.addEventListener("dragstart", (e) => {
        e.stopPropagation();
        e.dataTransfer.effectAllowed = "move";
        e.dataTransfer.setData("application/x-archery-card", String(index));
      });

      card.addEventListener("dragenter", (e) => {
        if (acceptExternalDrops || isCardDrag(e)) e.preventDefault();
      });

      card.addEventListener("dragover", (e) => {
        if (isCardDrag(e)) {
          e.preventDefault();
          e.dataTransfer.dropEffect = "move";
          card.classList.add("reorder-target");
        } else if (acceptExternalDrops) {
          e.preventDefault();
          e.dataTransfer.dropEffect = "copy";
          card.classList.add("drop-active");
        }
      });

      card.addEventListener("dragleave", () => {
        card.classList.remove("drop-active", "reorder-target");
      });

      card.addEventListener("drop", (e) => {
        if (isCardDrag(e)) {
          e.preventDefault();
          card.classList.remove("reorder-target");
          const fromIndex = parseInt(e.dataTransfer.getData("application/x-archery-card"), 10);
          const toIndex = parseInt(card.dataset.zoneIndex, 10);
          if (!Number.isNaN(fromIndex) && !Number.isNaN(toIndex) && fromIndex !== toIndex) {
            const order = zones.slice();
            const [moved] = order.splice(fromIndex, 1);
            order.splice(toIndex, 0, moved);
            ArchiverShared.saveZoneOrder(order);
          }
          return;
        }
        if (!acceptExternalDrops) return;

        e.preventDefault();
        card.classList.remove("drop-active");
        const result = extractDroppedImage(e.dataTransfer);
        if (!result) return;

        const reload = () => setTimeout(() => refreshFolder(folderKey), 400);
        if (result.file) {
          const reader = new FileReader();
          reader.onload = () => {
            chrome.runtime.sendMessage(
              { type: "download-image", url: reader.result, suggestedName: result.file.name, folder: folderKey || null },
              reload
            );
          };
          reader.readAsDataURL(result.file);
        } else {
          chrome.runtime.sendMessage({ type: "download-image", url: result, folder: folderKey || null }, reload);
        }
      });

      return card;
    }

    function cardAt(x, y) {
      const hovered = root.elementFromPoint(x, y);
      return hovered && hovered.closest ? hovered.closest(".zone-card, .zone-chip") : null;
    }

    function clearDropHighlights() {
      mount.querySelectorAll(".drop-active").forEach((c) => c.classList.remove("drop-active"));
    }

    // In einer geöffneten Zone: beim Ziehen klappt unten eine Leiste mit allen
    // anderen Zonen auf, damit Bilder trotzdem verschoben werden können.
    let stripEl = null;
    function showZoneStrip(sourceFolderKey) {
      hideZoneStrip();
      stripEl = el("div", "zone-strip");
      stripEl.appendChild(el("span", "zone-strip-label", t("dropInto")));
      zones.forEach((name) => {
        const key = folderKeyForName(name);
        if (key === sourceFolderKey) return;
        const chip = el("div", "zone-chip");
        chip.dataset.folderKey = key;
        chip.appendChild(zoneNameEl(name));
        stripEl.appendChild(chip);
      });
      mount.appendChild(stripEl);
    }

    function hideZoneStrip() {
      if (stripEl) stripEl.remove();
      stripEl = null;
    }

    // Klick = Lightbox, Shift/Strg/Cmd-Klick = Auswahl, Ziehen = Verschieben.
    function attachImageMove(img, item, sourceFolderKey, cell, onClick) {
      img.addEventListener("mousedown", (e) => {
        if (e.button !== 0) return;
        e.preventDefault();

        const startX = e.clientX;
        const startY = e.clientY;
        let dragging = false;
        let ghost = null;
        const isGroupDrag = selected.has(item.id) && selected.size > 1;

        function onMove(ev) {
          if (!dragging && Math.hypot(ev.clientX - startX, ev.clientY - startY) > 6) {
            dragging = true;
            ghost = el("div", "drag-ghost");
            const ghostImg = el("img");
            ghostImg.src = img.src;
            ghost.appendChild(ghostImg);
            if (isGroupDrag) ghost.appendChild(el("span", "drag-ghost-badge", String(selected.size)));
            mount.appendChild(ghost);
            if (focusKey !== null) showZoneStrip(sourceFolderKey);
            setBodyUserSelect("none");
          }
          if (dragging && ghost) {
            ghost.style.left = `${ev.clientX + 12}px`;
            ghost.style.top = `${ev.clientY + 12}px`;
            clearDropHighlights();
            const card = cardAt(ev.clientX, ev.clientY);
            if (card && card.dataset.folderKey !== sourceFolderKey) card.classList.add("drop-active");
          }
        }

        function onUp(ev) {
          document.removeEventListener("mousemove", onMove);
          document.removeEventListener("mouseup", onUp);
          setBodyUserSelect("");
          clearDropHighlights();
          if (ghost) ghost.remove();
          const stripTarget = dragging ? cardAt(ev.clientX, ev.clientY) : null;
          hideZoneStrip();

          if (!dragging) {
            if (ev.shiftKey || ev.ctrlKey || ev.metaKey) toggleSelect(item, cell, sourceFolderKey);
            else onClick();
            return;
          }

          const card = stripTarget;
          if (!card) return;
          const targetFolderKey = card.dataset.folderKey;

          if (isGroupDrag) {
            const moves = Array.from(selected.entries()).filter(([, v]) => v.folderKey !== targetFolderKey);
            const touched = new Set([targetFolderKey]);
            let remaining = moves.length;
            clearSelection();
            moves.forEach(([id, v]) => {
              touched.add(v.folderKey);
              chrome.runtime.sendMessage({ type: "move-download", id, sourceFolder: v.folderKey, targetFolder: targetFolderKey }, (resp) => {
                remaining--;
                if (!(resp && resp.ok)) showToast(mount, moveErrorText(resp), false);
                if (remaining === 0) touched.forEach(refreshFolder);
              });
            });
            return;
          }

          if (targetFolderKey === sourceFolderKey) return;
          chrome.runtime.sendMessage(
            { type: "move-download", id: item.id, sourceFolder: sourceFolderKey, targetFolder: targetFolderKey },
            (resp) => {
              if (resp && resp.ok) {
                refreshFolder(targetFolderKey);
                refreshFolder(sourceFolderKey);
              } else {
                showToast(mount, moveErrorText(resp), false);
              }
            }
          );
        }

        document.addEventListener("mousemove", onMove);
        document.addEventListener("mouseup", onUp);
      });
    }

    // Vorschau aus der App erst holen, wenn die Karte ins Bild kommt (3.24.0).
    // Das Ergebnis bleibt am Eintrag, damit die Lightbox es wiederverwendet.
    const previewWaiting = new Map(); // img -> item
    const previewSeen = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (!entry.isIntersecting) continue;
          const img = entry.target;
          const item = previewWaiting.get(img);
          previewSeen.unobserve(img);
          previewWaiting.delete(img);
          if (item) fetchPreview(item, (url) => (img.src = url));
        }
      },
      { rootMargin: "400px" }
    );
    function fetchPreview(item, done) {
      chrome.runtime.sendMessage({ type: "app-preview", path: item.preview, hash: item.hash }, (r) => {
        void chrome.runtime.lastError;
        const url = (r && r.url) || item.fallback;
        if (!url) return done("");
        item.url = url;
        item.preview = null;
        done(url);
      });
    }
    function showImage(img, item) {
      if (item.url || !item.preview) {
        img.src = item.url || item.fallback || "";
        return;
      }
      previewWaiting.set(img, item);
      previewSeen.observe(img);
    }

    function loadFolder(folderKey, cardGrid, countEl, cover) {
      chrome.runtime.sendMessage({ type: "list-folder-downloads", folder: folderKey }, (response) => {
        const items = (response && response.items) || [];
        const count = items.length === 1 ? t("objCountOne") : t("objCount", [String(items.length)]);
        const latest = Math.max(0, ...items.map((i) => i.ts || 0));
        if (!items.length) countEl.textContent = t("noImagesYet");
        else if (cover) countEl.textContent = latest ? count + " · " + dayText(latest) : count;
        else countEl.textContent = latest ? t("zoneLead", [count, dayText(latest)]) : count + ".";
        if (cover) {
          cover.innerHTML = "";
          cover.classList.toggle("trio", items.length >= 3);
          cover.hidden = !items.length;
          for (const item of items.slice(0, items.length >= 3 ? 3 : 1)) {
            const img = el("img");
            img.loading = "lazy";
            img.draggable = false;
            img.addEventListener("error", () => img.remove());
            showImage(img, item);
            cover.appendChild(img);
          }
          return;
        }
        cardGrid.innerHTML = "";

        if (!items.length) {
          cardGrid.appendChild(el("div", "zone-card-empty", t("noImagesYet")));
          return;
        }

        items.forEach((item, i) => {
          const cell = el("div", "zone-card-item");
          cell.title = (item.filename || "").split(/[\\/]/).pop() || "";

          const img = el("img");
          img.loading = "lazy";
          img.draggable = false; // eigenes Maus-basiertes Verschieben statt nativer Drag-API
          img.addEventListener("error", () => {
            cell.classList.add("broken");
            cell.appendChild(el("span", null, cell.title || t("imageUnavailable")));
          });
          showImage(img, item);
          cell.appendChild(img);

          const finderBtn = button("zoom-btn", "+", (e) => {
            e.stopPropagation();
            chrome.runtime.sendMessage({ type: "show-download", id: item.id });
          });
          finderBtn.title = t("showInFinder");
          cell.appendChild(finderBtn);

          attachImageMove(img, item, folderKey, cell, () => openLightbox(items, i));
          cardGrid.appendChild(cell);
        });

        applyFilter();
      });
    }

    // ---- Lightbox (Bild groß, Pfeiltasten blättern) ----

    function openLightbox(items, index) {
      closeLightbox();
      const backdrop = el("div", "lightbox-backdrop");
      backdrop.addEventListener("click", (e) => {
        if (e.target === backdrop) closeLightbox();
      });
      const img = el("img", "lightbox-img");
      const caption = el("div", "lightbox-caption");
      const prevBtn = button("lightbox-nav lightbox-prev", "‹", (e) => {
        e.stopPropagation();
        showLightboxItem(lightbox.index - 1);
      });
      const nextBtn = button("lightbox-nav lightbox-next", "›", (e) => {
        e.stopPropagation();
        showLightboxItem(lightbox.index + 1);
      });
      backdrop.appendChild(prevBtn);
      backdrop.appendChild(img);
      backdrop.appendChild(nextBtn);
      backdrop.appendChild(caption);
      backdrop.appendChild(button("lightbox-close", t("close"), () => closeLightbox()));
      mount.appendChild(backdrop);

      lightbox = { el: backdrop, img, caption, items, index };
      showLightboxItem(index);
    }

    function showLightboxItem(index) {
      if (!lightbox || !lightbox.items.length) return;
      const count = lightbox.items.length;
      lightbox.index = ((index % count) + count) % count;
      const item = lightbox.items[lightbox.index];
      const shown = lightbox;
      if (item.url || !item.preview) lightbox.img.src = item.url || item.fallback || "";
      else fetchPreview(item, (url) => lightbox === shown && shown.items[shown.index] === item && (shown.img.src = url));
      renderLightboxCaption(lightbox.caption, item, lightbox.index, count, "lightbox-source");
    }

    function closeLightbox() {
      if (lightbox) lightbox.el.remove();
      lightbox = null;
    }

    // Esc/Pfeiltasten für Lightbox und Auswahl. true = Taste verbraucht.
    function handleKey(e) {
      if (lightbox) {
        if (e.key === "Escape") closeLightbox();
        else if (e.key === "ArrowRight") showLightboxItem(lightbox.index + 1);
        else if (e.key === "ArrowLeft") showLightboxItem(lightbox.index - 1);
        else return false;
        return true;
      }
      if (e.key === "Escape" && selected.size) {
        clearSelection();
        return true;
      }
      if (e.key === "Escape" && focusKey !== null) {
        setFocus(null);
        return true;
      }
      return false;
    }

    function destroy() {
      closeLightbox();
      clearSelection();
      hintEl.remove();
      gridEl.remove();
    }

    return { el: gridEl, render, refreshFolder, setFilter, handleKey, destroy, setFocus, onFocusChange };
  }

  // ---- Navigation: Archiver (Home) › Zone ----
  // sizeTarget: Element, auf dem --thumb-size gesetzt wird.

  function buildToolbar(gallery, sizeTarget) {
    // Schwebende Navigation wie in der App: Archiver (Home) › Zone, dazu Einstellungen und Info.
    const toolbar = el("div", "gallery-toolbar");
    const home = button("nav-btn nav-home active", "Archery", () => gallery.setFocus(null));
    const crumb = el("span", "nav-crumb");
    toolbar.appendChild(home);
    toolbar.appendChild(crumb);
    gallery.onFocusChange((name) => {
      home.classList.toggle("active", !name);
      crumb.innerHTML = "";
      if (!name) return;
      crumb.appendChild(el("span", "nav-sep", "›"));
      const current = el("span", "nav-btn active");
      current.appendChild(zoneNameEl(name));
      crumb.appendChild(current);
    });

    // Bildgröße: kein Regler mehr in der Navigation, der gemerkte Wert gilt weiter.
    chrome.storage.local.get({ thumbSize: null, dashboardThumbSize: null }, (data) => {
      const stored = Number.isFinite(data.thumbSize) ? data.thumbSize : data.dashboardThumbSize;
      sizeTarget.style.setProperty("--thumb-size", `${Number.isFinite(stored) ? stored : 84}px`);
    });

    // Mit App: Hinweis, dass alles Weitere in der App liegt, ein Klick öffnet sie
    // Oben rechts wie der Bibliotheks-Reiter der App: Name der Bibliothek, Klick öffnet die App
    const appBadge = button("nav-btn nav-app", t("openApp") + " ↗", () => chrome.runtime.sendMessage({ type: "open-app" }));
    appBadge.title = t("openAppTitle");
    appBadge.hidden = true;
    // Eigenes Element neben der Navigation (die Leiste hat backdrop-filter, darin wäre fixed relativ zu ihr)
    const libTab = el("div", "x-lib-tab");
    libTab.appendChild(appBadge);
    toolbar.libTab = libTab;
    loadZones((list, info) => {
      appBadge.hidden = !(info && info.app);
      if (info && info.app && info.library) appBadge.textContent = info.library + " ↗";
    });
    return toolbar;
  }

  // ---- Panels ----

  // Kopf einer Seite: Titel weiß, Satz dahinter grau, beides in einer Größe.
  function pageHeader(title, lead, extras) {
    const header = el("div", "page-header");
    const text = el("h1", "page-title");
    text.appendChild(el("span", "page-title-main", title));
    text.appendChild(document.createTextNode(" "));
    text.appendChild(el("span", "page-title-lead", lead));
    header.appendChild(text);
    if (extras && extras.length) {
      const side = el("div", "page-extras");
      extras.forEach((node) => side.appendChild(node));
      header.appendChild(side);
    }
    return header;
  }

  // Block mit grauem Titel und Inhalt, wie eine Karte.
  function block(title) {
    const section = el("section", "block");
    if (title) section.appendChild(el("div", "block-title", title));
    return section;
  }

  // Zeile: links Titel (weiß) mit grauer Zweitzeile, rechts ein Wert.
  function infoRow(title, text, value) {
    const row = el("div", "block-row");
    const main = el("div", "block-row-main");
    main.appendChild(el("div", "block-row-title", title));
    if (text) main.appendChild(el("div", "block-row-text", text));
    row.appendChild(main);
    if (value) {
      const side = el("div", "block-row-value");
      if (typeof value === "string") side.textContent = value;
      else side.appendChild(value);
      row.appendChild(side);
    }
    return row;
  }

  function openChromePage(page) {
    chrome.runtime.sendMessage({ type: "open-chrome-page", page });
  }

  // Zonen + Tastenkombination. onClose: zeigt eine Kopfzeile mit "Schließen".
  // onRecordingChange(true/false): während eine neue Kombination aufgenommen wird.
  function buildSettingsPanel({ onClose, headerExtras, onRecordingChange } = {}) {
    let draftZones = [];
    // Mit App: Zonen aus appNames sind Ordner der Bibliothek. Sie lassen sich
    // hier nur sortieren; umbenennen und löschen geht in der App.
    let appInfo = null;
    let appNames = new Set();
    let shortcut = DEFAULT_SHORTCUT;
    let stopRecording = null;

    const panel = el("div", "panel-inner");
    if (onClose) panel.appendChild(pageHeader(t("settings"), t("settingsLead"), headerExtras));

    const zoneListEl = el("div", "zone-list");

    function renderZoneRows() {
      zoneListEl.innerHTML = "";
      draftZones.forEach((name, i) => {
        const fallbackName = t("zoneN", [String(i + 1)]);
        const row = el("div", "zone-row");
        row.dataset.idx = String(i);

        const handle = el("div", "drag-handle", "⋮⋮");
        handle.title = t("dragToReorder");
        const mono = el("div", "zone-monogram", monogram(name || fallbackName));

        const fromApp = !!appInfo && appNames.has(name);
        const input = el("input");
        input.type = "text";
        input.value = name;
        input.readOnly = !!fromApp;
        input.maxLength = 80;
        input.placeholder = fallbackName;
        input.addEventListener("input", () => {
          draftZones[i] = input.value;
          mono.textContent = monogram(input.value || fallbackName);
        });
        // Übernehmen beim Verlassen des Felds oder mit Enter, wie in der App
        input.addEventListener("change", () => persist());
        input.addEventListener("keydown", (e) => {
          if (e.key === "Enter") input.blur();
        });

        const swap = (a, b) => {
          [draftZones[a], draftZones[b]] = [draftZones[b], draftZones[a]];
          renderZoneRows();
          persist();
        };
        const upBtn = button("mini-btn", "▲", () => i > 0 && swap(i - 1, i));
        upBtn.title = t("moveUp");
        upBtn.disabled = i === 0;
        const downBtn = button("mini-btn", "▼", () => i < draftZones.length - 1 && swap(i, i + 1));
        downBtn.title = t("moveDown");
        downBtn.disabled = i === draftZones.length - 1;
        const removeBtn = button("mini-btn remove", "×", () => {
          if (draftZones.length <= 1 || fromApp) return;
          draftZones.splice(i, 1);
          renderZoneRows();
          persist();
        });
        removeBtn.title = t("removeZone");
        removeBtn.disabled = draftZones.length <= 1;
        if (fromApp) removeBtn.style.visibility = "hidden";

        const actions = el("div", "row-actions");
        // Mit App bestimmt die Übersicht der App Reihenfolge und Hot Zones
        if (appInfo) {
          handle.style.display = "none";
          mono.textContent = String(i + 1).padStart(2, "0");
          mono.classList.add("zone-no");
        } else {
          actions.appendChild(upBtn);
          actions.appendChild(downBtn);
        }
        actions.appendChild(removeBtn);
        row.appendChild(handle);
        row.appendChild(mono);
        row.appendChild(input);
        row.appendChild(actions);
        zoneListEl.appendChild(row);
      });
    }
    attachSortable(zoneListEl, draftZones, () => {
      renderZoneRows();
      persist();
    });

    const addZoneBtn = button("add-row", t("addZone"), () => {
      draftZones.push("");
      renderZoneRows();
      const inputs = zoneListEl.querySelectorAll("input");
      inputs[inputs.length - 1].focus();
    });

    const zonesHint = el("p", "hint");
    zonesHint.innerHTML = t("zonesHintHtml");
    const downloadsLink = zonesHint.querySelector("a");
    if (downloadsLink) {
      downloadsLink.href = "#";
      downloadsLink.addEventListener("click", (e) => {
        e.preventDefault();
        openChromePage("downloads");
      });
    }

    const shortcutRow = el("div", "zone-row shortcut-row");
    const shortcutDisplay = el("span", "shortcut-display");
    const changeBtn = button("mini-action", t("change"), () => {
      if (stopRecording) return;
      shortcutDisplay.textContent = t("pressKeys");
      shortcutDisplay.classList.add("listening");
      if (onRecordingChange) onRecordingChange(true);

      const onKeydown = (e) => {
        if (!e.isTrusted) return; // nur echte Tasten, keine von der Seite erzeugten Events
        e.preventDefault();
        e.stopPropagation();
        if (e.key === "Escape") {
          stopRecording();
          shortcutDisplay.textContent = formatShortcutLabel(shortcut);
          return;
        }
        if (["Alt", "Shift", "Control", "Meta"].includes(e.key)) return; // auf die eigentliche Taste warten

        const combo = {
          alt: e.altKey,
          shift: e.shiftKey,
          ctrl: e.ctrlKey,
          meta: e.metaKey,
          key: e.key.length === 1 ? e.key.toLowerCase() : e.key,
          code: e.code,
        };
        chrome.storage.sync.set({ shortcut: combo }, () => {
          shortcut = combo;
          shortcutDisplay.textContent = formatShortcutLabel(combo);
          if (stopRecording) stopRecording();
        });
      };
      stopRecording = () => {
        document.removeEventListener("keydown", onKeydown, true);
        shortcutDisplay.classList.remove("listening");
        stopRecording = null;
        if (onRecordingChange) onRecordingChange(false);
      };
      document.addEventListener("keydown", onKeydown, true);
    });
    shortcutRow.appendChild(shortcutDisplay);
    shortcutRow.appendChild(changeBtn);

    const shortcutHint = el("p", "hint");
    shortcutHint.innerHTML = t("shortcutHintHtml");
    const shortcutsLink = shortcutHint.querySelector("a");
    if (shortcutsLink) {
      shortcutsLink.href = "#";
      shortcutsLink.addEventListener("click", (e) => {
        e.preventDefault();
        openChromePage("shortcuts");
      });
    }

    // Jede Änderung gilt sofort (kein Speichern-Knopf), kurz bestätigt.
    const footer = el("div", "footer");
    const statusEl = el("div", "status-text");
    footer.appendChild(statusEl);
    let saving = Promise.resolve();
    function persist() {
      saving = saving.then(async () => {
        if (appInfo) {
          await saveAppZones();
          return;
        }
        const cleaned = draftZones.map((name, i) => (name && name.trim() ? name.trim() : t("zoneN", [String(i + 1)])));
        const focused = zoneListEl.contains(document.activeElement);
        draftZones.splice(0, draftZones.length, ...cleaned);
        if (!focused) renderZoneRows();
        await new Promise((res) => chrome.storage.sync.set({ zones: cleaned }, res));
        statusEl.textContent = t("saved");
        setTimeout(() => (statusEl.textContent = ""), 1500);
      });
      return saving;
    }

    // Mit App: neue Zonen als Ordner in der Bibliothek anlegen, Reihenfolge merken.
    async function saveAppZones() {
      const order = draftZones.filter((n) => appNames.has(n));
      for (const name of draftZones.filter((n) => !appNames.has(n))) {
        if (!name || !name.trim()) continue;
        const r = await new Promise((res) => chrome.runtime.sendMessage({ type: "create-app-zone", name: name.trim() }, res));
        if (r && r.ok) order.push(r.zone);
        else statusEl.textContent = t("appZoneFailed");
      }
      ArchiverShared.saveZoneOrder(order);
      draftZones.splice(0, draftZones.length, ...order);
      appNames = new Set(order);
      renderZoneRows();
      if (!statusEl.textContent) statusEl.textContent = t("saved");
      setTimeout(() => (statusEl.textContent = ""), 1500);
    }

    const zonesBlock = block(t("zones"));
    const zonesTitle = zonesBlock.querySelector(".block-title");
    zonesBlock.appendChild(zoneListEl);
    zonesBlock.appendChild(addZoneBtn);
    zonesBlock.appendChild(zonesHint);
    zonesBlock.appendChild(footer);
    panel.appendChild(zonesBlock);
    const shortcutBlock = block(t("shortcut"));
    shortcutBlock.appendChild(shortcutRow);
    shortcutBlock.appendChild(shortcutHint);
    panel.appendChild(shortcutBlock);
    const themeBlock = block(t("theme"));
    const themeRow = el("div", "zone-row shortcut-row theme-row");
    const themeButtons = [["system", t("themeSystem")], ["light", t("themeLight")], ["dark", t("themeDark")]].map(([id, label]) =>
      button("mini-action", label, () => {
        chrome.storage.sync.set({ theme: id });
        markTheme(id);
      })
    );
    function markTheme(id) {
      themeButtons.forEach((b, i) => b.classList.toggle("active", ["system", "light", "dark"][i] === id));
    }
    themeButtons.forEach((b) => themeRow.appendChild(b));
    chrome.storage.sync.get({ theme: "system" }, (d) => markTheme(d.theme));
    themeBlock.appendChild(themeRow);
    panel.appendChild(themeBlock);
    const appBlock = block(t("desktopApp"));
    const appRow = el("div", "zone-row shortcut-row");
    const appStatus = el("span", "shortcut-display", t("appConnected"));
    appRow.appendChild(appStatus);
    appRow.style.display = "none";
    appBlock.appendChild(appRow);
    const appHint = el("p", "hint");
    appHint.style.display = "none";
    appBlock.appendChild(appHint);
    const exportRow = el("div", "zone-row shortcut-row");
    const exportStatus = el("span", "shortcut-display", "archery-export.json");
    exportRow.appendChild(exportStatus);
    exportRow.appendChild(
      button("mini-action", t("exportForApp"), () => {
        exportStatus.textContent = "…";
        chrome.runtime.sendMessage({ type: "export-for-app" }, (r) => {
          exportStatus.textContent =
            r && r.ok ? t("exportDone", [String(r.count)]) : t("exportFailed");
        });
      })
    );
    appBlock.appendChild(exportRow);
    panel.appendChild(appBlock);
    const exportHint = el("p", "hint", t("exportHint"));
    appBlock.appendChild(exportHint);

    loadZones((list, info) => {
      appInfo = info && info.app ? info : null;
      appNames = new Set(appInfo ? list : []);
      const show = (node, visible) => (node.style.display = visible ? "" : "none");
      show(zonesHint, !appInfo);
      show(appRow, !!appInfo);
      show(appHint, !!appInfo);
      show(exportRow, !appInfo);
      show(exportHint, !appInfo);
      if (appInfo) {
        appStatus.textContent = t("appConnected", [appInfo.library || "Archery"]);
        appHint.textContent = t("appZonesHint");
      }
      // Mit App: nur die vier Active Zones; Zonen anlegen und ordnen geht in der App
      const shown = appInfo && Array.isArray(info.hot) && info.hot.length ? info.hot : list;
      draftZones.splice(0, draftZones.length, ...shown);
      addZoneBtn.style.display = appInfo ? "none" : "";
      zonesTitle.textContent = appInfo ? t("hotZones") : t("zones");
      renderZoneRows();
    });
    chrome.storage.sync.get({ shortcut: null }, (data) => {
      shortcut = data.shortcut && data.shortcut.key ? data.shortcut : DEFAULT_SHORTCUT;
      shortcutDisplay.textContent = formatShortcutLabel(shortcut);
    });

    function destroy() {
      if (stopRecording) stopRecording();
      panel.remove();
    }

    return { el: panel, destroy };
  }

  // Information: Wortmarke mit Claim, darunter Blöcke mit Zeilen, typografisch
  // reduziert (eine Größe, Weiß und Grau, feine Linien).
  function buildInfoPanel() {
    const panel = el("div", "panel-inner");
    panel.appendChild(pageHeader("Archery", t("aboutClaim")));

    const how = block(t("infoHow"));
    for (const key of ["Save", "Store", "View", "Sort", "Full"]) {
      how.appendChild(infoRow(t("info" + key + "Title"), t("info" + key + "Text")));
    }
    panel.appendChild(how);

    const keys = block(t("infoKeys"));
    const overlayKey = el("span", "", formatShortcutLabel(DEFAULT_SHORTCUT));
    keys.appendChild(infoRow(t("infoKeyOverlay"), null, overlayKey));
    keys.appendChild(infoRow(t("infoKeyBrowse"), null, "← →"));
    keys.appendChild(infoRow(t("infoKeyBack"), null, "Esc"));
    panel.appendChild(keys);
    chrome.storage.sync.get({ shortcut: null }, (data) => {
      if (data.shortcut && data.shortcut.key) overlayKey.textContent = formatShortcutLabel(data.shortcut);
    });

    panel.appendChild(el("div", "page-foot", "Archery " + chrome.runtime.getManifest().version));
    return { el: panel, destroy: () => panel.remove() };
  }

  // Schaltet zwischen Einstellungen, Information und geschlossen um.
  function createPanelController(hostEl, { settingsHeaderExtras, onRecordingChange } = {}) {
    let mode = null;
    let current = null;
    let leaving = null; // Inhalt, der gerade hinausgleitet
    const listeners = [];
    // Die Seite gleitet von rechts herein und schiebt die Übersicht nach links.
    // Die Klasse am Elternelement verschiebt Raster und Navigation.
    const setOpen = (open) => {
      hostEl.classList.toggle("open", open);
      if (hostEl.parentElement) hostEl.parentElement.classList.toggle("page-open", open);
    };

    function dropLeaving() {
      if (leaving) leaving.destroy();
      leaving = null;
    }

    function close() {
      if (!current) return;
      dropLeaving();
      leaving = current;
      current = null;
      mode = null;
      setOpen(false);
      const gone = leaving;
      setTimeout(() => {
        if (leaving === gone) dropLeaving();
      }, 260);
      listeners.forEach((fn) => fn(mode));
    }

    function toggle(nextMode) {
      const same = mode === nextMode;
      if (same) return close();
      if (current) current.destroy();
      current = null;
      dropLeaving();
      current =
        nextMode === "settings"
          ? buildSettingsPanel({ onClose: close, headerExtras: settingsHeaderExtras, onRecordingChange })
          : buildInfoPanel();
      hostEl.appendChild(current.el);
      hostEl.scrollTop = 0;
      setOpen(true);
      mode = nextMode;
      listeners.forEach((fn) => fn(mode));
    }

    return {
      toggle,
      close,
      get mode() {
        return mode;
      },
      onChange: (fn) => listeners.push(fn),
    };
  }

  // Knöpfe "Einstellungen" und "Information"; der aktive ist hervorgehoben.
  // "Archiver" schließt eine offene Seite und führt zur Übersicht.
  function buildPanelButtons(controller) {
    // Wie in der App: Einstellungen und Information liegen unter „Mehr“.
    // Ist eine Seite offen, steht sie als aktiver Schritt in der Navigation.
    const wrap = el("div", "panel-buttons");
    const crumbSep = el("span", "nav-sep", "›");
    const pageBtn = button("nav-btn active", "", () => controller.close());
    const moreBtn = button("nav-btn nav-more", "≡", () => (menu.hidden ? openMenu() : closeMenu()));
    moreBtn.title = t("more");
    moreBtn.setAttribute("aria-label", t("more"));
    const menu = el("div", "more-menu");
    menu.hidden = true;
    const item = (text, mode) => {
      const b = button("more-item", text, () => {
        closeMenu();
        controller.toggle(mode);
      });
      b.dataset.mode = mode;
      menu.appendChild(b);
    };
    item(t("settings"), "settings");
    item(t("information"), "info");
    function openMenu() {
      menu.hidden = false;
      moreBtn.classList.add("active");
      setTimeout(() => document.addEventListener("pointerdown", outside, true));
    }
    function closeMenu() {
      menu.hidden = true;
      moreBtn.classList.remove("active");
      document.removeEventListener("pointerdown", outside, true);
    }
    function outside(e) {
      if (!e.composedPath().includes(wrap)) closeMenu(); // auch im Shadow-DOM des Overlays
    }
    controller.onChange((mode) => {
      crumbSep.hidden = pageBtn.hidden = !mode;
      pageBtn.textContent = mode === "settings" ? t("settings") : mode === "info" ? t("information") : "";
      for (const b of menu.children) b.classList.toggle("on", b.dataset.mode === mode);
      const toolbar = wrap.closest(".gallery-toolbar");
      if (toolbar) toolbar.classList.toggle("page-open", !!mode);
    });
    crumbSep.hidden = pageBtn.hidden = true;
    queueMicrotask(() => {
      const home = wrap.closest(".gallery-toolbar")?.querySelector(".nav-home");
      if (home) home.addEventListener("click", () => controller.close());
    });
    wrap.append(crumbSep, pageBtn, moreBtn, menu);
    return wrap;
  }


  // Erscheinungsbild: Klasse "light" am Element (html oder Shadow-Host),
  // nach Einstellung oder System. Folgt Änderungen sofort.
  function watchTheme(target) {
    const query = global.matchMedia ? global.matchMedia("(prefers-color-scheme: dark)") : null;
    let choice = "system";
    const apply = () => target.classList.toggle("light", choice === "light" || (choice === "system" && !!query && !query.matches));
    if (!chrome.storage || !chrome.storage.sync) return apply();
    chrome.storage.sync.get({ theme: "system" }, (d) => {
      choice = d.theme;
      apply();
    });
    chrome.storage.onChanged.addListener((changes, area) => {
      if (area === "sync" && changes.theme) {
        choice = changes.theme.newValue || "system";
        apply();
      }
    });
    if (query) query.addEventListener("change", apply);
    watchRadius(target);
  }

  // Ecken wie in der App: Die Bibliothek meldet die Stufe (storage.local
  // "appRadius", von background.js). Ohne App gilt „Weich“ wie ab Werk in der App.
  function watchRadius(target) {
    const apply = (id) => {
      id = id || "soft";
      if (id !== "round") target.dataset.radius = id;
      else delete target.dataset.radius;
    };
    if (!chrome.storage || !chrome.storage.local) return;
    chrome.storage.local.get({ appRadius: null }, (d) => apply(d.appRadius));
    chrome.storage.onChanged.addListener((changes, area) => {
      if (area === "local" && changes.appRadius) apply(changes.appRadius.newValue);
    });
  }

  global.ArchiverUI = {
    watchTheme,
    el,
    button,
    showToast,
    block,
    createGallery,
    buildToolbar,
    buildSettingsPanel,
    buildInfoPanel,
    createPanelController,
    buildPanelButtons,
  };
})(typeof globalThis !== "undefined" ? globalThis : window);

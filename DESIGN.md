# Archery – Design-System

Stand: App 0.43.3 · Extension 3.31.2

Diese Datei hält fest, wie Archery aussieht und warum. Sie gilt für die Mac-App
(`app/src`) und die Extension (`extension/`, in Chrome und in Safari). Wer an der Oberfläche
arbeitet, Mensch oder KI, liest sie vorher. Die Werte stammen aus dem Code.
Wo Code und Datei auseinanderlaufen, gilt der Code, und die Datei wird im
selben Commit nachgezogen.

---

## 1. Prinzipien

Die Prinzipien gelten in dieser Reihenfolge. Kommen sie sich in die Quere,
gewinnt das obere.

1. **Das Bild zuerst.** Die Oberfläche tritt hinter die Referenzen zurück.
   Alle Grautöne sind neutral (R = G = B) und haben keinen Farbstich, damit
   Bilder unverfälscht wirken und auch weiße Bilder eine Kante haben.
2. **Slick und minimal, nicht techy.** Keine sichtbare Technik, keine
   Zusatzrahmen und keine Icons, wo ein Wort reicht. Lieber eine Fläche
   weniger als eine mehr.
3. **Eine Entscheidung statt einer Einstellung.** Neue Gestaltungsfragen werden
   entschieden und nicht an die Nutzenden weitergereicht. Bestehende Optionen
   wie Rundung oder Outline wachsen nicht weiter. Seit 0.30 fest: Navigation
   oben links, Ordner als Karten, Neu laden ohne Welle. Regler laufen
   immer in 5-%-Schritten.
4. **Eine Sprache für App und Extension.** Was in beiden Oberflächen vorkommt,
   sieht in beiden gleich aus. Beispiele sind die Glasleiste beim Ziehen, die
   Zonenkarten, Geist als Schrift und die Hell- und Dunkel-Werte.
5. **Ein Zustand, ein Zeichen.** Auswahl, Tastaturfokus, Tauschziel,
   Umsortierziel und Ablageziel zeigen alle denselben gestrichelten Ring
   (siehe 4.1). Für diese Zustände gibt es keine zweite Hervorhebung.
6. **Ehrlich zum Dateisystem.** Eine Zone ist ein Ordner und die Bibliothek ein
   normaler Ordner auf dem Mac. Die Oberfläche tut nicht so, als läge etwas
   woanders.

---

## 2. Tokens

Die Tokens sind CSS-Variablen auf `:root`. Schemata überschreiben sie über
Klassen und Attribute auf `html`:

| Schema | Selektor |
|---|---|
| Dunkel (Standard) | `:root` |
| Hell | `html.light` (setzt `app.js` je nach Einstellung) |
| Mac mit Fensterglas | `html.mac` (halbtransparent über der macOS-Unschärfe) |

Bei neuen Regeln gilt:

- Immer Tokens verwenden und keine Hex-Werte im Komponenten-Code.
- Transparente Flächen entstehen über `rgba(var(--ink-rgb), α)`. So drehen sie
  sich mit dem Schema automatisch um.

### 2.1 Farbe

| Token | Dunkel | Hell (0.25.0) | Zweck |
|---|---|---|---|
| `--bg-solid` | `#1a1a1a` (0.42.5, vorher `#1c1c1c`) | `#fafafa` (0.42.5, vorher `#f4f5f7`) | Grundfarbe des Fensters, gleich dem Tray (0.28) |
| `--win-alpha` | 0.72 | 0.78 | Deckkraft über dem Mac-Glas, Standard wie im Tray. Regler „Glas“ (Einstellungen › Darstellung, 40–100 % in 5er-Schritten, `archiver.glass`, i18n.js `applyGlass`) für App, Tray und Observer |
| `--bg` | aus `--bg-solid` × `--win-alpha` | dito | Fenster über dem Mac-Glas |
| `--ink` / `--ink-rgb` | `#ffffff` / `255,255,255` | `#111111` / `17,17,17` | Text und Basis aller Alpha-Flächen |
| `--ink-inverse` | `#0f0f0f` | `#f4f5f7` | Text auf `--ink`-Flächen (Primärknopf) |
| `--p-text` | `#ededed`, mit Melange `#f0f0ee` | `#111111` | Haupttext |
| `--p-dim` | `#8c8c8c`, mit Melange `#9d9d9b` | `#5e6064` | Sekundärtext, Meta |
| `--p-faint` | `#5e5e5e`, mit Melange `#666664` (0.42.5, vorher `#6d6d6b`) | `#b2b2b2` (0.42.5, vorher `#8f9196`; etwa 2:1 auf dem Grund, nur Nebentext) | Tertiärtext, inaktive Chips |
| `--p-rule` | ink × 0.10 | schwarz × 0.10 | Trennlinien |
| `--p-rule-strong` | ink × 0.55 | schwarz × 0.55 | betonte Linien |
| `--p-pill` | `rgba(52,52,52,.82)` | `#ebebeb` deckend (0.42.5) | Pillen und Navigationsglas |
| Karte | `rgba(150,150,150,.055)` (0.42.5, vorher ink × 0.055; Hover ink × 0.08) | `#f2f2f2` deckend (`--card-bg`, 0.42.5). Blöcke, Listen und Neu-Karte bleiben `#e9ebee` | Zonen- und Ordnerkarten, eine Stufe vom Grund abgesetzt |
| `--thumb-bg` | `#262626` | `#e3e5e8` | Platzhalter hinter Bildern |
| `--menu-bg` | `rgba(40,40,40,.97)` | `rgba(246,247,244,.98)` | Menüs, Zonenglas |
| `--sheet-bg` | `#1c1c1c` | `#eef0f2` (auch auf dem Mac) | Seitenblatt (Details) |
| `--lightbox-bg` | `rgba(0,0,0,.9)` | `rgba(243,244,241,.97)` | Lightbox |
| `--accent` / `--accent-rgb` | `#2d5aeb` | dito | Toast, aktive Systemzustände |
| `--glow-rgb` | `91,124,255` | dito | Glühen der Zielkarte beim Ablegen |
| `--danger` | `#e5484d` | dito | Fehler-Toast, Löschen |

Für die Farbe gilt:

- **Akzent sparsam.** Blau zeigt nur Systemrückmeldung (Toast, Glühen beim
  Ablegen). Auswahl und Fokus sind nie blau, sondern Ring
  (Prinzip 5).
- **Kein reines Weiß als Fläche.** Hell beginnt bei `#f4f5f7` (0.42.4, Lukas' Wahl „Kühl“:
  ein Hauch Blau, Flächen `#e9ebee`, Schrift `#111`; das warme `#f3f4f1` aus 0.42.2
  wirkte neben Bildern schmutzig). Tray, Viewer und Extension bleiben bei `#f4f4f4`. Reines Weiß
  bleibt Glas (Navigation, Zonenleiste) und Bildern vorbehalten.

### 2.2 Typografie

| Rolle | Schrift | Größe | Gewicht | Laufweite | Beispiel |
|---|---|---|---|---|---|
| Label, Chip | Geist Mono, Versalien | 11 px / 1.4 | 400 | +0.02em | Filter-Chips, Zähler, `.p-mono`, `.p-chip` (0.42.5, vorher Melange Medium 10 px) |
| Meta | Geist Mono, Versalien | 11 px | 400 | +0.02em | Anzahl · Datum unter Karten (`.p-card-meta`), Quelle und Datum auf Text-Karten, Notiz-Kennung und Fuß (0.42.5) |
| Zahl im Index | Melange | 12 px | 350 | 0 | `.p-num` (Ziffern tabellarisch) |
| UI | Melange | 13–13.5 px | 400 | 0 | Navigation, Menüs |
| Karten-Titel | Melange Semi-Light | 14 px | 350 | 0 | Zonenname auf der Karte |
| Index-Zeile | Melange Semi-Light | 19 px | 350 | −0.01em | Zonenname im Index |
| Seitenkopf | Melange Semi-Light | 30 px / 1.16 | 350 | −0.02em | alle Seiten und Einstellungen, Titel voll, Satz hell (`--p-faint`) im selben Absatz |
| Einstellungen | Melange Semi-Light | 20 px | 350 | −0.012em | Bereiche in der Vollbild-Liste |
| Notiz | Melange Semi-Light | Titel 28 px, Text 16 px / 1.5 | 350 | −0.02em / 0 | Lightbox-Notiz |
| Satz | Melange | 34 px / 1.14 | 350 | 0 | Konto, Sperre (`.p-say`) |

Für die Typografie gilt:

- Grundschrift ist seit 0.42.2 Melange, Typo-System „A Leise“ (`--sans`),
  seit 0.42.3 fest ohne Wahl im Labor. Semi-Light (350) trägt Köpfe und
  Inhalt, die Hierarchie entsteht über Grau statt über Größe. Etiketten, Chips
  und Daten sind seit 0.42.5 wieder Geist Mono (Variante B „Etikett-Mono“ aus
  Figma, Lukas 27.10.; `--p-mono`). Tray und Observer setzen fest Geist.
- Melange ist kommerziell (kimera, Freigabe für den internen Gebrauch) und
  liegt nicht in diesem Repo. Der Build holt die woff2-Schnitte 100, 300, 350
  (Semi-Light), 350 kursiv, 400, 400 kursiv, 500, 600 und 700 aus dem privaten
  Repo `lr-practice/archery-website` (Branch `claude/project-thread-85f33d`)
  (`fonts/melange/`) nach `app/src/fonts/melange/` und
  `extension/fonts/melange/` (beide in `.gitignore`). Fehlen sie, springt
  Geist ein. Text-Karten und Bildbeschriftungen der Extension behalten ihre
  eigenen Stapel.
- Die Grundschrift läuft mit Standard-Laufweite (`letter-spacing: 0`), auch in
  Köpfen und Sätzen. Nur Geist Mono in Versalien bekommt +0.02em, die
  Notiz-Blätter in Systemschrift behalten ihre eigenen Werte.
- Mono ist für Daten und Etiketten da, nicht für Fließtext.
- Gewichte sind 350 (Inhalt, Köpfe), 400 (Bedienung, Etiketten in Geist Mono), 500 und 600
  (Geist-Stand).
- Neue Größen werden auf diese Stufen gerundet. Im Code stecken heute auch
  10, 14, 16 und 19 px. Diese Größen werden bei Gelegenheit auf die Stufen
  gezogen und nicht neu eingeführt.

### 2.2a Schriftzug
- „Archery“ als Schriftzug (Navigation, Startbild, Kopf unter Information): das y steht in `<span class="wm-y">` mit `margin-left:
  0.035em` (+35/1000 em, Brand Guideline › Logo). Gilt für jede Größe und
  für Melange wie Geist. Fenstertitel und Fließtext bleiben ohne.

### 2.3 Form und Raum

| Token | Wert | Zweck |
|---|---|---|
| `--r-lg` / `--r-md` / `--r-sm` | 8 / 6 / 5 px | Karten, Menüs, kleine Flächen |
| Rundung „Eckig“ / „Weich“ / „Rund“ / „Sehr rund“ | `--r-lg` 0 / 4 / 8 / 14 px (`data-radius`) | Einstellung, keine weiteren Stufen. Gilt seit 0.30 auch in Tray, Zonenleiste und Extension (über die Bibliothek gemeldet). Ab Werk „Weich“ (0.40.4, vorher „Rund“), seit Extension 3.31.1 auch in der Extension ohne App; neue Entwürfe werden mit „Weich“ gestaltet (4 / 3 / 3 px) |
| Pille | 999 px | Navigation, Pillen |
| Objekt (Bild) | 0 | Bilder bleiben rechteckig, wie das Original |
| `--p-pad` | 40 px | Seitenrand |
| `--top-h` | 72 px | Höhe Kopfbereich |
| `--nav-h` / `--nav-rim` / `--nav-top` | 36 / 5 / 17 px | Navigationsleiste: Knopfhöhe, Innenrand, Oberkante = Oberkante der Fensterknöpfe |
| Abstand in der Leiste | 3 px | zwischen Navigationsknöpfen |
| `--card-cols` | 6 | Kartenraster der Übersicht |
| Karten-Format | 2:1 | Zonen- und Ordnerkarten (Ordner ein Sechstel breit) |

### 2.4 Glas

Glas ist das einzige Material neben der Fläche. Es kommt nur bei schwebenden
Ebenen vor: Navigation, Active-Zones-Leiste, Menüs und Ladeplatzhalter.

| | Dunkel | Hell |
|---|---|---|
| Unschärfe | `blur(24px) saturate(1.4)` | `blur(28px) saturate(0)` |
| Schatten | `0 14px 44px rgba(0,0,0,.5)` | `0 14px 44px rgba(0,0,0,.16)` |
| Kante | `inset 0 0 0 1px` ink × 0.07 | `inset 0 0 0 1px` weiß × 0.7 + schwarz × 0.08 auf 0.5 px |
| Am ganzen Mac (0.40.24) | `NSVisualEffectView` hinter jeder Karte, Material HUD (13) | Material Seitenleiste (7) |
| Deckkraft | `--zone-alpha`, eigener Regler „Zonen beim Ziehen“ (30–100 % in 5er-Schritten, `archiver.zoneGlass`, Standard 0.97). App, Zonenleiste und Extension (über `zones.json` „alpha“) | dito |

### 2.5 Bewegung

| Zweck | Dauer | Kurve |
|---|---|---|
| Hover (Farbe, Hintergrund) | 0.12 s | `ease` |
| Ein- und Ausblenden, Toast, Wenden in der Lightbox (je Hälfte) | 0.18 s | `ease` / `ease-out` |
| Seitenblatt, Gleiten | 0.28 s | `cubic-bezier(0.2, 0.7, 0.2, 1)` (`--sheet-ease`) |
| Zonenleiste rein | 0.14 s | `ease-out` |
| Zonenleiste raus | 0.28 s, 0.7 s verzögert | `ease`. In der Extension (3.29.8, vorher 0.2 s nach 0.3 s) ab dem Loslassen; am ganzen Mac und nach dem Ablegen in der App (0.40.24) 0.28 s, 0.7 s nach „Gesichert“ (0.40.22: 0.35 s wirkte sofort, 1.2 s zu lang), ohne Drop sofort; das Fenster mit dem Glas blendet gleich lang aus (`dropbar_fade`); bei einem Fehler 2.5 s später (4.13) |
| Miniatur, Ring und Rollen beim Ablegen (4.13) | 0.28 s | `--sheet-ease` |
| Ring „Umlaufend“ | 1.6 s je Runde | `linear` |

Für die Bewegung gilt:

- Bewegung bestätigt eine Handlung, sie schmückt nicht.
- Alles respektiert `prefers-reduced-motion: reduce`.
- Neue Dauern werden auf diese Stufen gelegt.

---

## 3. Oberflächen

**Observer** (`viewer.*`): seit 0.30 ebenfalls macOS-Glas, Grund
`#1c1c1c`/`#f4f4f4` × `--win-alpha`.

**App** (`app/src`): Fenster mit macOS-Glas, Navigation oben in der Titelzeile
auf Höhe der Fensterknöpfe, Bibliotheks-Reiter oben rechts. Startseite mit den
Zonen als Karten, Active Zones oben in jeder Übersicht. Passen keine vier
Karten nebeneinander, stehen sie 2 × 2 wie der Index (0.28).

**Extension** (`extension/`): Overlay im Shadow DOM auf fremden Seiten
(`overlay.css`, `:host`), Galerie im App-Layout (`gallery.css`), Panel
(`panel.css`). Sie folgt der App 1:1. Seit 3.20.0 gilt das für die Übersicht,
die Zone, die Glasleiste beim Ziehen, Geist und die Bibliothek oben rechts.
Safari nutzt denselben Code und sieht gleich aus (4.27).

**Am ganzen Mac** (`palette.*`, `dropbar.*`, `src-tauri/src/system.rs`):
schwebende Fenster über anderen Programmen, auch wenn das Archery-Fenster zu
ist. Sie nutzen dieselben Karten, Ringe und Glaswerte wie die App (4.12, 4.13).

---

## 4. Komponenten

Alle Angaben sind Klassen in `app.css`. Die Extension hat ihre eigenen Namen
(`zone-card`, `nav-btn` …), soll aber gleich aussehen.

### 4.1 Fokus-Ring (`::after` mit `--ring`)
- **Zweck:** Er ist das einzige Zeichen für Auswahl, Tastaturfokus,
  Tauschziel, Umsortierziel und Ablageziel.
- **Form:** Der Ring ist eine 1 px gestrichelte Linie (4/4), als SVG-Maske
  gezeichnet. Er folgt der Rundung:
  - `--ring-lg` für Karten mit Abstand (3 px)
  - `--ring-in` für bündige Karten
  - `--ring-0` für Objekte und Zeilen
- **Varianten:** „Ruhig“ (Standard) steht still. „Umlaufend“ dreht sich
  langsam im Uhrzeigersinn.
- **Nicht:** Keine `outline`, keine Akzentfarbe und kein zweiter Rahmen.

### 4.2 Navigation (`.p-nav`)
- **Zweck:** Wechsel zwischen Archery (Start), Zonen, Alle Objekte und Suche,
  dazu das Menü ≡.
- **Form:** Eine Glas-Pille oben, auf der Mittellinie der Fensterknöpfe.
  Aktive Knöpfe sind gefüllt.
- **Nicht:** Keine Icons neben Wörtern und keine zweite Leiste.

### 4.3 Active-Zones-Leiste (`.p-dropbar`, `.p-hot`)
- **Zweck:** Beim Ziehen fahren die vier ersten Zonen (01–04) als Glas ein.
- **Zustände:**
  - Einfahren
  - Ziel unter dem Zeiger (Ring)
  - Herkunftszone (Inhalt auf 45 % zurückgenommen, Glas bleibt)
  - Abgelegt (Miniatur wie in 4.13, dann fährt die Leiste aus; das frühere Glühen nur noch als Wahl unter Drop-Feedback, 4.24b)
- **Angeschnitten (0.35, Einstellungen › Darstellung, ab Werk „Ganz“):** Wahlweise
  ragen die Karten erst 90 px herein (`.p-dropbar.strip`), wie früher die Leiste
  der Extension (4.13): oben gerundet, unten bündig, ohne Hinweis. Zeiger in
  den Streifen → volle Höhe, mehr als 24 px darüber → wieder Streifen. Höhe mit
  0.22 s Übergang, ohne bei `prefers-reduced-motion`. Die Vorschau beim
  Glas-Regler zeigt immer die ganze Leiste.
- **Nicht:** Sie erscheint nicht ohne Ziehen und zeigt nicht mehr als vier
  Zonen.

### 4.4 Zonenkarte (`.p-card`)
- **Zweck:** Zone in der Übersicht, mit Cover (ein Bild oder drei), Name und
  Anzahl.
- **Form:** 2:1, `--r-lg`, Fläche ink × 0.055, beim Hover × 0.08.
  Umsortieren hebt die Karte an (Schatten, 85 %).
- **Nicht:** Keine Rahmenlinie. Die Karte setzt sich nur über die Fläche ab.
- **Neue Zone (0.39.1):** Die Karte zum Anlegen (Index und freier Platz bei den
  Active Zones) zeigt nur ein Plus in der Mitte, ein Viertel der Kartenhöhe,
  Linien 1 px in `--dash` wie die gestrichelte Outline, beim Hover in
  Textfarbe. Kein Text; „Neue Zone“ steht im Tooltip.
- **Bibliotheken (0.36.0):** vier Plätze in Einstellungen › Bibliothek, ⌘1–4.
  Ein leerer Platz bietet „Ordner wählen…“ und „Neue Bibliothek…“ (Ort und Name
  im Sichern-Dialog des Finders, der Ordner wird angelegt und gleich geöffnet).
- **Externe Zone (0.30.1):** Ordner außerhalb der Bibliothek, angelegt nur in
  Einstellungen › Bibliothek (Verknüpfung im Bibliotheksordner). Sieht aus wie
  jede Zone; ✕ löst nur die Verknüpfung. Seit 0.39.1 heißt das überall
  „Trennen“ (⋯-Menü, Einstellungen, Rückfrage mit `.btn primary` statt rot),
  und Ordner oder Unterzonen in einer externen Zone haben kein Löschen: Archery
  löscht nie etwas außerhalb der Bibliothek. Seit 0.31.0 gelten dieselben Schranken
  wie für eine Bibliothek (kein Benutzer- oder Systemordner, höchstens 20.000
  Dateien), und verschachtelte Ziele werden abgelehnt. Ist das Ziel nicht
  erreichbar, bleibt die Zone mit ihren Vorschauen stehen; in den Einstellungen
  steht hinter dem Namen „· offline“.

### 4.5 Ordnerkarte (`.p-folder-card`)
- **Zweck:** Unterordner neben dem Zonen-Titel als Karten (3/6, die Liste
  entfällt seit 0.30). Klick betritt den Ordner (Titel wird zum Pfad), die Chips
  links filtern nur und lassen Titel und Karten stehen.
- **Form:** Wie die Zonenkarte, ein Sechstel breit. Aktiv zeigt sie den Ring.
  Auch leere Ordner haben eine Karte (0 Objekte).
- **Ablageziel (0.27):** Objekte und Ordner lassen sich auf die Karte ziehen,
  sie zeigt dabei den Ring (4.1).
- **⋯ und Ziehen (0.28):** ⋯ und Rechtsklick öffnen Umbenennen, Im Finder
  zeigen und Löschen wie bei Zonen. Eine Ordnerkarte auf eine andere gezogen
  legt den Ordner hinein (gleiches Anheben wie 4.4, Ziel mit Ring).
- **Titel:** Steht oben im Kopf (`align-self: start`) und springt nicht, wenn
  sich die Zahl der Karten beim Wechsel der Ebene ändert.
- **Segmente (0.39.1):** Rechts im Kopf steht nur noch ein Chip „(Segmente)“,
  der an- und ausgeklickt wird; aus ist das Mosaik. Das Wort „Mosaik“ entfällt.
- **Segmente als Ablageziel (0.35):** In „(Segmente)“ ist jeder Abschnitt
  (`.p-folder-section`) Ziel für seinen Ordner, auch im Unterordner; der Abschnitt
  der losen Objekte legt direkt in die offene Ebene. Ziel zeigt den Ring (4.1,
  `--ring-lg`, 8 px Abstand) ohne Fläche, der eigene Abschnitt ist kein Ziel.

### 4.6 Objekt (`.card`)
- **Zweck:** Ein gesichertes Bild, Video, PDF oder eine Text- bzw. Seitenkarte
  im Raster.
- **Form:** Eckig, ohne Fläche. Solange es lädt, steht ein Glasplatzhalter da.
- **Video beim Hover (0.27):** Nach 250 ms Verweilen zeigt `.card-scrub` acht
  Standbilder aus dem ganzen Clip nacheinander (alle 320 ms), ohne Ton. Mit
  `prefers-reduced-motion` bleibt das Standbild.
- **Unsorted (0.33):** Die letzten zwei Tage vor „Zuletzt gelöscht“ steht oben
  links „noch n T.“ (`.card-days`, Marke wie `.card-media`, Schrift `--danger`).
  Gezählt ab dem ersten Sehen (`inboxSeen` im Snapshot). Die Kopfzeile von
  Unsorted sagt nur „Nach 7 Tagen gelöscht.“, ohne Anzahl, Datum und Quellen.
  Dieselben Worte stehen im Index hinter dem Namen (dim) und auf der Zonenkarte
  hinter der Anzahl statt des Datums (0.33.1). Es ist immer Unsorted (0.35, vorher
  wählbar); Einstellungen › Bibliothek „Unsorted nach 7 Tagen gelöscht“ schaltet
  nur an (ab Werk) oder aus. Seit 0.36.0 zählt jeder Ordner namens Unsorted, auch
  selbst angelegt, klein geschrieben, mit Nummer davor oder als Unterzone
  (`library.rs is_unsorted`). Unsorted lässt sich nicht umbenennen. Unsorted entsteht mit jeder eigenen Bibliothek, auch in einem
  bestehenden Ordnersystem, und steht vorn als erste Active Zone; in einer
  Bibliothek mit eigener Reihenfolge, sobald sie neu dazukommt (0.33.2). Wer sie
  danach verschiebt, behält seinen Platz. Geteilte
  Bibliotheken räumen ab Werk nicht auf und zeigen nichts davon; wer dort in den
  Einstellungen „An“ wählt (0.36.1), bekommt dieselben Hinweise und dasselbe
  Aufräumen wie in einer eigenen Bibliothek.
- **Liste (0.34):** dritte Ansicht neben Raster und Original, Chip
  „(Liste)“. Zeilen wie im Finder: Bild, Name, Format, Maße, Größe, Quelle,
  Gesichert. Dateiname in Geist 13 px/500 (`--p-text`), Angaben in 12 px
  `--p-dim`, Format als Label (Mono), Spaltenköpfe als Label (Mono,
  Versalien), Zeilen mit `--p-rule` getrennt, Auswahl mit `--ring-0` um die
  ganze Zeile. Die Größenstufen setzen die Bildhöhe (`--lt`, 24–96 px).
  In „Zuletzt gelöscht“ steht als Quelle die Zone und die Resttage.
  Text- und Notizkarten (0.34.2) stehen als Blatt in der Bildspalte, quadratisch
  wie im Raster, ohne Kennung und Fuß; im Namen steht der Anfang des Texts
  (Notiz: Titel · Text). Spaltenköpfe sind Knöpfe (Label faint → dim beim
  Hover, aktiv in `--p-text` mit ↑/↓): sortieren aufsteigend, absteigend, dann
  wieder nach der Reihenfolge der Zone. Maße, Größe und Datum beginnen mit dem
  Größten bzw. Neuesten, leere Werte stehen immer unten. Sortiert ist Ziehen
  zum Umstellen aus.
- **Nicht:** Keine Rundung, kein Schatten und keine Bildunterschrift im Raster.
  Die Info gibt es per `I` oder im Seitenblatt.

### 4.6a Farbmarke (App 0.41.0, `.card-mark`, `.p-marks`)
- **Zweck:** Objekte gehören zusammen. Unabhängig von der Stage (4.25), in jeder Zone.
- **Farben:** die fünf Farben der Textkarten (Rot, Grün, Violett, Cyan, Gelb,
  `QUOTE_TONES`), ein Objekt hat höchstens eine.
- **Form:** Innenrand 4 px in der Farbe auf dem Bild (`box-shadow: inset`),
  eckig wie das Objekt (Lukas 23.10.: Variante B). Er liegt innen, berührt den
  Ring (außen, 4.1) nie und hält das Raster ruhig. In der Liste ein Strich 3 px
  links an der Zeile.
- **Setzen:** Rechtsklick › Farbmarke (Quadrate 16 px, gewählte mit gestricheltem
  Abstand wie `.tone-dot`, „Ohne“ als Label), ⌥1–5, ⌥0 nimmt sie weg. Dieselbe
  Farbe noch einmal nimmt sie auch weg.
- **Filtern:** Im Kopf der Zone neben „(Alle)“ je vergebene Farbe ein Chip mit
  Quadrat 9 px und Anzahl. Klick zeigt nur diese Farbe, noch ein Klick alles.
- **Speichern:** in der Bibliothek (`note.mark`, `metadata.json`), wie Bildtext
  und Rechte, also auch auf anderen Macs und in geteilten Bibliotheken.
- **Nicht:** Keine Farbe für Auswahl oder Fokus: Farbe heißt „gehört zusammen“,
  der Ring heißt „ausgewählt“.

### 4.7 Chip (`.p-chip`)
- **Zweck:** Filter und Ansichtswahl.
- **Form:** Geist Mono, 11 px, Versalien, ohne Fläche. Die Zustände laufen nur
  über die Textfarbe: faint → dim (Hover) → text (aktiv).
- **Hinweis in der Filterzeile (0.31.2):** Solange die Bilderkennung läuft,
  steht am Ende der Filterzeile „(Bilderkennung: noch n von m)“ als Mono-Text in
  dim, ohne Klick. Er zählt mit und verschwindet, wenn alles erkannt ist.
- **Filter in Alle Objekte (0.39.1, Variante C):** Eine Zeile mit Art, Farbe
  und Zeit, dahinter „(Mehr)“. Das öffnet darunter eine Tabelle (Label 96 px
  in Mono dim, Chips rechts) mit Form, Format, Doppelt, Motive, Quellen und
  bei geteilten Bibliotheken Von. Der Zustand bleibt (`archiver.filtersMore`).
  Ist sie zu, stehen gesetzte Filter daraus als Pille in der ersten Zeile:
  Mono 11 px, Fläche ink × 0.08 (Hover 0.14), `--r-md`, „×“ in dim, Klick
  entfernt den Filter. „(Zurücksetzen)“ steht rechts außen.

### 4.8 Knopf (`.btn`)
- **Zweck:** Handlungen in Dialogen und Einstellungen.
- **Form:**
  - Standard: 1 px Linie `--dash`, beim Hover `--ink`.
  - Primär: Fläche `--ink` mit Text `--ink-inverse`.
  - Ecke `--r-md` überall (18.10., vorher eckig), folgt also der Rundung
    aus Einstellungen › Darstellung. Das gilt auch für Sperrbild (Feld und
    „Freischalten“) und die Felder im PDF-Ersteller.
- **Nicht:** Höchstens ein Primärknopf pro Dialog.

### 4.8a Rückfrage (`.dialog.confirm`)
- **Zweck:** Kleine Dialoge über der Fläche: Löschen, Installieren,
  Umbenennen. Dieselbe Form haben Importieren
  (`.dialog`), der Hinweis zum ersten Bild
  (`.onb-drop`) und die Tour-Karten (18.10., vorher gestrichelte, eckige
  Rahmen).
- **Form (18.10.):** Glas wie Menüs (2.4: `--menu-bg`, `--menu-shadow`,
  `blur(24px) saturate(1.4)`, hell `blur(28px) saturate(0)`), Ecke `--r-lg`,
  Knöpfe `--r-md`. Beide folgen der Rundung aus Einstellungen › Darstellung.
  Eingabefelder haben 20 px Luft zu den Knöpfen.
- **Nicht:** Keine feste Rundung und keine Pillen-Knöpfe (bis 0.40.1 waren es
  14 px und 999 px, Lukas 18.10.: „rundungen passen nicht mit den
  einstellungen“).

### 4.9 Menü (`.p-more-menu`)
- **Zweck:** Kontextmenüs, ≡-Menü und Zonenwahl.
- **Form:** `--menu-bg`, `--menu-shadow`, `--r-md`. Zonen erscheinen als
  Kästchen in zwei Spalten.
- **≡-Menü (18.10.):** hängt bündig an der Kante der Navigation, ist aber so
  breit wie sein Inhalt (mindestens 240 px, höchstens 360 px) und richtet sich
  nicht nach der Breite der Leiste.

### 4.10 Toast (`.toast`)
- **Zweck:** Kurze Rückmeldung, etwa „gesichert“ oder „verschoben“.
- **Form:** Akzentfläche, 12 px, Gewicht 600, blendet in 0.18 s ein.
  Fehler erscheinen in `--danger`.

### 4.11 Lightbox und Seitenblatt
- **Lightbox:** Das Bild steht groß auf `--lightbox-bg`, das Raster dahinter
  verschwimmt (`blur(24px)`, 0.30.1), damit kein Text durchscheint. Leertaste öffnet sie,
  Pfeile blättern, `I` zeigt die Info. Klick neben das Objekt schließt sie
  (offene Zonen klappen erst zu, vergrößert schließt nichts).
- **Leiste der Lightbox (0.28):** dieselben Maße wie die Navigation (4.2):
  Innenrand `--nav-rim`, Knöpfe `--nav-h`, 13,5 px, Abstand 3 px, Ecke `--r-md`.
  Knopf „Gesichert“ (0.30.1) nur bei Linkkarten mit gesicherter Seite, neben „Quelle“.
  Feste Breite `min(960px, 100% − 32px)` (0.37.1, vorher 600 px), bei offener
  Info abzüglich der Info-Spalte; sie springt beim Blättern nicht. Passt der
  Dateiname nicht, kürzt `fitLbName` ihn in der Mitte („Anfang … Ende.png“),
  damit Anfang und Endung sichtbar bleiben. Der volle Name steht im Tooltip.
- **Seitenblatt:** Details von rechts, `--sheet-w` = min(560 px, 46 vw).
- **Stage (0.39.1 als Wolke, 0.42 Atlas, 0.42.1 Collection):** Knopf „Stage“
  in der Leiste und Taste W öffnen die Stage (4.25) über der Lightbox.

### 4.11a Bildtext (`.card-cap`, `.lb-caption`, 0.29)
- **Zweck:** Credits oder Unterschrift zu einem Bild, wie unter einem Bild im
  Magazin. Kommt nur aus Text, den man beim Sichern auf der Seite markiert hat,
  aus dem Feld „Bildtext“ in der Info oder per ⌘V auf ein gewähltes Bild.
  Seit 0.30 ist das frühere Studio-Feld „Notiz“ darin aufgegangen (wird beim
  Öffnen der Bibliothek angehängt). Durchsuchbar, im PDF „Mit Bildtext“.
- **Form:** Drei Zeilen: Kennung in Geist Mono 10/11 px, Versalien, `--p-dim`
  (Zone—Nummer Position/Anzahl, etwa TYPO—02 3/7), Text in Geist 17 px
  (Lightbox).
- **Raster (0.33):** Kein Textauszug mehr unter dem Bild, der Chip „(+ Text)“
  ist entfallen. Ein Punkt 14 px (bis 0.42.5: 10 px), 6 px vom Rand unten rechts, in der Farbe der
  Textkarte (`.card-dot`, 0.33.1 statt Eselsohr) zeigt an, dass das Bild Text
  hat. Seit 0.42.6 auch, wenn eine Note das Bild verknüpft (Farbe der Note). Feine dunkle Kante, damit er auf gleichfarbigen Bildern stehen bleibt.
  Bilder ohne Text bleiben stumm.
- **Farbe (0.33):** In ⌘D unter „Bildtext“ wählbar, dieselbe Reihe wie bei
  Textkarten und Notizen (`tonePicker`). Ohne Wahl gilt die Farbe aus dem Hash.
- **Lightbox (0.30.1):** Karte im Ton des Bildes wie eine Textkarte
  (`quoteTone`, `.text-sheet.lb-cap-card`): Kennung BILDTEXT · Zone—Nummer,
  Text in Geist 500 nach Länge 7 / 5 / 3,6 `cqmin` (mind. 15 px), Fuß Quelle ·
  Datum. Zwei Formen, Einstellungen › Darstellung „Bildtext in der Lightbox“:
  - **Rückseite** (Standard): Die Karte hat genau das Maß des Bildes. Ein
    Eselsohr 22 px in der Kartenfarbe unten rechts zeigt sie an (Hover 30 px).
    `T`, Knopf „Text“ oder das Eselsohr wenden um die senkrechte Mitte,
    2 × 0.18 s (`ease-in` auf die Kante, `ease-out` zurück). Blättern zeigt
    wieder die Vorderseite.
  - **Doppelseite:** Karte 3:4 (mind. 260 px) 24 px links neben dem Bild,
    gleich hoch, beide zusammen mittig. Vergrößert ist die Karte aus.
  Video, PDF und Audio: Karte 300 × 400 px unten links. Der Text ist auf der
  Karte editierbar (Esc oder ⌘↩ beendet, Sichern beim Verlassen), ebenso der
  Text einer Textkarte; deren PNG bleibt unverändert.

### 4.11b Note (`.note-sheet`, `.note-paper`, `.note-links`, 0.29, „Note“ seit 0.42.6)
- **Eine Funktion (0.42.6):** Textkarte und Notiz heißen in der Oberfläche
  „Note“ (Kennung NOTE). Neuer Text wird immer `.md`: Ausschnitt aus der
  Extension, ⌘V in einer Zone, Rechtsklick. Alte Textkarten (PNG) und Bildtexte
  bleiben lesbar wie bisher. Rechtsklick auf ein oder mehrere Bilder › „Note zu
  diesem Bild“ / „Note zu n Objekten“ legt die Note im Ordner des ersten an,
  gleich verknüpft, und öffnet sie zum Schreiben.
- **Herkunft (0.42.6):** Kommt die Note aus dem Web, steht die Domain im Fuß
  (Raster: Domain · n verknüpft; Lightbox: „↗ domain“ zwischen Zahl und
  „Gesichert“, Tooltip Seitentitel). Klick öffnet die Seite an der Stelle
  (Textfragment, `sourceUrl`). Seite und Titel wandern beim Bearbeiten mit dem
  neuen Hash mit (`move_note_hash`).
- **Bild mit Note (0.42.6):** Ohne Bildtext zeigt die Lightbox die (erste) Note
  des Bildes an derselben Stelle wie den Bildtext, als Blatt `.lb-note-card`
  (Kennung, Titel 18–30 px, Text 13–17 px, Fuß), Rückseite oder Doppelseite
  wie eingestellt. Knopf heißt dann „Note“, Taste T. Klick aufs Blatt öffnet die
  Note; die Lightbox blättert durch Note und Verknüpfte.
- **Suche (0.42.6):** Ein Bild passt auch, wenn der Text einer Note passt, die
  es verknüpft (halbe Punktzahl, steht also hinter direkten Treffern).
- **Datei:** Eine Notiz ist eine `.md`-Datei in der Zone. Erste Zeile = Titel
  (als `# Titel` gespeichert), beim Verlassen der Notiz auch Dateiname.
  Anlegen per Rechtsklick in den Hintergrund einer Zone („Neue Notiz“) oder
  ⌘⇧N; sie öffnet sich gleich in der Lightbox, der Titel ist markiert.
- **Raster:** Ein Blatt wie die Textkarte (4.6): Kennung NOTIZ · Datum in Mono,
  Titel fett (6,4 cqw, höchstens drei Zeilen), Text darunter normal, Fuß mit
  „n verknüpft“. Grundton Weiß (`white`), keine Zufallsfarbe. In „Original“
  steht das Blatt hochkant 3 : 4.
- **Lightbox:** Papier max. 620 × 760 px, Innenrand 48/52 px. Kennung NOTIZ ·
  Zone · Datum (Mono 11 px), Titel 30 px/600, Text 17 px/1,45, Fuß mit Wörtern
  und „Gesichert“. Geschrieben wird direkt auf dem Blatt, gesichert nach 0,6 s
  Pause und beim Verlassen. Die Farbpunkte stehen in der Leiste: eine Reihe
  für Notizen und Textkarten (0.30) aus den bestehenden Farben Rot, Grün,
  Violett, Cyan, Gelb, Weiß (`#ffffff`, kein Papierton mehr) und Schwarz. Die
  Extension wählt beim Markieren zufällig aus den fünf Buntfarben. Schwarzes Blatt bekommt eine feine helle Kante.
- **Verknüpfungen (0.42.6):** Die Verknüpften stehen in voller Höhe 24 px
  rechts neben dem Blatt in einer Reihe, jedes im eigenen Seitenverhältnis
  (Text 3 : 4), wie die Doppelseite beim Bildtext; am Ende „+“ als Rahmen
  44 px. `noteFit` setzt die gemeinsame Höhe `--nh` (max. 760 px, Blatt
  620 : 760, Innenrand mitskaliert), mindestens 320 px; passt es dann nicht,
  läuft die Reihe waagerecht weiter. Fuß: Wörter · n verknüpft links, Quelle
  und „Gesichert“ rechts. × oben rechts auf der Kachel löst die Verknüpfung.
  (Bis 0.42.5: Spalte 150 px links neben dem Blatt, Kacheln 72 px.) „+“ öffnet eine Auswahl (Menüfläche 430 px, Suchfeld). Erst
  stehen die Zonen als kleine Tray-Karten da (drei Spalten, Cover 58 px mit
  einem großen und zwei kleinen Bildern, Name 12 px, Anzahl in Mono). Klick
  öffnet die Zone: Raster mit sechs Spalten, gewählte gestrichelt, „‹ Zonen“
  bzw. Esc führt zurück. Suchen durchsucht die ganze Bibliothek (0.29.1). Klick auf eine Kachel öffnet das Objekt; die Lightbox
  blättert dann durch Notiz und Verknüpfte.

### 4.12 Tray (`palette.css`)
- **Name (0.28):** In der Oberfläche heißt sie „Tray“, im Code weiter `palette`.
- **Zweck:** Per Kürzel (Standard ⌥ Space, Einstellungen › Mac & Browser) über
  jedem Programm Zonen und Objekte holen, etwa in einen InDesign-Rahmen.
- **Form:** 860 × 620 px, macOS-Glas, Suchfeld 17 px oben. Grund `--pal-bg`
  (Dunkel rgb 28 × `--win-alpha` 0.72, Hell rgb 244 × 0.78) ist seit 0.28 auch
  der Standard des App-Fensters und folgt seit 0.30 dem Glas-Regler (2.1). Hell/Dunkel und Ecken folgen Einstellungen › Darstellung;
  die Fensterecke ist eckig 0, klein 6, mittel 12 (Standard), groß 18 px.
  Start: Active Zones als Zonenkarten (4.4) mit 01–04 und „Active Zone“,
  darunter der Index; seit 0.32 haben alle Karten dieselbe Höhe (Cover 8:3).
  Zone: Active Zones als Reiter, darüber ihre Unterordner als Karten
  („(Ordner)“, Klick oder ↵ öffnet, ⌫ eine Ebene hoch, Kopf „Zone / Ordner“),
  darunter die Objekte eckig in sechs Spalten (4.6). Der Inhalt hat oben 4 px
  Luft, damit der Ring (4.1) der ersten Reihe nicht abgeschnitten wird. Suche: passende Zonen als Karten,
  darunter Objekte. Fußzeile: links „Tray“, rechts die Tasten als
  Mono-Etiketten, esc am Ende.
- **Verhalten:** Aus InDesign geöffnet heißt ↵ „Platzieren“ (markierter
  Rahmen oder Platzieren-Cursor), sonst „Kopieren“. Beim Herausziehen (0.37.2, Variante V4) klappt der Tray
  in 0.22 s ease-out nach oben auf das Suchfeld zusammen (Fenster 72 px hoch,
  Oberkante bleibt, `panel_ghost` in macos.rs) und lässt Klicks durch. Im
  Suchfeld steht dann die Leiste: „Tray“ mono, Vorschau 30 px mit Ring (4.1),
  Name 15 px (bei mehreren „3 Bilder“), Zone gedimmt, rechts „Loslassen, um
  abzulegen“ (aus InDesign: „zu platzieren“) und esc · Abbrechen. Der übrige
  Inhalt blendet in 0.12 s aus. Nach dem Ziehen geht der Tray auf den
  gemerkten Rahmen zurück. Mit „Bewegung reduzieren“ ohne Animation. Klick in ein anderes Programm
  blendet sie aus.
- **Textkarten (0.27):** wie in der App in ihrer Grundfarbe (`.pal-text`).
- **Leertaste (0.27):** zeigt das gewählte Objekt groß, innerhalb der Palette
  (`.pal-preview` ersetzt den Inhalt, Suchfeld und Fußzeile bleiben). ←→
  blättern, Leertaste oder esc schließen. Im Suchfeld mit Text tippt die
  Leertaste weiter.
- **⌘1–4:** wechselt die Bibliothek wie in der App.
- **⌘C, ⌘V (0.32):** ⌘C kopiert das gewählte Objekt, ⌘V legt die
  Zwischenablage in die offene Zone. Steht etwas im Suchfeld, gehören beide
  dem Suchfeld.
- **Mehrere Objekte (0.37.2):** ⌘-Klick nimmt einzeln dazu oder heraus,
  ⇧-Klick und ⇧ + Pfeile wählen einen Bereich. Jedes gewählte Objekt trägt
  den Ring (4.1). Ziehen und ⌘C nehmen alle gewählten mit, esc hebt die Wahl
  zuerst auf, ein Pfeil ohne ⇧ ebenso.
- **Nicht:** Sie aktiviert Archery nicht und holt das Hauptfenster nicht nach vorne.
  Liegt Archery hinten, wird das Hauptfenster beim Öffnen weggelegt und beim
  Schließen hinter alle Fenster zurückgelegt (0.28).

### 4.13 Zonenleiste am ganzen Mac (`dropbar.css`)
- **Zweck:** Wie 4.3, aber beim Ziehen von Dateien in jedem Programm. Seit
  0.40.21 nur, wenn beim Ziehen ⌥ gehalten wird; ohne ⌥ bleibt alles frei.
- **Form:** Volle Breite unten auf dem Bildschirm mit dem Zeiger, dieselben
  vier Glaskarten und derselbe Ring wie 4.3. Seit 0.40.22 im Format 2:1 wie
  die Zonenkarte (2.3, 4.4): Die Höhe ergibt sich aus der Bildschirmbreite
  (`dropbar_height`, Karte halb so hoch wie breit, plus 4 px oben und 10 px
  unten, 120–320 px; 13-Zoll-Mac 192 px, vorher fest 280 px). Rand 10 px zu den
  Bildschirmkanten, 8 px zwischen den Karten (18.10., vorher 14 und 10),
  gleich in der App (4.3) und der Extension.
- **Einblenden (0.40.21):** Dateien ziehen, ⌥ halten → die Leiste steht
  sofort in voller Höhe, ohne Streifen und ohne Fenster-Animation;
  ⌥ darf danach losgelassen werden. Die Karten sind vorab gebaut und werden
  im Hintergrund nachgeladen, wenn sich die Bibliothek ändert; beim
  Einblenden wird nichts geladen. Ein Drop in einer Lücke oder am Rand zählt
  für die Karte derselben Spalte. Der Streifen (0.32 bis 0.40.20: erst 90 px,
  volle Höhe erst unter dem Zeiger) ist weg. Die Extension im Browser zeigt
  seit 3.29.7 keinen Streifen mehr: Ihre Leiste steht beim Ziehen gleich in
  voller Höhe (`overlay.css` `.zones-bar`, 30vh, höchstens 260 px), mit Hinweis
  unten auf der Karte. Die Leiste der Extension hat
  seit 3.25.2 genau die Karten der Leiste in der App (4.3, `.p-hot`): Glas mit
  Unschärfe und Schatten, Nummer oben, Name in der Mitte, Ziel mit
  gestricheltem Ring (`--ring-lg`, 3 px) statt Outline. Sie nimmt
  den Zeiger an (3.25.1), damit Chrome am unteren Rand die Seite nicht
  darunter wegscrollt; Text- und Instagram-Ziehen nutzen dieselbe Leiste. Am ganzen Mac
  liegt seit 0.40.24 hinter jeder Karte echtes macOS-Glas (2.4): Die Leiste
  meldet nach dem Einblenden die Rechtecke ihrer Karten und die Rundung
  (`dropbar_glass`), darüber färbt `--zone-alpha` wie in der App. Vorher
  hatten die Karten dort keine Unschärfe und wirkten wie graue Folie.
- **Ablauf (0.31.0):** Nach dem Loslassen zeigt das Ziel „Wird gesichert …“
  (`.busy`, kein Glühen). Erst wenn Import oder Verschieben fertig ist, kommt
  „Gesichert in …“ (`.landed`). Scheitert es, steht „Nicht
  gesichert: …“ in `--danger` (`.failed`, kein Glühen) 2.5 s lang, dann
  blendet die Leiste aus. Die Leiste bleibt so lange stehen (`dropbar_hold`).
- **Ablegen (0.40.22, statt Glühen; seit 0.40.23 auch in der Leiste der App,
  4.3, gemeinsam in `dropfx.js` und `dropfx.css`):** Ab Werk „Miniatur“: Das erste Bild
  fliegt vom Zeiger in die obere rechte Ecke der Karte (36 px, 0.28 s
  `--sheet-ease`) und bleibt dort, bis die Leiste geht; bei mehreren Dateien
  steht „+N“ in Mono davor. Das Bild kommt direkt aus der abgelegten Datei
  (`dropbar_preview` gibt nur diese Bilddatei frei), noch bevor gesichert ist;
  „Gesichert“ steht weiter erst danach. Ohne Bild (Video, PDF) bleibt nur der
  Hinweis. Danach steht die Leiste 0.7 s und blendet in 0.28 s aus (2.5). Darstellung › Drop-Feedback (4.24b)
  bietet Varianten: Haken vor „Gesichert“, Ring schließt sich, Zähler (Nummer
  rollt auf „+N“), Dateiname (Name rollt zum Dateinamen, mindestens 0.6 s, und
  zurück), Nachbild (graues Bild × 0.22 verblasst in 1.6 s), Glühen (bisher).
  In der App (4.3) steht dabei „Abgelegt“, weil das Verschieben dort nicht
  abgewartet wird. Die Position aus `onDragDropEvent` ist auf macOS schon in
  Punkten; nie durch `devicePixelRatio` teilen (0.26 bis 0.40.22 taten es, der
  Drop landete auf Retina links daneben). Die Extension zeigt seit 3.29.8 fest die
  Miniatur (`.zone-stamp` in `overlay.css`), ohne die Varianten aus 4.24b und ohne
  „+N“; die Leiste steht dort 0.7 s ab dem Loslassen, weil sie das Sichern nicht abwartet.
- **Ab Werk an (0.40.21, vorher ab 0.32.1 aus):** Weil sie nur mit ⌥ kommt,
  ist sie wieder an, auch wenn die alte Leiste ausgeschaltet war.
  Einstellungen › Mac & Browser (0.40.24): „Mit ⌥“ (Standard), „Immer“
  (sobald Dateien gezogen werden, `dropbarAlways`) oder „Aus“; erst diese
  Wahl zählt (`dropbarAltChosen`). Im Browser übernimmt die Extension.
- **Nicht:** Erscheint nicht, wenn Archery selbst vorne ist, und nicht bei
  Text oder Links ohne Datei. Zeigt nie „Gesichert“, bevor es gesichert ist.
  Im Browser (Extension 3.29.5) erscheint sie nur, wenn das Ziehen auf der
  Seite beginnt, nicht bei Dateien von außen. Regler, Links und kleine
  Bedienelemente über Bildern und Videos bleiben Bedienelemente. Seit 3.31.3
  kommt sie von selbst nur für Bild, Link oder markierten Text; eigenes Ziehen
  der Seite (Kanban, sortierbare Listen) sowie Text und Links von außen nur
  mit ⌥, das auch erst beim Ziehen kommen darf. Ohne Zonen bleibt das Ablegen
  ganz bei der Seite. Karten aus Kacheln (Google Maps, OpenStreetMap), Text und
  Zeichenflächen auf Hintergrundbildern und schmale Griffe über Bildern
  (Vorher/Nachher) gehören der Seite: Ein verstecktes Bild zählt nur, wenn es
  mindestens die Hälfte der gedrückten Fläche deckt und umgekehrt.

### 4.14 Zuletzt gelöscht (`.p-deleted`, `.card.del`)
- **Zweck:** Gelöschte Objekte und Zonen liegen 7 Tage unter
  `.archiver/deleted/<Zeit>/<alter Pfad>` in der Bibliothek, dann gehen sie in
  den Papierkorb des Macs. Erreichbar über die letzte Zeile im Index und ≡.
- **Form (0.28):** Dasselbe Raster und dieselbe Karte wie in jeder Zone (4.6),
  mit Größenstufen. Das Badge zeigt Zone und Resttage. Auswahl zeigt den Ring
  (4.1). Wiederherstellen und Sofort löschen stehen als Chips rechts.
- **Dateien ohne Vorschau (0.37.0):** PSD, AI, ZIP … aus einer gelöschten Zone
  erscheinen als Karte mit dem Format-Platzhalter (wie ein Video ohne Vorschau)
  und lassen sich ebenso zurückholen. Die Rückfrage beim Zone-Löschen nennt sie:
  „12 Objekte und 3 Dateien ohne Vorschau“.
- **Lightbox (0.29.1):** Doppelklick, Leertaste oder Enter öffnen sie wie in
  jeder Zone. In der Leiste stehen statt Zone und Info „Wiederherstellen“ und
  „Sofort löschen“ (rot beim Überfahren). Notizen sind hier nur lesbar.
- **Endgültig löschen fragt immer nach** (⌫, Chip, Menü, Lightbox): Dialog mit
  „Abbrechen“ und rotem „Sofort löschen“, Enter bestätigt.
- **Nicht:** Kein eigener Zonenplatz unter den Active Zones, keine Zeile, wenn
  nichts gelöscht ist.

### 4.15 Handbuch (`.p-manual`, `manual.js`)
- **Zweck:** Alle Funktionen samt Tasten, erreichbar über ≡ › Handbuch. Der
  Inhalt steht in `app/src/manual.js` (Deutsch und Englisch) und ist dieselbe
  Quelle wie archiver-app.com/manual. Neue Funktionen dort nachziehen.
- **Form:** Seite wie Information und Einstellungen: Seitenblatt von rechts,
  Blöcke (`.p-block`) je Abschnitt, Zeile mit Titel und grauem Text links,
  Tasten rechts (`.p-keys`, höchstens 140 px breit, umbrechend).
- **Taste (`kbd` im Seitenblatt):** Geist Mono 11 px, `--p-text`, Rand als
  `inset 0 0 0 1px` ink × 0.25, Ecke `--r-sm`.
- **Nicht:** Keine Bilder, keine Akzentfarbe, kein eigenes Inhaltsverzeichnis.

### 4.16 Hinweisleiste (`.p-update`, `.p-ext`)
- **Zweck:** Ein Hinweis, der bleibt, bis man handelt: „Archery X ist da“
  und seit 0.30 „Extension X ist da“ bzw. „Keine Extension verbunden“.
- **Form:** Glas-Pille oben rechts (12 px vom Rand), `--p-pill`, Text 13 px,
  danach Handlung in Mono 12 px in Klammern („(Laden)“) und ×. Immer nur
  eine Leiste; das App-Update verdrängt den Extension-Hinweis.
- **Extension:** Der Host notiert bei jedem Kontakt die Version
  (`extension.json` neben den Einstellungen). Älter als `EXTENSION_MIN`
  (`bridge.rs`) oder ohne Version = veraltet. Hat sich nie eine gemeldet,
  kommt der Hinweis erst nach zwei Tagen. × blendet ihn aus, bis sich Zustand
  oder Mindestversion ändern. Einstellungen › Mac & Browser zeigt den Stand,
  am Mac darunter eine Zeile für Safari (4.27), solange die Safari-Brücke an ist.
- **Nicht:** Kein Toast, keine Akzentfarbe, kein Dialog beim Start.

### 4.17 Mehrfachmarkierung (Extension 3.27.0, `content.js`)
- **Zweck:** Mit ⌥ (seit Extension 3.29.1, vorher ⌘) mehrere Stellen einer
  Seite sammeln und in einem Zug in eine Zone ziehen. ⌘-Klick gehört dem
  Browser (Link im neuen Tab). ⌥-Klicks bekommt die Seite nicht: kein
  Weiterspringen, kein Download (3.29.3). ⌥ + Markieren nimmt Text dazu, ⌥-Klick nimmt ein Bild
  (ab 80 px) dazu oder wieder heraus. Klick daneben oder Esc hebt alles auf.
  Liegt das Bild in einem Link, öffnet ⌘-Klick wie gewohnt einen neuen Tab,
  solange noch nichts markiert ist (Extension 3.29.1); erst während einer
  Sammlung nimmt er das Bild dazu.
- **Form:** Text behält die Farbe der Markierung, also die Farbe der späteren
  Textkarte (`::highlight(archery-marks)`, gleiche Werte wie `::selection`).
  Alle Stücke einer Runde teilen eine Farbe. Bilder tragen den Ring aus 4.1:
  1 px gestrichelt 4/4, 3 px Abstand, eckig, weiß mit feinem dunklen Schatten,
  weil die Extension auf fremden Seiten keinen eigenen Grund hat. Einblenden
  0.18 s `ease-out`, ohne bei `prefers-reduced-motion`.
- **Ablegen:** nur Text → eine Note (`.md`, seit 3.30.0; ohne App eine Textkarte als Bild), Stücke als Absätze in
  Markierreihenfolge. Text und Bild → jedes Bild mit allem Text als Bildtext.
  Nur Bilder → jedes Bild einzeln.
- **Nicht:** Kein Zähler, kein zweiter Rahmen um Text, keine Akzentfarbe.

### 4.18 Ziehbild (Extension 3.28.0, `content.js`)
- **Zweck:** Am Zeiger hängt beim Ziehen das Motiv selbst.
- **Form:** Bild oder Video im eigenen Seitenverhältnis, höchstens 220 px an
  der langen Seite, eckig (ohne Rundung), Schatten `0 12px 32px` 40 %, 92 %
  Deckkraft. Video zeigt das gerade sichtbare Bild, sonst das Poster. Liegt in
  der Überlagerung über den Zonen. Gilt für das eigene Ziehen (Instagram) und
  für Videos beim Browser-Ziehen.
- **Nicht:** Kein Quadrat, kein grauer Kasten mit „Video“.

### 4.19 Dateinamen (App 0.36.0, `.nm-*`, `.rn-*`, `.lb-info-sugg`)
- **Zweck:** Einheitliche Namen auf Wunsch, nie ungefragt. Ab Werk „Original
  behalten“. Seit 0.36.4 benennt nichts mehr beim Sichern oder Importieren um,
  auch nicht über die Extension: Umbenannt wird nur im Blatt.
- **Einstellungen › Dateinamen:** Blöcke „Schema“ (sechs Zeilen mit Punkt und
  Beispiel in Geist Mono, gewählt = gefüllter Punkt, Text `--p-text`, sonst
  `--p-dim`), „Eigenes Schema“ (Bausteine als Kästchen mit ‹ und ×, nicht
  benutzte als „+ Baustein“, darunter die Vorschau; jede Änderung dort wählt
  das eigene Schema) und „Aufräumen“ (Wörter statt Schalter, 4.7).
- **Umbenennen-Blatt:** Rechtsklick auf die Auswahl › „Umbenennen …“. Blatt in
  `--menu-bg`/`--menu-shadow`, `--r-lg`, 760 px, blendet in 0.18 s ein, Grund
  abgedunkelt. Zeilen: Häkchen, Vorschaubild 40 px eckig, alter Name (Geist
  Mono, `--p-dim`, „uneinheitlich“ als Kästchen mit Linie), Pfeil, neuer Name
  (Fläche ink × 0.06, Klick = bearbeiten, gestrichelter Ring wie 4.1). Ein
  Primärknopf „n umbenennen“ (4.8). Gilt seit 0.36.4 auch in geteilten
  Bibliotheken. Im Blatt wählen die Chips ein anderes Schema, ohne die
  Einstellung zu ändern. Das Blatt liegt über der Lightbox (z-index 90).
- **Rechtsklick in der Lightbox (0.36.4):** dasselbe Menü wie 4.7
  (`zone-menu p-ctx-menu`, `--menu-bg`, `--menu-shadow`), nur mit
  „Umbenennen …“, Trennlinie, „Öffnen“, „Quelle“ (bei Seitenadresse),
  „Im Finder zeigen“, „Kopieren ⌘C“. Gilt für das gezeigte Objekt. Nicht für
  Notizen und nicht in „Zuletzt gelöscht“. Über Eingabefeldern, der Leiste und
  der Info-Spalte bleibt das Systemmenü.
- **Namensvorschlag (Info-Spalte, Details und Lightbox):** Kasten ink × 0.055
  über „Bildtext“, Überschrift Geist Mono 11 px, Name in Geist Mono 12.5 px,
  Begründung `--p-dim`, „Übernehmen“ (Primär) und „Ändern“. Erscheint, wenn
  Seitentitel, Motiv oder Farbe einen Namen ergeben, der vom jetzigen
  abweicht. Beim Einblenden läuft einmal ein Lichtstreif über den Namen.
- **Ausnahme zur Tabelle 2.5:** Der Lichtstreif braucht 1.1 s, einmal, auf
  ausdrücklichen Wunsch (F11). Mit `prefers-reduced-motion` entfällt er.
- **Nicht:** Kein Umbenennen ohne Bestätigung im Blatt oder auf „Übernehmen“,
  kein Dienst im Netz: Vorschlag und Schema nutzen nur vorhandene Daten
  (`naming.rs`).

### 4.20 Bibliotheken beim Ziehen (Extension 3.29.0, `.lib-bar`)
- **Zweck:** Beim Ziehen im Browser die Bibliothek wechseln, ohne ⇧ mehrfach
  zu drücken. Erscheint nur, wenn es mehr als eine Bibliothek gibt.
- **Form:** Eine Glas-Pille mittig über der Zonenleiste (4.13), Werte wie die
  Navigation (4.2): 10 px Rundung, 5 px Innenabstand, Namen 14 px Geist. Die
  aktive Bibliothek ist gefüllt (ink × 0.1 / schwarz × 0.08). Steht
  `min(30vh, 260px) + 22px` über dem unteren Rand, also gleich über der
  vollen Leiste (3.29.7, vorher erst über dem Streifen).
- **Ablauf:** Zeiger auf einen Namen → Ring aus 4.1 (`--ring-md`, rx 9, 3 px
  Abstand). Nach 0.4 s Verweilen wechselt die Bibliothek, die Karten bleiben
  stehen und tauschen nur ihre Namen. ⇧ springt weiter zur nächsten (dragover liest nur
  Modifier-Tasten).
- **Nicht:** Kein ⇧-Zeichen in der Pille, keine Zahlen, keine zweite
  Hervorhebung neben Ring und Füllung. Ablegen auf der Pille legt nichts ab.

### 4.21 PDF-Ersteller, Plakat-Anordnung (App 0.37.2, `.pdf-page.flow`)
- **Zweck:** Plakat hat drei Anordnungen: Raster (bisher, angeschnitten,
  Beschriftung im Bild), Index und Paar (nach den Sort-Office-Vorlagen).
  Feld „(Anordnung)“ erscheint nur beim Stil Plakat; bei Index und Paar
  fällt „(Pro Seite)“ weg und „(Beschreibung)“ kommt unter den Titel.
- **Seite:** weiß, 14 pt Rand, Helvetica bzw. Geist 6.4 pt. Kopfzeile in vier
  gleichen Spalten (Titel · Anzahl · Datum · Seite), darunter Nummer [01],
  Titel, Beschreibung (nur Seite 1, höchstens 2 Zeilen, 230 pt breit) und Jahr
  rechts. Bilder ab 58 pt, ungeschnitten, oben bündig, 12er-Raster mit 6 pt
  Abstand. Index wechselt die Reihen (z. B. 5·4·3, 3·_·3, 4·3·5, _·4·_·4),
  Paar setzt zwei Bilder (5·7, 7·5; hoch 12, 7·5). Zu hohe Reihen schrumpfen
  oder gehen auf die nächste Seite. Bildunterschrift 12 pt: Name links, Quelle
  grau rechts.
- **Gleich in Vorschau und PDF:** `flowPages` (app.js) rechnet die Felder in
  Punkten, `flow_sheet` (pdf.rs) setzt genau diese Felder.
- **Nicht:** kein Anschnitt, keine Schrift im Bild, keine Notizen.
- **Knöpfe unten links (K2, gilt für alle Stile):** „PDF sichern“ über die
  ganze Breite, 40 px hoch, `--r-md`, ⌘↩ rechts in Mono 11 px (0.55).
  Darunter „Abbrechen“ als Mono-Etikett 11 px, Großbuchstaben, gedimmt.

### 4.22 Name zum Testen (App 0.37.3 bis 0.43.1, entfernt)
- Das Feld unter Information › App ist seit 0.43.2 weg: Der Name steht fest
  (Archery), und jeder hätte ihn sonst in seiner App überschreiben können.

### 4.22a Updates (App 0.38.1, seit 0.40.8 Einstellungen › App)
- **Zweck:** Kanal wählen (Beta oder Vorab) und jede veröffentlichte Version
  installieren, auch eine ältere.
- **Form:** Block „Version“ (Installiert, rechts Nummer und „Nach Updates
  suchen“), darunter der Block „Updates“. Kanal als `p-choice` (wie alle
  Schalter), „Version wählen“ mit `p-link` rechts; die Liste darunter sind
  normale Blockzeilen (Version, Datum gedimmt, rechts „Installieren“ oder
  gedimmt „installiert“). Vor dem Installieren ein `dialog confirm`, bei
  älteren Versionen mit Hinweis, dass Späteres dort fehlt.
- **Sichtbar:** nur mit Vorab-Kanal oder nach ⌥-Klick auf die Versionsnummer.
- **Kürzel:** ⌘I öffnet Information.

### 4.25 Stage (App 0.39.1 als Wolke, neu 0.41.0, als Atlas 0.42.0, Collection 0.42.1, Stage seit 0.43.4, `cloud.css`, `cloud.js`)
- **Zweck:** Am offenen Objekt weitere Referenzen andocken und sie wie auf einem
  Tisch ordnen, als Moodboard für genau dieses Objekt (Lukas 17.10.: „eine Wolke
  an Referenzen, die als Mood-Strang agieren“; 23.10.: „intuitiver, mehrere
  auswählen, einzelne groß, mehr Zusammenspiel, mehr Ansichten“, dann „eine
  Menübar, ausblendbar, als Bild speichern, Autoplay“). Vorabversion.
- **Name:** Stage, Englisch und Deutsch gleich, „die Stage“: Ein Bild steht auf
  der Bühne, die anderen spielen mit (Lukas 01.11., vorher Collection ab 23.10.,
  davor Atlas nach Warburgs Bildatlas). Menü: „Open on Stage“ / „Auf der Stage
  öffnen“, „Own Stage“ / „Eigene Stage“, „Remove from Stage“ / „Von der Stage nehmen“.
  Im Code bleibt `cloud`.
- **Öffnen:** aus der Lightbox (Knopf Stage, W) oder in der Zone: mehrere
  wählen, Rechtsklick › Auf der Stage öffnen. Das erste Bild ist die Mitte, die
  anderen liegen gleich daneben. Kopf, Navigation und Ablage der App treten
  weg wie bei der Lightbox.
- **Fläche:** ab Werk wie die Lightbox (`--lightbox-bg`, `blur(24px)`). Wählbar
  im Feld über dem Flächen-Quadrat: Lightbox, Schwarz, Weiß, die fünf Farben der
  Textkarten (`QUOTE_TONES`) und Auto (gedämpfter Farbton des offenen Bildes,
  hsl × 32 % × 13 %). Schrift, Ring, Fäden und Lasso nehmen die Schriftfarbe der
  Fläche (`--cl-fg`). Auf farbiger Fläche deckt die Leiste wie ein Menü
  (`--menu-bg`).
- **Zwei Leisten (0.42.1):** Knöpfe wie die Lightbox-Leiste (4.11), aber 10 px
  Innenabstand. Oben (10 px unter der Fensterkante, mittig, so breit wie nötig,
  höchstens Fensterbreite − 200 px, damit die Fensterknöpfe frei bleiben): die
  neun Ansichten und Mischen, immer gleich. Seit 0.43.4 ohne Fläche (Lukas
  01.11., Variante B): kein Glas, kein Schatten, Schrift in `--cl-fg`, nicht
  gewählte Ansichten auf 55 %, die gewählte gefüllt in der Schriftfarbe × 0.15
  (`::before` mit `currentColor`, damit es auf Weiß und Farbe sichtbar bleibt). Unten (wie 4.11, bis 960 px): ohne
  Auswahl Flächen-Quadrat, „+ Objekte“, „+5 ähnliche“ (seit 0.43.4 wieder
  direkt daneben, Lukas 01.11.), Linien, Autoplay, Snapshot, rechts
  Ausblenden und Schließen; mit Auswahl „n gewählt“, Groß,
  Vergleichen (ab zwei) bzw. In die Mitte (eins), rechts Entfernen und ×. Zu
  schmal brechen beide in eine zweite Zeile um. Aktiv = gefüllter Knopf
  (ink × 0.15). Lukas 23.10.: erst „Ansichten in die Leiste“, dann „doch zwei,
  oben und unten, weil sie zu lang wird“.
- **Feld über der Leiste:** Glas wie das Menü (`--menu-bg`, `--menu-shadow`,
  `--r-lg`), steht über dem Knopf, der es öffnet, nie über den Rand.
  Flächen-Quadrat: die neun Flächen als Quadrate 20 px, die gewählte mit
  gestricheltem Rahmen. Klick auf die Fläche oder Esc schließt.
- **Flächen-Quadrat in der Leiste (0.43.4):** 13 px, sitzt in einem Knopf wie
  die anderen (`.lb-btn.cl-tone-btn`, 10 px Abstand), offen = gefüllter Knopf
  statt gestricheltem Rahmen. Lightbox zeigt hier die volle `--lightbox-bg`,
  nicht die halbe Diagonale aus dem Feld (Lukas 01.11.: „wird komisch
  angezeigt“, der Rahmen ragte über die Leiste, die leere Hälfte verschwamm mit
  ihr).
- **Rechtsklick auf ein Bild:** Menü wie in den Zonen (`.p-ctx-menu`): Groß
  ansehen, In die Mitte (eins) oder Vergleichen (mehrere), Eigene Stage
  (dieses Bild als Mitte, ↩), In der Zone zeigen (schließt, öffnet die Zone und
  wählt das Bild), Von der Stage nehmen (rot, ⌫). Rechtsklick auf ein ungewähltes
  Bild wählt es. Keine Größe im Menü (0.42.1).
- **Größe am Bild (0.42.1):** Beim Überfahren unten rechts im Bild − und + (je
  22 px, Glas wie das Menü, `--r-sm`, 6 px vom Rand). Nur in Wolke, Spektrum,
  Strang und Frei, wo die Größe zählt; beim Ziehen weg.
- **Tauschen (0.42.1):** Ziehen auf ein anderes Bild tauscht die Plätze, das
  Ziel trägt währenddessen den Ring. Die Ansicht bleibt (vorher sprang sie auf
  Frei). Ins Leere gleitet das Bild zurück. Mit der Mitte tauscht es in jeder
  Ansicht; die Reihenfolge sonst in Wolke, Fokus, Raster und Spalten. Spektrum,
  Strang, Stapel und Vergleich ordnen selbst. Nur in Frei bleibt liegen, was
  man zieht.
- **Ansichten:** Jede rechnet ihre Lage einmal aus, die Objekte gleiten dorthin
  (`transform`, 0.45 s, cubic-bezier(.2,.7,.2,1)). Tab wechselt.
  - Wolke: dicht um die Mitte
  - Fokus: Mitte groß (halbe Höhe), die anderen klein auf einer Ellipse; Klick tauscht
  - Raster: gleiche Zellen
  - Spalten: gleich breit, verschieden hoch
  - Spektrum: Zeilen nach Farbton der Bildanalyse, Unbuntes am Ende
  - Strang: Kette vom ähnlichsten zum nächsten, Schlangenzeilen, Faden gestrichelt
  - Stapel: je Zone ein Stapel mit Label (4.7), Klick fächert auf
  - Vergleich: zwei bis vier gewählte groß, der Rest klein und auf 40 %
  - Frei: was gezogen wird, bleibt liegen (in den anderen Ansichten tauscht Ziehen)
- **Abstand:** fest 26 px. Die Wahl Eng/Mittel/Weit ist seit 0.42.1 raus
  (Lukas 23.10.: „erstmal rausnehmen“).
- **Größe:** vier Stufen (0.62, 1, 1.45, 2.05 × Grundmaß = kürzere
  Fensterseite / 8, 56–140 px). Gilt in Wolke, Frei und allen Zeilen-Ansichten;
  Raster, Spalten, Fokus, Stapel und Vergleich setzen die Größe selbst. Einstellen
  am Bild (− +) oder mit 1–4 und − +.
- **Auswahl:** Klick wählt, ⇧/⌘-Klick mehrere, Lasso (1 px gestrichelt) auf
  freier Fläche, ⌘A alles, Esc löst. Ring wie 4.1 (`--ring-0`, 4 px Abstand).
  1–4 und − + gelten für alles Gewählte, ohne Auswahl für das offene Objekt.
- **Gruppen:** seit 0.42.1 entfernt (Lukas 23.10.). Gespeicherte Gruppen werden
  beim Öffnen übergangen.
- **Linien:** Beim Überfahren verbinden gestrichelte Fäden das Bild mit den drei
  Bildern der ähnlichsten Farbanteile (Bildanalyse, nicht Zone oder Tags); die
  drei tragen den Ring auf 45 %. Kein Label daran (das „Farbe 94 %“ aus 0.42.0
  ist seit 0.42.1 raus, Lukas). Im Strang zeigt ein Faden die Kette.
  Knopf Linien (0.42.1): von jedem Bild eine Linie zu seinem farblich
  ähnlichsten, jedes Paar einmal, stehen fest. Bleibt gemerkt
  (`archiver.cloudLines`).
- **Autoplay (0.42.0):** P oder Knopf. Schaltet auf Fokus, die Mitte wechselt
  alle 3,2 s, mit mehreren gewählten nur durch die Auswahl. Ein Klick auf die
  Fläche, eine Taste oder Esc hält an. Knopf aktiv, solange es läuft.
- **Ausblenden (0.42.0):** H oder Knopf, nur noch die Bilder auf der Fläche.
  H oder Esc holt beide Leisten zurück, ein Hinweis sagt das einmal. Bleibt für
  die nächste Stage gemerkt (`archiver.cloudBarHidden`).
- **Snapshot (0.42.0 als „Als Bild“, Snapshot seit 0.43.4, Lukas 01.11.):** PNG der ganzen Anordnung, wie sie gerade liegt, mit
  6 % Rand (mind. 24 px) im Ton der Fläche; Lightbox wird deckend (#0a0a0a
  dunkel, #ebebeb hell). Feste Linien und der Faden im Strang kommen mit, unter
  den Bildern, in der Schriftfarbe der Fläche auf 45 %, Strich 1 und 4/4
  gestrichelt, im Maß des Bildes (0.42.1). Lange Seite bis 4000 px, aus den
  Originalen, sonst der Vorschau. Dateiname nach Schema (0.42.1):
  `stage_<zone>_<jjjj-mm-tt>_<hhmm>.png` (bis 0.43.3 `collection_…`), Zone klein, ohne Akzente,
  Leerzeichen als „-“. Sichern-Dialog in Downloads, danach im Finder gezeigt
  (`export_cloud_image`, `atlas.rs`).
- **Fassungen (0.42.0 bis 0.43.3, entfernt):** Lukas 01.11.: „Versionen bitte
  löschen“. Knopf, Feld und Speicher sind raus; gesicherte Fassungen fallen
  beim nächsten Speichern weg.
- **Groß ansehen:** Leertaste oder Doppelklick; Fläche wie die Stage, Bild
  `contain` in einer festen Zelle (`grid-template: minmax(0, 1fr) / minmax(0,
  1fr)`, sonst laufen hohe Bilder aus dem Fenster, 0.42.1), ← → blättert durch die Auswahl (bei einer: alle), Label unten mit
  Zone, Name und Stelle. Esc oder Leertaste schließt.
- **Hinzufügen („+ Objekte“, 0.42.1, vorher „+ Referenz“):** Glas über der
  Leiste (2.4) mit Suchfeld, daneben „Zonen ▾“ („+5 ähnliche“ seit 0.43.4 nur
  in der Leiste), darunter
  Streifen 88 px, ähnlichste Bilder zuerst (Farbanteile, gleiche Zone). Klick
  dockt an. „Zonen ▾“ klappt auf (höchstens 52vh/460 px, liegt über den
  Bildern): oben „Alle Zonen › Zone“, darunter die Zonen als Karten wie im Tray
  (Cover aus bis zu drei Bildern 8:3, Name, Mono-Anzahl, vier je Zeile, unter
  700 px zwei), Klick öffnet die Zone, ihre Bilder brechen um. Die Suche gilt
  auch dort. Enter macht das gewählte Objekt zur Mitte seiner eigenen
  Stage.
- **Bewegung:** Bei `prefers-reduced-motion` springen die Objekte ohne Gleiten,
  Feld und Glas erscheinen ohne Einblenden.
- **Speichern:** Mitglieder, Größe, Platz (Tauschen), Ansicht, Fläche, die
  freie Lage (Anteile der Fläche) je Objekt, lokal unter
  `archiver.clouds` (höchstens 300). Keine Datei, keine Kopien. Alte Modi werden
  beim Öffnen übersetzt: Orbit → Fokus, Farbe → Spektrum, Konstellation → Strang.
- **Nicht:** Keine Rundung an Objekten, keine Akzentfarbe, keine Schatten an
  den Objekten. Die Extension kennt die Stage nicht.

### 4.24c Seitenkopf (vorher Labor › Kopf, fest seit App 0.42.3)
- Ein Kopf für alle Seiten: Titel und Satz als ein Absatz, Titel `--p-text`,
  Satz `--p-faint`, 30 px Semi-Light (2.2). Die vierzehn Labor-Varianten,
  Größe, Farben und die Schriftwahl sind entfernt.

### 4.24a Übersicht mit Kontaktbogen, ein Kopf für alle Seiten (App 0.40.11, `.p-sheetlist`, `.p-home-hero`)
- **Kopf (alle Seiten):** Übersicht, Zone, Alle Objekte, Suche und Zuletzt
  gelöscht tragen denselben Kopf. Titel in voller Farbe, darunter auf eigener
  Zeile der Satz in `--p-dim`. Regular (400), Zeilenhöhe 1.1, Laufweite 0,
  höchstens 20em breit. Größe M 28–42 px (3vw), nur die Übersicht
  eine Stufe größer: L 34–56 px (4vw). Oben 24–72 px (6vh), unten 40–96 px
  (9vh) Raum. In der Übersicht ohne Punkt und ohne „seit …“. Ordnerkarten
  einer Zone stehen rechts oben neben dem Kopf.
- **Übersicht (Ansicht „Mix“ seit 0.42.5, die Seite heißt weiter Übersicht):**
  Oben immer die vier Active Zones als Karten (2:1, vierspaltig ab 200 px
  Kartenbreite, sonst 2 × 2). Darunter „Index (N)“ und die übrigen Zonen als
  Kontaktbogen.
- **Index (0.42.6):** besteht nur aus Kontaktbogen-Zeilen über die volle
  Breite, die Active Zones zuerst (Nummer und „Active Zone“ in Mono über dem
  Namen). Die frühere Tabelle (Nr., Zone, Bilder, Zuletzt, Vorschau) ist weg;
  „Neue Zone“ und „Zuletzt gelöscht“ stehen im Menü.
- **Karten-Fläche (0.42.5, seit 0.42.6 einzige Form, `.p-thumb-cover`):**
  Active Zones, Zonenkarten und Gruppenkarten zeigen seit 0.45.1 immer ein
  **Titelbild** groß oben, darunter eine Reihe der übrigen Bilder (Entwurf J,
  mind. 42 px breit, 8 px Abstand zum Bild). Name und Anzahl unten. Oben 44 px
  Luft für Nummer und Etikett der Active Zones, sonst 16 px, seitlich 16 px.
  Ohne eigene Wahl ist das Titelbild die zuerst gesicherte Referenz der Zone
  (Lukas 03.11.: steht fest und macht die Zone wiedererkennbar, das Neueste
  steht vorne in der Reihe). Rechtsklick auf ein Objekt › „Als Titelbild der
  Zone“ legt es fest, „Titelbild zurücksetzen“ nimmt es zurück. Gruppenkarten
  nehmen das eigene Titelbild der obersten Zone, sonst das erste einer
  Unterzone, sonst die erste Referenz der Gruppe. Gespeichert je Bibliothek in
  `archiver.covers` (lokal). Fehlt das Bild, gilt wieder die erste Referenz.
  Ordnerkarten in der Zone bleiben bei 1 oder 3 Bildern (`cardCover`). Notes und Textkarten stehen als Kachel mit dem Anfang
  ihres Texts (Innenabstand 9 %, wie im Index; bis 0.42.7 schob der
  Abstand der ganzseitigen Karte den Text aus der Kachel).
- **Kontaktbogen:** Je übrige Zone eine Zeile als Karte (`.p-card`, gleiche
  Fläche wie die Active Zones, 10 px Abstand). Die Zeile ist ein einziges
  Klickziel: Klick öffnet die Zone, Ziehen tauscht mit einer Active Zone,
  Fokus legt den Ring um die ganze Zeile. Einzelne Bilder sind nicht
  anklickbar. Die Zeile nutzt das Raster der Active Zones: Name und
  Mono-Anzahl in Spalte 1, die Bilder beginnen genau auf der linken Kante der
  zweiten Active-Karte. Bilder in Höhe der Zeilen-Miniatur (mind. 40 px),
  13:10, 4 px Abstand, Rundung `--r-sm`, höchstens 24, danach „+N“ in Mono.
  Seit 0.42.7 als Raster wie die Karten-Fläche: Spalten dehnen sich, bis die
  Reihe bis zum rechten Rand bündig ist, keine leere Endspalte.
- **Ansichten der Startseite:** Mix (bis 0.42.4 „Übersicht“), Karten, Index. Die geteilte
  Vorschau-Ansicht (Liste links, Mosaik rechts) ist seit 0.40.11 entfernt.
- **Fokus:** Ring (4.1) um die ganze Zeile, keine Farbe.
- **Schmal (< 700 px):** Name über den Bildern, eine Spalte.

### 4.24b Drop-Feedback und frühere Labor-Varianten (Labor 0.40.12 bis 0.43.1)
- **Labor ist seit 0.43.2 entfernt.** Was blieb, steht fest oder ist in die
  Darstellung gewandert.
- **Zonenkarten:** Die kleinen Bilder laufen nicht mehr aus (fest „Aus“). Die
  Varianten Farbverlauf und Glas-Blur samt `.fade-*`/`.cut` sind entfernt.
- **Video-Vorschau:** fest Skim (Zeiger waagerecht = Zeit im Clip, ohne Ton,
  ohne Verzögerung). Daumenkino, Abspielen und Aus sind entfernt. Skim läuft
  auch bei `prefers-reduced-motion`, weil es der Hand folgt.
- **Drop-Feedback (Einstellungen › Darstellung, nach Glas):** Rückmeldung der
  Active-Zone-Karte in der App und am ganzen Mac (4.13): Miniatur (Standard),
  Haken, Ring, Zähler, Dateiname, Nachbild, Glühen, in zwei Reihen (4 + 3),
  darunter ein Satz zur gewählten Variante. Gespeichert nur lokal
  (`archiver.lab`, Schlüssel `drop`), `dropfx.js` liest es selbst. Die Stehzeit
  danach ist fest (0.7 s, 2.5). Klassen der Karte heißen `fx-is-<variante>`,
  nie `fx-<variante>` (sonst greifen die Regeln der Bauteile wie `.fx-ring`
  auf die Karte).

### 4.25a Einstellungen (App 0.40.8, `.p-settings-nav`)
- **Aufbau:** Seitenkopf, darunter die Konto-Liste (4.26), darunter vier
  Bereiche als ruhige Liste: Bibliothek, Darstellung, Mac & Browser (ohne Mac
  nur „Extension“), App. Rechts je eine Zusammenfassung in `--p-dim` und ›.
  Darunter, als eigene Fläche abgesetzt (0.43.2): Studio (4.25b).
- **Bereiche:** Bibliothek = Plätze, verknüpfte Ordner, Standardzone,
  Struktur, Zonen, Dateinamen. Darstellung = Ansicht, Aussehen, Glas, Start,
  Startton. Mac & Browser = Extension, Tray und Zonenleiste, Vorschau-Formate.
  App = Version und Updates (4.22a), Sprache, Feedback, Tasten.
- **Regel:** Ein neuer Schalter kommt in einen dieser Bereiche, nicht in
  einen neuen. Jeder Block trägt seinen Titel, auch wenn er allein steht.

### 4.25b Studio (eigenes Abo, App 0.43.2)
- **Freigabe:** `app_metadata.studio` im Supabase-Konto (setzt nur der Server,
  bis zum Bezahlweg Lukas von Hand). Die App liest es beim Anmelden und beim
  Abgleich (Start, stündlich). In der Beta prüft nur die Oberfläche; fest
  wird es mit der signierten Lizenz (H1).
- **Ohne Studio:** Zusammenfassung „Eigenes Abo“. Die Seite zeigt den Lead,
  darunter einen Block mit dem Satz „Studio ist ein eigenes Abo …“ und Rechte,
  Paket, Geteilte Bibliothek als Zeilen ohne Schalter. Rechte und Paket sind
  aus (`feat()`).
- **Mit Studio:** wie bisher, das Feld „Dein Name“ heißt „Kürzel“; leer gilt
  das Monogramm des Benutzernamens (Platzhalter zeigt es).
- **Geteilte Bibliothek ohne Studio:** Die Sperre (4.26) zeigt „Geteilte
  Bibliothek. Geteilte Bibliotheken gehören zu Studio. Wähle eine andere
  Bibliothek.“ und nur „(Bibliothek wählen)“, ohne Linie und Feld.

### 4.26 Konto, Sperre und Lese-Modus als Liste (App 0.40.6, `.p-say`, `.p-list`, `.p-lock-*`)
- **Ansatz (Lukas 20.10.: „3 aber ohne Highlight“):** Wie die Einstellungen.
  Ein Satz als Überschrift, darunter eine Mono-Zeile und gruppierte Zeilen auf
  einer Fläche. Keine farbigen Highlights mehr (das `.p-hl` aus 0.40.4 ist
  entfernt).
- **Satz (`.p-say`):** 34 px, Zeilenhöhe 1.14, 500, Laufweite 0, `balance`, der
  Rest in `--p-dim`. 34 px, kleiner als der Seitenkopf, kein Titelblatt.
  Auf der Webseite entspricht das 40 px.
- **Mono-Zeile (`.p-group-label`):** 11 px Mono, Versalien, `--p-dim`, links
  der Name in Klammern, rechts ein Zusatz.
- **Liste (`.p-list`):** Fläche ink × 0.045, `--r-lg`, Innenabstand
  2 / 16 px. Zeilen 12 px hoch gepolstert mit Linie `--p-rule`, Schlüssel
  132 px in `--p-dim`, Wert, Zusatz in `--p-faint`. Als Knopf Hover ink × 0.07.
- **Anmelden und erster Start (0.45, Variante W, Lukas 03.11.: „W“):** Wie
  „Sign in“ auf der Webseite (`account/`), damit Webseite und App gleich
  aussehen. Gilt für die Sperre (`#lock`) und die Schritte (`#welcome`), Klassen
  `.onb-*` in `app.css`. Eine Spalte von 480 px, mittig, scrollt, wenn das
  Fenster niedrig ist.
  - Oben rechts nur beim ersten Start eine Mono-Zeile „(1 / 3) Konto ·
    Bibliothek · Browser“ (`.onb-steps`, Nummer in `--p-text`). Keine
    Wortmarke oben (Lukas 03.11.).
  - Umschalter „Anmelden / Konto anlegen“ (`.onb-mode`): Fläche ink × 0.07,
    Knöpfe 32 px, 13.5 px, gedrückt ink × 0.12, Ecke `--r-md`. In den Schritten
    2 und 3 steht an seiner Stelle, gleich hoch (40 px), ein Mono-Hinweis
    „→ Angemeldet als …“ bzw. „→ Bibliothek in …“.
  - Satz (`.onb-say`): `clamp(34px, 3.4vw, 52px)`, Zeilenhöhe 1.04, Laufweite
    −0.025em, 350, 28 px Abstand oben und unten, der Rest in `--p-dim`.
  - Feld (`.onb-field`): Mono-Label 11 px, 0.04em, Versalien, `--p-dim`,
    „(optional)“ in `--p-faint`. Eingabe 56 px hoch, 16 px Innenabstand,
    Rahmen ink × 0.22 (im Fokus `--p-text`), Fläche ink × 0.04, 19 px, Ecke
    `--r-lg`. Code in Mono 22 px mit 0.3em Laufweite. Der Ordner in Schritt 2
    ist ein Feld mit dem Pfad als Wert und „(Anderer Ordner)“ rechts darin.
  - Haken (`.onb-check`): 20 px, Rahmen ink × 0.35, gesetzt `--p-text` mit
    4 px Innenrand in `--ink-inverse`. Nie vorausgewählt.
  - Primärknopf (`.onb-go`): volle Breite, 56 px, 17 px, Text links, Pfeil
    rechts, `--p-text` auf `--ink-inverse`, Ecke `--r-md`.
  - Darunter die Meldung (`.onb-note`, „→ …“, Fehler in `--danger`) und das
    Kleingedruckte (`.onb-fine`, 13 px, `--p-dim`, Links unterstrichen in
    `--p-text`). Mono-Fußzeile (`.onb-foot`) für „(Später)“ und „(Abmelden)“.
  - Schritt 3 „Browser“: Liste wie 4.26 (`.onb-list`) mit „Extension“
    (Verbunden + Version oder Noch nicht da) und „Browser“ (gefundene Browser).
    Der Knopf heißt „Extension laden ↗“, bis sie sich meldet, dann „Zur
    Bibliothek →“. Prüft alle 2 s.
  - Ablauf: Konto → Bibliothek → Browser → App mit Hinweis zum ersten Bild.
    „Anmelden“ legt kein Konto an (`create_user: false`); gibt es zur E-Mail
    keins, wechselt die Sperre zu „Konto anlegen“ mit Hinweis. „Konto anlegen“
    fragt Name (Pflicht), Firma und Website (freiwillig) und den Haken für die
    Beta-Bedingungen ab, gespeichert wie auf der Webseite in `user_metadata`
    (`terms_version`, `terms_accepted_at`). Konten ohne `terms_version` sehen
    beim nächsten Abgleich einmal „Noch ein Schritt.“ mit dem Haken, ohne
    Annahme geht es nicht weiter, Abmelden geht immer (Legal 03.11.).
  - Code-Schritt: Wer die E-Mail ändert, ist wieder beim Senden.
    „einen neuen Code senden“ steht im Kleingedruckten. 6 bis 8 Ziffern.
  - Abweichung zur Webseite: Fehlerfarbe `--danger` statt `#ff6b5e`, Gewicht
    350 statt 300 (Melange gibt es in der App nur in 350), Ecken folgen der
    Rundung aus Einstellungen › Darstellung (bei „Weich“ wie die Webseite).
  Den alten Freischalt-Code gibt es seit 0.43.0 nicht mehr, auch nicht als
  Übergang: frei ist nur, wer ein Konto hat. Abmelden vergisst das Konto.
- **Konto (Einstellungen):** Oben über der Liste der Bereiche: „(Konto)“ und
  eine Liste als Knopf mit Status und Danach (Abonnent: Läuft bis). Öffnet
  Einstellungen › Konto. Dort ist der Kopf ein Satz zum Zustand (seit 0.43.2
  nur Beta-Bonus). Block „Konto“: E-Mail, Benutzername (Eingabe rechts wie
  „Kürzel“, gespeichert in `user_metadata.username`, gleich auf jedem Mac),
  Angemeldet, Studio („Aktiv“; in der Beta seit 0.45.0 für jedes Konto, sonst „Bald
  dazubuchbar“).
- **Abgleich (0.43.2):** Beim Start und stündlich tauscht die App den
  Refresh-Token beim Server. Lehnt er ab (4xx), ist das Konto abgemeldet und
  die Sperre steht da. Ohne Netz bleibt alles, wie es ist.
- **Vorschau und Lese-Modus (0.40.5 bis 0.43.1):** entfernt. Der Umschalter
  Beta / Abonnent / Lese-Modus / Anmelden erlaubte, zwischen Zuständen zu
  springen. Abo und Lese-Modus kommen mit der Lizenz zurück.
- **Nicht:** Keine Highlights, keine großen Titelblätter. Eingabekästen mit
  Rahmen gibt es nur beim Anmelden und im ersten Start (seit 0.45, wie die
  Webseite), nicht in den Einstellungen.

### 4.27 Safari-Extension (in Arbeit, `safari/`, `http_bridge.rs`)
- **Zweck:** Dieselbe Extension in Safari am Mac. Sie steckt als
  App-Erweiterung in Archery.app (`Contents/PlugIns`) und wird nur in Safari
  eingeschaltet, nicht geladen.
- **Form:** Kein eigener Look. Overlay, Galerie, Zonenleiste, Toasts und
  Panels sind die aus `extension/`, Byte für Byte. Was in Chrome gilt, gilt
  hier (4.17, 4.18, 4.20).
- **Unterschiede:** Ohne laufende App versucht die Extension, Archery zu
  starten. Klappt das nicht, kommt ein Toast („Archery läuft nicht …“) statt
  eines Sicherns in Downloads. ⇧ nach Downloads gibt es nicht (Toast). Die
  Linkkarte hat kein Seitenarchiv (Safari kennt kein pageCapture). Links auf
  `chrome://`-Seiten im Panel tun in Safari nichts.
- **App:** Einstellungen › Mac & Browser › Extension zeigt unter dem
  Chrome-Stand eine Zeile in `.p-block-text`: „Safari: Extension verbunden“
  oder der Weg zum Einschalten (Safari › Einstellungen › Erweiterungen, ohne
  Apple-Signatur vorher Entwickler › „Unsignierte Erweiterungen erlauben“).
  Kein neuer Block, kein Knopf. Seit 0.43.0 ist die Brücke aus
  (`http_bridge::ENABLED`, Sicherheitsprüfung 30.10.), die Zeile fehlt dann.
- **Nicht:** Keine Safari-eigenen Farben, Icons oder Abstände. Was nur in
  Safari anders sein muss, regelt `background.js` (`IS_SAFARI`), nie CSS.


### 4.28 Abstand unter dem Seitenkopf (App 0.42.2)

Zwischen Kopf und Inhalt stehen auf allen Seiten 24–44 px (vorher 40–96 px).
Die Einstellungen bleiben Seitenblatt (ein Vollbild-Versuch in 0.42.2 ist in
0.42.3 zurückgenommen).

## 5. Bekannte Abweichungen

Diese Punkte stehen hier, damit niemand sie für Absicht hält. Behoben werden
sie nur auf ausdrückliches Go.

- **Farbmarken fehlen in der Extension (0.41.0).** Die Galerie der Extension
  zeigt den Innenrand (4.6a) noch nicht; die Brücke liefert `mark` noch nicht mit.
- **Die Extension hat ihre Farben fest im Code und nutzt keine Tokens.**
  Seit 3.22.0 stimmen die Werte mit der App überein (Hellgrund `#f4f4f4`,
  Sekundärtext `#5c5c5c`, Tertiär `#8f8f8f`, Karte schwarz × 0.045/0.07,
  Navigation weißes Glas, Menü `rgba(250,250,250,.97)`, Akzent `#5b7cff`).
  Ecken laufen über `--r-lg/--r-md/--r-sm` mit `data-radius` wie in der App.
  Ziel bleibt: Die Tabellen in Abschnitt 2 werden als Variablen in
  `overlay.css` und `gallery.css` auf `:host` gespiegelt.
- **Tote Regeln nach 0.30:** In `app.css` stehen noch Regeln für
  `data-nav` (Navigation-Positionen), `.p-head-row.is-list` (Ordner als
  Liste) und die Welle beim Neu laden. Sie greifen nicht mehr und können beim
  nächsten Aufräumen weg.
- **`app.css` ist nach Versionen geschichtet.** Komponenten wie `.p-nav` oder
  `.p-dropbar` sind mehrfach definiert, und weiter unten überschreibt. Bei
  Änderungen gilt deshalb immer der letzte Block. Abschnitte sind mit
  `/* ---- <Version>: … ---- */` markiert.
- **`--muted` (`#8a8a8e`) und `--p-dim` (`#8c8c8c`)** sind fast gleich und
  sollen zu einem Token werden.
- **`.btn.primary:hover`** nutzt fest `#dcdcdc` statt eines Tokens.
- **Farben und Etikett-Mono aus 0.42.5 nur in der App.** Extension, Tray und
  Viewer stehen noch auf den Werten davor (Hellgrund `#f4f4f4`).

---

## 6. Prüfen vor dem Commit

- [ ] Die neue Regel nutzt Tokens und keine festen Farben, Dauern oder Radien
      außerhalb der Stufen.
- [ ] Sie funktioniert in Dunkel und Hell sowie mit und ohne `html.mac`.
- [ ] Hervorhebungen laufen über den Ring (4.1) und nicht über Farbe. Es gibt
      keine farbigen Highlights im Text (4.26). Einzige Ausnahme: die
      Farbmarke (4.6a), sie heißt „gehört zusammen“, nicht „ausgewählt“.
- [ ] Entwürfe und neue Flächen sind mit der Rundung „Weich“ gestaltet.
- [ ] Wenn die Komponente auch in der Extension vorkommt, ist sie dort gleich
      oder als Abweichung in Abschnitt 5 notiert.
- [ ] `prefers-reduced-motion` wird respektiert.
- [ ] Diese Datei ist nachgezogen, wenn sich ein Wert oder eine Regel geändert
      hat.


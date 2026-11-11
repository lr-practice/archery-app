# Changelog

## App 0.47.0

- Bildschirmfotos in Archery: Einstellungen › Bibliothek › Screenshots lenkt
  neue Bildschirmfotos des Macs in den Ordner Screenshots der Bibliothek
  statt auf den Schreibtisch. Aus stellt den Ort von vorher wieder her.
- Screenshots hat einen eigenen Reiter in der Navigation und einen Knopf
  neben dem Suchfeld im Tray.
- Screenshots räumt nach 7 Tagen auf wie Unsorted, mit eigenem Schalter.

## App 0.46.0

- Studio: In „Alle Objekte“ und in der Suche filtern „Ohne Urheber“ und „Nur
  intern“ die Rechte. Die Liste zeigt Urheber, Lizenz und in geteilten
  Bibliotheken „Von“ als sortierbare Spalten.
- Geteilte Bibliothek: Das Kürzel der Person, die abgelegt hat, steht fest
  oben rechts am Bild statt nur beim Darüberfahren.

## App 0.45.8

- Wortmarke im Startbild und unter Information auf 90 %, damit sie neben
  Melange nicht zu groß wirkt.

## App 0.45.7

- Startbild und Information zeigen die Wortmarke in Record Laser. Der
  Mac-Build holt sie aus dem privaten Website-Repo, wie Melange.

## App 0.45.6

- Behebt: 0.45.5 blieb nach dem Start grau. Fehlte die Wortmarke im Build,
  lieferte Tauri statt der SVG die ganze Seite aus, und die App setzte sie
  ins Startbild ein. Jetzt wird nur echtes SVG übernommen, sonst bleibt der
  Schriftzug Text.

## App 0.45.5

- Neues App-Icon: das kalligrafische A in Grau auf Dunkel.
- Startbild und Information können die Wortmarke in Record Laser zeigen,
  sobald die Datei im Build liegt. Bis dahin bleibt der Schriftzug Text.

## App 0.45.4

- Der erste Knopf in der Leiste heißt jetzt „Overview“ (DE „Übersicht“) statt
  „Archery“ und sieht aus wie die anderen Reiter. Er führt wie bisher zu den
  Zonen als Karten. Der Name der Bibliothek bleibt oben rechts.

## App 0.45.3

- Bilder und Ordner aus dem Finder landen in der offenen Zone bzw. ihrem
  geöffneten Ordner, auch auf der leeren Fläche unter dem Raster. Bisher
  gingen sie in die Standard-Zone oder als neue Zone auf die oberste Ebene.

## App 0.45.2 · Extension 3.32.1

- Notes und Textkarten zeigen im Raster nur noch den Titel, groß. Ohne Titel
  steht der Anfang des Texts. Bis zu drei verknüpfte Referenzen stehen klein
  darunter.
- Gelb ist etwas weniger grell (`#f2e14b` statt `#ffff00`), in App, Tray und
  Extension.
- Eine Note ohne Verknüpfung steht in der Lightbox wieder in der Mitte.

## App 0.45.1

- Jede Zonenkarte in der Übersicht hat ein festes Titelbild. Ohne eigene Wahl
  ist es die zuerst gesicherte Referenz; Rechtsklick › „Als Titelbild der
  Zone“ legt es von Hand fest. Gruppenkarten nehmen das Titelbild der
  obersten Zone, sonst einer Unterzone, sonst die erste Referenz der Gruppe.

## App 0.45.0

- Erster Start und Anmelden sehen aus wie „Sign in“ auf der Webseite: eine
  Spalte, echte Felder, ein Knopf. Reihenfolge Konto → Bibliothek → Browser,
  oben rechts „(1 / 3) Konto · Bibliothek · Browser“.
- „Anmelden“ legt kein Konto mehr an. Ohne Konto geht es zu „Konto anlegen“
  mit Name, Firma, Website (freiwillig) und dem Haken für die
  Beta-Bedingungen, gespeichert wie auf der Webseite.
- Konten ohne angenommene Beta-Bedingungen sehen beim nächsten Start einmal
  „Noch ein Schritt.“; Abmelden geht immer.
- Browser-Schritt zeigt Extension (verbunden mit Version) und gefundene
  Browser. Der Download-Link führt auf archery-visual.com.
- Studio mit geteilter Bibliothek ist in der Beta wieder für jedes Konto an.

## App 0.44.0 · Extension 3.32.0

- Erste mit Apple Developer ID signierte und notarisierte Version. Beim ersten
  Start fragt macOS nicht mehr nach „Trotzdem öffnen“.
- Interne Namen heißen jetzt Archery: App-Kennung `app.archery.desktop`,
  Brücke zur Extension `app.archery.bridge`, Datenordner `.archery` in der
  Bibliothek, Export `archery-export.json`, Safari-Erweiterung „Archery
  Extension“. Einstellungen und `.archiver` werden beim ersten Start
  übernommen. App und Extension müssen beide neu sein, sonst finden sie sich
  nicht; macOS fragt Rechte einmal neu ab.
- Extension: Schriften sind für Webseiten nur noch über eine Adresse
  erreichbar, die pro Sitzung wechselt (`use_dynamic_url`). Fremde Seiten
  können Melange nicht mehr über die Extension einbinden.

## App 0.43.4 · Extension 3.31.3

- Konto: neue Preise. Lizenz ab 1.0 einmalig 19 €, Beta-Bonus 9 € bis 31.01.
  (vorher 9,99 € und 4,99 €).
- Karten lassen sich wieder verschieben: Google Maps, OpenStreetMap und
  eingebettete Karten öffnen beim Ziehen keine Zonen mehr.
- Text auf Hintergrundbildern lässt sich wieder markieren, Zeichenflächen und
  Vorher/Nachher-Regler über Bildern bleiben bedienbar.
- Zonen kommen von selbst nur noch für Bild, Link oder markierten Text. Das
  eigene Ziehen einer Seite (Kanban, Listen, Web-Apps) und Text oder Links
  von außen bleiben frei; mit ⌥ beim Ziehen kommen die Zonen trotzdem.
- Collection heißt jetzt Stage (Menü „Auf der Stage öffnen“). Snapshots heißen
  `stage_<zone>_<datum>_<zeit>.png`.
- Stage: Das Flächen-Quadrat sitzt in der Leiste wie ein Knopf, offen
  gefüllt statt mit Rahmen über den Rand; Lightbox zeigt die volle Farbe.
- Stage: Die obere Leiste mit den Ansichten hat keine Fläche mehr, nur noch die
  Wörter; die gewählte Ansicht ist gefüllt.
- Stage: Fassungen sind raus. „Als Bild“ heißt jetzt Snapshot.
- Stage: „+5 ähnliche“ steht wieder direkt neben „+ Objekte“ in der Leiste
  und nicht mehr im Glas darüber.

## App 0.43.3 · Extension 3.31.2

- Konto: Texte aufs Lizenzmodell (9,99 € einmalig, zwei Macs, alle 1.x-Updates;
  Beta-Bonus: Lizenz für 4,99 € bis 31.01.; Studio eigenes Abo).
- Sichtbare Ordner heißen Archery: `~/Archery Demo`, `~/Library/Logs/Archery/archery.log`,
  `Downloads/Archery-Transfer`. Demo und Protokoll werden beim ersten Zugriff
  übernommen. Die App nimmt weiter auch `Archiver-Transfer` an, damit ältere
  Extensions funktionieren.
- Einstieg: Extension-Hinweis für alle Chromium-Browser (Chrome, Brave, Edge, Arc, Vivaldi).
- Texte aus dem Copy-Deck: Anmelden „Deine Bibliothek.“, neuer Claim unter Information, Einstieg „Browser-Extension verbinden.“, neue Extension-Beschreibung (zugleich Store-Kurztext).

## App 0.43.2 · Extension 3.31.1

- Einstellungen › Labor ist weg. Zonenkarten laufen unten nicht mehr aus, die
  Video-Vorschau ist fest „Skim“. Drop-Feedback steht jetzt unter Darstellung.
- Konto abgeriegelt: Der Umschalter Beta / Abonnent / Lese-Modus / Anmelden
  ist entfernt. Beim Start und stündlich prüft die App die Anmeldung beim
  Server; ein gesperrtes oder gelöschtes Konto ist danach abgemeldet.
- Benutzername im Konto, gespeichert im Konto selbst (gleich auf jedem Mac).
- Studio ist ein eigenes Abo und steht abgesetzt unten in den Einstellungen.
  Ohne Freigabe sind Rechte, Paket und geteilte Bibliothek aus; eine geteilte
  Bibliothek öffnet sich dann nicht. Das Feld „Dein Name“ heißt dort „Kürzel“,
  ohne eigenes gelten die Initialen des Benutzernamens.
- Das Testfeld „Name“ unter Information ist entfernt.
- Schriftzug „Archery“: das y steht 35/1000 em weiter weg.
- Extension: Ohne App gelten die Ecken „Weich“ wie ab Werk in der App (vorher „Rund“).

## App 0.43.1

- rustls und webpki aktualisiert (bekannte Lücken geschlossen).
- Sicherheit (Prüfung 30.10.): Die HTTP-Brücke für Safari ist aus, bis die
  Safari-Extension gekoppelt werden kann; ohne Geheimnis hätte jedes Programm
  auf dem Mac sie ansprechen können. Der Safari-Hinweis in den Einstellungen
  ist so lange ausgeblendet.
- Feedback geht an feedback@archery-visual.com.
- Die alten Beta-Freischalt-Codes gelten nicht mehr. Wer nur damit
  freigeschaltet war, meldet sich einmal mit seiner E-Mail an.

## App 0.43.0 · Extension 3.31.0

- Archiver heißt jetzt Archery: Schriftzug, Fenstertitel, Menü, alle Texte
  (Deutsch und Englisch), Handbuch, PDF-Fußzeile, DMG und Update-Pakete.
- Was intern bleibt, damit bestehende Libraries, Einstellungen und die
  Verbindung zur Extension beim Update erhalten bleiben: Bundle-ID
  `app.archiver.desktop`, Brücke `app.archiver.bridge`, der verborgene Ordner
  `.archiver` in jeder Library, `Downloads/Archiver-Transfer`, die Protokolle
  unter `~/Library/Logs/Archiver` und das Update-Repository Archiver-Releases.
- Ein per Auto-Update aktualisiertes Programm behält den Dateinamen
  `Archiver.app`, zeigt aber überall Archery. Wer neu aus dem DMG installiert,
  bekommt `Archery.app` und löscht das alte `Archiver.app`.

## Extension 3.30.0

- Markierter Text landet bei laufender App als Note (.md) statt als .txt in
  der Zone, mit Seite und Seitentitel als Herkunft. Dort ist er bearbeitbar und
  verknüpfbar. Ohne App bleibt es das Bild.

## App 0.42.8

- Notes und Textkarten zeigen in den Karten wieder ihren Text statt einer
  leeren Fläche.
- Labor › Ausblenden greift nur noch, wenn die Bilder einer Karte unten
  abgeschnitten sind. Karten mit wenigen Bildern bleiben ruhig, ohne Kante
  und Farbschein. Der Glas-Blur hat am unteren Rand keine Kante mehr.

## App 0.42.7

- Karten zeigen kleine Bilder wie die Index-Zeilen, als Raster über die volle
  Kartenbreite. Rechtsklick auf ein Bild › „Als Titelbild der Zone“: das Bild
  steht groß oben, darunter eine Reihe der ersten Bilder.
- Der Index besteht nur noch aus Zeilenkarten über die volle Breite, die
  Active Zones zuerst.
- Labor: Die alten Kartenvarianten sind weg. Neu ist „Ausblenden“: Die
  späteren Bilder einer Karte laufen unten per Farbverlauf oder Glas-Blur aus.
- Auch die Bilder der Index-Zeilen reichen bündig bis zum rechten Rand.

## App 0.42.6 (Vorabversion)

- Textkarte und Notiz sind eins: „Note“. Eingefügter Text (⌘V in einer Zone)
  wird eine Note (.md) statt .txt.
- Verknüpfte Bilder stehen in voller Höhe direkt neben der Note, wie die
  Doppelseite beim Bildtext; „+“ am Ende verknüpft weitere.
- Rechtsklick auf ein oder mehrere Bilder › „Note zu diesem Bild“ bzw. „Note zu
  n Objekten“ legt eine Note daneben an, gleich verknüpft.
- Ein Bild mit Note zeigt sie in der Lightbox an der Stelle des Bildtexts
  (Knopf „Note“, Taste T); Klick auf das Blatt öffnet die Note.
- Im Raster trägt ein Bild mit Note den Punkt in der Farbe der Note. Der Punkt
  ist größer (14 statt 10 px).
- Zitat behält Herkunft: Die Quelle steht im Fuß der Note und führt zur Stelle
  auf der Seite. Sie bleibt jetzt auch nach dem Bearbeiten erhalten.
- Suche findet Bilder auch über den Text ihrer Note.
- Karten zeigen ab Werk kleine Bilder wie die Index-Zeilen, eigenes Titelbild
  per Rechtsklick. Die Ansicht „Übersicht“ heißt jetzt „Mix“.

## App 0.42.5 (Vorabversion)

- Farben aus der Figma-Datei: Fenster Dunkel #1a1a1a, Hell #fafafa. Karten
  Dunkel neutral grau statt weiß getönt, Hell #f2f2f2. Navigation Hell #ebebeb
  deckend. Tertiärtext Dunkel #666664, Hell #b2b2b2.
- Etiketten, Chips, Anzahl und Datum unter Karten, Quelle auf Text-Karten und
  die Notiz-Kennung wieder in Geist Mono 11 px, Versalien.

## App 0.42.4

- Hell kühler und sauberer: Grund #f4f5f7, Karten und Flächen #e9ebee, Grau
  leicht bläulich statt gelbgrün. Das Seitenblatt hat auf dem Mac denselben Ton.

## Extension 3.29.8

- Ablegen wie in der App: Das Bild fliegt als Miniatur vom Zeiger in die
  obere rechte Ecke der Zielkarte, die Leiste steht kurz und blendet dann
  ruhig aus. Die aufgehellte Zielkarte blendet jetzt mit den anderen aus,
  statt hell stehen zu bleiben und dann wegzuspringen.

## App 0.42.3 (Vorabversion)

- Einstellungen wieder als Seitenblatt rechts.
- Labor › Kopf ist aufgelöst: Kopf und Schrift stehen fest auf „A Leise“
  (Melange Semi-Light), ohne Varianten, Größe, Farben oder Schriftwahl.

## App 0.42.2 (Vorabversion)

- Neue Typografie „A Leise“: Melange Semi-Light für Köpfe, Namen, Text-Karten
  und Notizen, Etiketten in Melange Medium statt Geist Mono. Köpfe ruhiger
  (30 px), die Ordnung entsteht über Grau statt über Größe.
- Hell neu: warmes Hellgrau (#f3f4f1), Karten und Flächen eine Stufe dunkler
  statt weiß, Schrift fast schwarz, Grau mit mehr Kontrast.
- Dunkel: Grau etwas heller, damit der dünne Schnitt trägt.
- Einstellungen als Vollbild: links Kopf und Rückweg, rechts die Bereiche als
  ruhige Liste mit Haarlinien.
- Weniger Abstand zwischen Kopf und Zonen auf allen Seiten.
- Geist lässt sich unter Labor › Kopf › Schrift zurückholen.

## App 0.42.1 (Vorabversion)

- Der Atlas heißt jetzt Collection.
- Groß ansehen: hohe Bilder liefen unten aus dem Fenster. Jetzt passt jedes
  Bild ganz hinein.
- Zwei Leisten: oben die neun Ansichten und Mischen, unten die Werkzeuge.
- Abstand ist raus, es gilt fest Mittel (26 px).
- Gruppen sind raus.
- „+ Referenz“ heißt „+ Objekte“. „Zonen ▾“ klappt das Glas auf und zeigt die
  Zonen als Karten wie im Tray; ein Klick öffnet eine Zone.
- Ziehen auf ein anderes Bild tauscht die Plätze, die Ansicht bleibt. Nur in
  Frei bleibt liegen, was man zieht.
- Größer und kleiner direkt am Bild beim Überfahren (− +), nicht mehr im Menü
  oder in der Leiste.
- Fassungen haben einen eigenen Knopf „Fassungen ▾“.
- Neu: Linien. Der Knopf lässt von jedem Bild eine Linie zu seinem farblich
  ähnlichsten stehen. Sie kommen mit „Als Bild“ ins PNG, auch der Faden im
  Strang.
- Als Bild sichert unter einem festen Schema:
  `collection_<zone>_<jjjj-mm-tt>_<hhmm>.png`.
- Beim Überfahren steht kein „Farbe 62 %“ mehr unter den Bildern.

## App 0.42.0 (Vorabversion)

- Die Wolke heißt jetzt Atlas, nach Warburgs Bildatlas.
- Eine Leiste statt zwei: „Ansicht ▾“ öffnet ein Feld mit den neun Ansichten,
  Abstand, Mischen und den Fassungen, das Quadrat daneben die Flächen. Dazu
  + Referenz, Autoplay, Als Bild, Ausblenden und Schließen. „+5 ähnliche“
  steht jetzt im Glas von + Referenz neben der Suche.
- Rechtsklick auf ein Bild: Groß ansehen, In die Mitte oder Vergleichen,
  Gruppe bilden oder lösen, Größe S M L XL, Eigener Atlas, In der Zone zeigen,
  Aus dem Atlas nehmen.
- In einer Zone mehrere Bilder wählen, Rechtsklick › Im Atlas öffnen: das
  erste ist die Mitte, die anderen liegen gleich daneben.
- Autoplay (P): Im Fokus wechselt die Mitte alle 3,2 Sekunden, mit Auswahl nur
  durch die Auswahl. Ein Klick auf die Fläche hält an.
- Leiste ausblenden mit H, zurück mit H oder Esc.
- Als Bild: die ganze Anordnung als PNG, lange Seite bis 4000 px, mit
  Sichern-Dialog.
- Fassungen: beliebig viele benannte Stände je Atlas sichern und wieder
  auslegen. Rechtsklick auf eine Fassung löscht sie.
- Die Linien beim Überfahren bleiben und sagen jetzt, was sie verbinden:
  unter den drei verbundenen Bildern steht „Farbe 94 %“, der Anteil gleicher
  Farben.

## App 0.41.1 (Vorabversion)

- Wolke, Gruppen: Ein Klick auf ein Bild einer Gruppe wählt die ganze Gruppe,
  ⌥-Klick nur das eine. Ziehen bewegt die Gruppe gemeinsam. Im Raster beginnt
  jede Gruppe eine eigene Zeile, in Spalten stehen ihre Bilder hintereinander,
  in Stapel ist jede Gruppe ein eigener Stapel. Die Kennung „G1“ steht jetzt
  auch in Raster, Spalten, Spektrum und Strang.

## App 0.41.0 (Vorabversion)

- Farbmarken: Objekte in einer Zone bekommen eine von fünf Farben (Rot, Grün,
  Violett, Cyan, Gelb), über Rechtsklick › Farbmarke oder ⌥1–5, ⌥0 nimmt sie
  weg. Markierte Objekte tragen einen 4 px Innenrand in ihrer Farbe. Im Kopf
  der Zone steht je Farbe ein Quadrat mit Anzahl, ein Klick zeigt nur diese.
  Die Marke liegt in der Bibliothek (metadata.json) und geht mit auf andere Macs.
- Wolke neu gedacht: neun Ansichten (Wolke, Fokus, Raster, Spalten, Spektrum,
  Strang, Stapel, Vergleich, Frei), die Objekte gleiten beim Wechsel an ihren
  Platz. Mehrere wählen mit ⇧ oder ⌘, Lasso auf freier Fläche, ⌘A. Die Leiste
  unten zeigt dann Größe, Groß, Gruppe, Vergleichen, In die Mitte und Entfernen.
  Leertaste oder Doppelklick zeigt groß, ← → blättert. G bildet eine Gruppe,
  die zusammenhält. Beim Überfahren zeigen Fäden die drei ähnlichsten. Ziehen
  legt frei ab. Die Fläche gibt es in Lightbox, Schwarz, Weiß, den fünf Farben
  und „Auto“ aus dem offenen Bild. Orbit, Farbe und Konstellation gehen in
  Fokus, Spektrum und Strang auf. Doppelklick macht ein Bild nicht mehr zur
  Mitte einer neuen Wolke, dafür bleibt Enter.

## App 0.40.25 (Vorabversion)

- Zonenleiste, Observer und Tray holen nicht mehr das ganze Archiver-Fenster
  nach vorne. War Archiver mit ⌘H ausgeblendet, kommt nur die Leiste. Öffnet
  der Observer ein Bild aus dem Finder, bleibt das Hauptfenster weg, statt
  hinten mit aufzugehen; schließt der Observer, liegt es wieder dort, wo es
  war. Ein „Reopen“ von macOS direkt beim Öffnen holt es auch nicht mehr.

## App 0.40.24 (Vorabversion)

- Zonenleiste am ganzen Mac mit echtem Glas: Hinter jeder Karte liegt jetzt
  die macOS-Unschärfe, so wie in der App. Vorher wirkten die Karten wie graue
  Folie.
- Einstellungen › Mac & Browser: Die Zonen kommen „Mit ⌥“ (Standard),
  „Immer“ (sobald Dateien gezogen werden) oder gar nicht.
- Nach dem Ablegen bleibt die Leiste 0.7 s stehen und blendet in 0.28 s aus,
  in der App und am ganzen Mac. Die Miniatur bleibt Standard, die Wahl
  „Danach“ im Labor ist weg.

## App 0.40.23 (Vorabversion)

- Zonenleiste am ganzen Mac: Ein Drop landet wieder in der Zone unter dem
  Zeiger. Seit 0.26 wurde die Position auf Retina-Bildschirmen halbiert, der
  Drop traf deshalb oft die Karte links daneben (vermutlich der alte Fehler
  „sortiert falsch ein“). Auch die Miniatur kam von der falschen Stelle.
- Die Varianten aus Labor › Ablegen gelten jetzt auch für die Leiste in der
  App (Ziehen aus der App heraus oder von außen hinein). In der App steht
  dabei „Abgelegt“.
- Labor › Ablegen: Die sieben Varianten stehen in zwei Reihen, statt aus dem
  Blatt zu laufen. Die Überschrift heißt auf Englisch „Drop feedback“.

## App 0.40.22 (Vorabversion)

- Zonenleiste am ganzen Mac: Die Karten haben jetzt das Format 2:1 wie die
  Zonenkarten in der App. Die Leiste ist dadurch flacher (auf einem 13-Zoll-Mac
  192 statt 280 px).
- Ablegen ohne blaues Glühen: Das abgelegte Bild fliegt als Miniatur in die
  obere rechte Ecke der Karte, bei mehreren Dateien mit „+N“. Danach geht die
  Leiste schnell (0.35 s stehen, 0.18 s ausblenden). Lässt man ohne Drop los,
  ist sie sofort weg.
- Neu im Labor › Ablegen: Miniatur, Haken, Ring, Zähler, Dateiname, Nachbild
  und das alte Glühen zum Vergleichen. Dazu „Danach“: schnell weg oder kurz
  stehen.

## App 0.40.21 (Vorabversion)

- Zonenleiste am ganzen Mac ist zurück: Dateien irgendwo ziehen (Finder,
  Schreibtisch, Mail …) und ⌥ halten, dann stehen unten die vier Active Zones
  in voller Höhe. Ohne ⌥ kommt nichts. Ab Werk an, auch wenn die alte Leiste
  aus war; ausschalten unter Einstellungen › Mac & Browser.
- Schneller und verlässlicher: Die Karten liegen schon bereit, bevor gezogen
  wird, statt erst beim Einblenden zu laden. Kein Streifen mehr und keine
  Fenster-Animation beim Hochfahren. Ein Drop in einer Lücke zwischen zwei
  Karten landet in der Zone darunter, statt verloren zu gehen.

## App 0.40.20 (Vorabversion, Safari-Extension)

- Die Extension läuft auch in Safari am Mac, aus demselben Code wie in
  Chrome. Sie steckt als App-Erweiterung in Archiver.app und wird unter
  Safari › Einstellungen › Erweiterungen eingeschaltet (ohne Apple-Signatur
  vorher Entwickler › „Unsignierte Erweiterungen erlauben“).
- Safari spricht mit der laufenden App über HTTP auf 127.0.0.1 (Port 47613,
  sonst 47614/47615). Die App nimmt dort nur Anfragen von Safari-Extensions an
  (Origin, Host und Kopf „X-Archiver“ werden geprüft). Bilder kommen als Bytes
  („save_data“) statt über Downloads/Archiver-Transfer.
- Läuft Archiver nicht, startet die Extension die App und legt danach ab.
  Klappt das nicht, sagt ein Hinweis Bescheid; in Downloads wird in Safari
  nichts gesichert. Linkkarten haben in Safari kein Seitenarchiv.
- Einstellungen › Mac & Browser zeigt, ob Safari verbunden ist, oder wie man
  die Extension einschaltet.
- Build: `safari/build.sh` erzeugt die Erweiterung mit
  `safari-web-extension-converter`; der Mac-Workflow baut sie vor der App und
  legt sie nach `Contents/PlugIns`.

## Extension 3.29.7

- Die Zonenleiste beim Ziehen im Browser steht gleich in voller Höhe. Die
  Karten ragen nicht mehr erst als 90-px-Streifen herein.

## App 0.40.19 (Vorabversion)

- Seitenkopf ab Werk Variante 05 (Satz im Fluss) in Größe M, Titel voll, Satz
  hell. Wer im Labor schon etwas gewählt hat, behält seine Wahl.

## App 0.40.18 (Vorabversion)

- Grundschrift wieder Geist. Melange lässt sich unter Einstellungen › Labor
  zurückholen.
- Labor › Kopf: vierzehn Mechaniken für den Seitenkopf (01 Geist-Original bis
  14 Hochkant), dazu Größe S–XL und die Farbe von Titel und Satz (Voll, Grau,
  Hell, Blau). Ab Werk 01.

## App 0.40.17 (Vorabversion)

- Liegt die Bibliothek in einem geteilten iCloud-Ordner, fragte macOS bei
  jedem Start „Möchtest du diesen geteilten Ordner wirklich bewegen?“, sobald
  „Zuletzt gelöscht“ Einträge älter als 7 Tage in den Papierkorb legte. Der
  Papierkorb wird jetzt direkt über das System angesprochen statt über den
  Finder; die Abfrage entfällt.

## App 0.40.16 (Vorabversion)

- KMR Melange Grotesk läuft überall mit Standard-Laufweite (0): Seitenköpfe,
  Sätze, Kartentitel, Navigation, Tray, Zonenleiste und Observer. Vorher
  lagen die Werte zwischen −0.025em und +0.01em.

## App 0.40.15 (Vorabversion)

- Boards und Projekte als Funktion gestrichen. Archiver ist eine visuelle
  Bibliothek zum Sammeln von Referenzen; Präsentationen und
  Projektmanagement gehören nicht mehr dazu. Weg sind die beiden Bereiche in
  der Leiste, ihre Schalter unter Einstellungen › Studio, „Aufs Board …“ im
  Rechtsklick und die Einträge im Handbuch.
- Auf der Platte bleibt alles, wie es ist: ein Ordner `Boards` mit
  `.board`-Dateien und der Projekte-Ordner werden nicht angefasst und nicht
  gelöscht. Der Ordner `Boards` gilt jetzt wie jeder andere Ordner.
- Jeder Ordner kann jetzt Bibliothek sein; die Sperre gegen Ordner, die nach
  Projekten aussehen, entfällt.

## App 0.40.14 (Vorabversion)

- Seitenköpfe fest in Regular und leicht gesperrt: die Übersicht in L
  (34 bis 56 px), alle anderen Seiten in M (28 bis 42 px). Der Schrift-Schalter
  im Labor ist damit entschieden und wieder weg.

## App 0.40.13 (Vorabversion)

- Labor › Schrift: Kopfgröße der Seiten in vier Stufen (S, M, L, XL) und
  Schnitt Medium oder Regular, für Seitenköpfe und die Sätze in Konto und
  Anmelden.

## App 0.40.12 (Vorabversion)

- Neuer Bereich Einstellungen › Labor zum Ausprobieren. Dort lässt sich
  umschalten, wie die Active-Zone-Karten ihren Inhalt zeigen (A bis H: Heute,
  Skim, Streifen, Kontaktbogen, Stapel, Aufklappen, Lauf, Blättern) und wie
  Videos beim Darüberfahren vorschauen (Daumenkino, Skim, Abspielen, Aus).
- Übersicht: jede Index-Zeile ist jetzt eine Karte mit eigener Fläche und ein
  einziges Klickziel. Die Bilder beginnen auf der Kante der zweiten Active Zone.

## App 0.40.11 (Vorabversion)

- Übersicht: oben wieder immer die vier Active Zones als Karten, darunter die
  übrigen Zonen als Kontaktbogen.
- Die Ansicht „Vorschau“ auf der Startseite ist entfernt. Es bleiben Übersicht,
  Karten und Index.
- Seitenköpfe etwas kleiner (28 bis 42 px) und nicht mehr so eng gesetzt.

## App 0.40.10 (Vorabversion)

- Übersicht als Kontaktbogen: je Zone eine Zeile, links Name und Anzahl,
  rechts die Bilder klein und dicht. Ab 24 Bildern „+N“, das die Zone öffnet.
- Ein Kopf für alle Seiten: Übersicht, Zone, Alle Objekte, Suche und Zuletzt
  gelöscht zeigen den Titel groß, darunter den Satz gedimmt, mit gleichem
  Abstand oben und unten.

## App 0.40.9 (Vorabversion)

- Anmelden neu: ein Satz, darunter ein einziges Feld in Satzgröße auf einer
  Linie, rechts ein runder Pfeil. Die Zeilen „Weg“ und „Beta“ sind weg. Beim
  Code nennt der Satz die Adresse, darunter „(Neuer Code)“ und „(Andere E-Mail)“.
- Übersicht mit Plakat-Kopf: „Übersicht“ groß in voller Farbe, darunter die
  Zahlen gedimmt. Viel Raum darunter, die vier Active Zones stehen am unteren
  Fensterrand, der Rest kommt beim Scrollen. „seit …“ ist weg.
- Active Zones schon ab mittlerer Fensterbreite vierspaltig.

## App 0.40.8 (Vorabversion)

- Einstellungen aufgeräumt: oben das Konto, darunter fünf statt zehn Bereiche.
  Bibliothek (mit Dateinamen), Darstellung (Ansicht, Aussehen, Glas, Start und
  Ton), Mac & Browser (Extension, Tray, Zonenleiste, Vorschau-Formate), Studio
  und App.
- Version und Updates stehen jetzt unter Einstellungen › App, mit „Nach Updates
  suchen“. Information (⌘I) zeigt nur noch die App und deine Sammlung.
- Der alte Beta-Code lässt sich nicht mehr eingeben. Wer schon freigeschaltet
  ist, bleibt es, bis er sich mit E-Mail anmeldet oder abmeldet.
- Anmelden nimmt Codes mit 6 bis 8 Ziffern und nennt bei Fehlern den Grund.

## App 0.40.7 (Vorabversion)

- Anmelden mit Konto: E-Mail eingeben, den 6-stelligen Code aus der Mail
  eintippen, fertig. Dasselbe Konto wie auf archiver-app.com. Ein Passwort gibt
  es nicht.
- Der Beta-Code funktioniert weiter, über „(Beta-Code)“ im Anmelde-Fenster.
- Einstellungen › Konto zeigt die angemeldete E-Mail. Abmelden meldet das Konto
  auf diesem Mac ab.

## App 0.40.6 (Vorabversion)

- Anmelde-Fenster, Konto und Lese-Modus sind jetzt ruhig wie die
  Einstellungen: ein Satz als Überschrift, darunter gruppierte Zeilen. Keine
  farbigen Markierungen mehr.
- Im Anmelde-Fenster ist der Code eine Zeile der Liste, „Freischalten“ steht
  darunter.
- Überschriften in der Übersicht, in Zonen, Alle Objekte und Zuletzt gelöscht
  sind größer (34 px statt 24 px).
- Verknüpfte Ordner von außerhalb der Bibliothek heißen beim Entfernen jetzt
  „Entkoppeln“. Der Ordner auf dem Mac bleibt, gelöscht wird nichts.

## App 0.40.5 (Vorabversion)

- Einstellungen › Konto › Ansicht: Vorschau als Beta, als Abonnent (Jahresabo
  aktiv, mit Abo, Geräten und Zahlung), im Lese-Modus oder als Anmelde-Fenster. Es wird nichts
  gebucht oder gesperrt.
- Einstellungen › Konto › Abmelden vergisst den Freischalt-Code auf diesem Mac.
  Danach steht das Anmelde-Fenster wie beim ersten Start da.

## App 0.40.4 (Vorabversion)

- Die Sperre ist jetzt ein Titelblatt: „Dein Archiv.“ groß und farbig markiert,
  darunter der Beta-Hinweis und das Feld für den Code.
- Oben in den Einstellungen steht eine Konto-Karte mit dem Beta-Bonus. Sie
  öffnet Einstellungen › Konto mit dem Plan für 1.0 und einer Vorschau des
  Lese-Modus (nur zum Ansehen, es wird nichts pausiert).
- Ecken sind ab Werk „Weich“ (4 px) statt „Rund“. Wer selbst gewählt hat,
  behält seine Wahl.

## App 0.40.3 (Extension 3.29.6, Vorabversion)

- Grundschrift ist jetzt KMR Melange Grotesk in App, Tray, Observer,
  Zonenleiste, Viewer, Palette und Extension (Popup, Dashboard, Galerie,
  Overlay). Geist Mono bleibt für Etiketten. Die Schrift ist eingebettet und
  muss nicht installiert sein.

## Extension 3.29.5

- Regler und Knöpfe über Videos (etwa die Lautstärke) lassen sich wieder
  bedienen, statt als Bild gezogen zu werden.
- Links und Knöpfe über Bildern (etwa in Mediatheken) bekommen den Klick wieder.
- Bilder und Dateien, die von außen in den Browser gezogen werden, blenden
  keine Zonen mehr ein; die Seite bekommt sie selbst (Upload-Felder).

## Extension 3.29.4

- Zonenleiste beim Ziehen: Rand 10 px statt 14 px, 8 px statt 10 px zwischen den
  Zonen, gleich wie in der App 0.40.2.

## App 0.40.2 (Extension 3.29.3, Vorabversion)

- Rückfragen (Löschen, Installieren, Neues Board, Umbenennen, Neues Projekt)
  folgen jetzt der Rundung aus Einstellungen › Aussehen, statt fest rund mit
  Pillen-Knöpfen.
- Das ≡-Menü ist so breit wie sein Inhalt und nicht mehr so breit wie die
  Navigation.
- Board-Editor: In der Leiste unten heißt „Bilder“ jetzt „Objekte“.
- Alle Knöpfe, Importieren, die Willkommen-Schritte, Schritt 3, Tour, Sperrbild
  und die Felder im PDF-Ersteller folgen ebenfalls der Rundung; die alten
  gestrichelten Rahmen sind weg.
- „1 Objekt umbenennen“ statt „1 Objekte umbenennen“.
- Zonenleiste beim Ziehen (App, am ganzen Mac, Extension): etwas weniger
  Abstand zum Rand und zwischen den Zonen.

## App 0.40.1 (Extension 3.29.3, Vorabversion)

- Die Ansicht „(Form)“ (Ring, Sonnenblume, Karussell, Fächer) ist wieder
  entfernt. Wer sie gewählt hatte, landet im Raster.

## App 0.40.0 (Extension 3.29.3, Vorabversion)

- Neu: Projekte ersetzt Works. Ein eigener Ordner „Projekte“, jedes Projekt
  mit der Vorlage 01_Briefing bis 08_Admin. Übersicht mit Phase und nächster
  Frist, im Projekt links die Ordner, in der Mitte der Inhalt, rechts das
  Projektblatt mit Kunde, Kürzel, Start, Frist, Abgabe, offenen Punkten und
  Notiz. „Abschließen“ verschiebt das Projekt nach Archiv/Jahr, „Wieder
  öffnen“ holt es zurück. Dateien von außen hineinziehen kopiert sie in den
  Ordner. Einschalten unter Einstellungen › Studio.
- Works ist entfernt. Bestehende Works-Ordner bleiben unberührt.
- Neu: Ansicht „(Form)“ neben Raster, Original und Liste. Die Objekte einer
  Zone (oder einer Suche) bilden eine Figur: Ring mit dem Namen in der Mitte,
  Sonnenblume, Karussell (dreht langsam) oder Fächer. Klick öffnet die
  Lightbox.

## App 0.39.1 (Extension 3.29.3, Vorabversion)

- Neu zum Ausprobieren: die Wolke. In der Lightbox „Wolke“ oder W. Am
  offenen Objekt lassen sich weitere Referenzen andocken („+ Referenz“ mit
  Suche, „+5 ähnliche“), die sich per Physik anziehen und Abstand halten.
  Sechs Modi: Wolke, Orbit, Strang, Farbe, Konstellation, Raster. Abstand
  Eng, Mittel, Weit. Jedes Objekt in vier Größen (S M L XL, 1–4, − +).
  Ziehen wirft, ⌫ entfernt, Doppelklick öffnet die Wolke der Referenz.
  Die Wolke merkt sich ihre Mitglieder je Objekt.
- Verknüpfte Ordner außerhalb der Bibliothek lassen sich nur noch trennen.
  Ordner und Unterzonen darin bieten kein Löschen mehr, und Archiver löscht
  dort nichts.
- Neue-Zone-Karte: ein großes Plus in der Mitte statt „+ Neue Zone“.
- Alle Objekte: Art, Farbe und Zeit stehen in einer Zeile, „(Mehr)“ öffnet
  Form, Format, Doppelt, Motive und Quellen als Tabelle. Gesetzte Filter
  daraus stehen als Pille in der ersten Zeile, solange „Mehr“ zu ist.
- Zonen mit Unterordnern: Statt „(Mosaik) (Abschnitte)“ gibt es einen Chip
  „(Segmente)“ zum An- und Ausklicken.

## App 0.39.0 (Extension 3.29.3, Vorabversion)

- Neu: Boards, ein eigener Bereich für Präsentationen aus Bildern der
  Bibliothek, in der Navigation nach Works. Einstellungen › Studio › Boards,
  ab Werk aus und nur in Vorabversionen schaltbar.
- Ein Board ist eine lesbare Datei `Boards/Name.board` in der Bibliothek,
  sichtbar im Finder. Bilder darin sind Verweise (Hash und Pfad), keine
  Kopien. Der Ordner `Boards` erscheint nicht als Zone.
- Übersicht mit allen Boards als Karten (erste Folie als Vorschau), „Neues
  Board“, Rechtsklick für Umbenennen, Im Finder zeigen und Löschen
  (Papierkorb).
- Aus einer Zone: Auswahl, Rechtsklick › „Aufs Board …“ legt die Bilder als
  neue Folie auf ein bestehendes oder neues Board.
- Editor: nur die Folie und unten eine Leiste. Folien (F), Bilder (B) und
  Gestalten (E) öffnen sich als Glas, Esc schließt sie. Zwölf Vorlagen
  (Index, Paar, Aussage, Raster, Einzel, Farbe, Titel, Streifen,
  Kontaktbogen, Diptychon, Zitat, Index-Liste), Raster mit Spalten, Rand und
  Abstand, Schrift, Farbe je Folie, Palette aus den Bildern, Kopfzeile wie
  Sort Office, Text direkt auf der Folie (T), Farbfelder. Ziehen und
  Skalieren mit Smart Guides, ⌥ hält sie an. ⌘Z, ⌘⇧Z, ⌘D, ⌫, Pfeiltasten.
- Präsentieren (P): Folien im Vollbild, ← → blättern, esc zurück.
- Noch nicht drin: Ausgabe als PDF oder Bild, Ziehen aus dem Finder oder
  Browser direkt aufs Board, Ausschnitt.

## App 0.38.2 (Extension 3.29.3, Vorabversion)

- Bibliothek und Works lassen sich nicht mehr verwechseln: Ein Works-Ordner
  (oder einer, in dem Projekte mit `brief.md` liegen) öffnet sich nicht als
  Bibliothek, eine Bibliothek nicht als Works-Ordner, auch keine der anderen
  Bibliotheksplätze. Vorher landete ein Works-Ordner als Bibliothek auf dem
  aktiven Platz und verdrängte die eigentliche Bibliothek aus der Liste.
- Works: Ein Sonderzeichen kurz vor dem Ende eines Dateinamens (etwa „–“ in
  „Angebot – Auftrag.eml“) brachte den Scan zum Absturz, die Projektliste
  blieb leer.

## App 0.38.1 (Extension 3.29.3, Vorabversion)

- ⌘I öffnet Information, auch in Works; im Menü ≡ steht das Kürzel dahinter.
- Information › Updates: Kanal Beta oder Vorab als Schalter (bisher nur per
  ⌥-Klick auf die Versionsnummer) und „Version wählen“: lädt die Liste aus
  Archiver-Releases und installiert jede Freigabe, auch eine ältere. Vorher
  fragt ein Dialog. Sichtbar mit Vorab-Kanal oder nach ⌥-Klick auf die Version.
- Build: Jede Vorabversion bleibt zusätzlich unter `vorab-X.Y.Z` liegen, damit
  sie später wählbar ist, ab 0.38.0.
- Works heißt jetzt „Works“ statt „WORKS“.

## App 0.38.0 (Extension 3.29.3, Vorabversion)

- Neu: Works, ein eigener Bereich für laufende Kundenprojekte, gleich nach
  „Archiver“ in der Navigation. Einstellungen › Studio › Works, ab Werk aus
  und nur in Vorabversionen schaltbar. Die Bibliothek bleibt unberührt.
- Works liest einen eigenen Ordner: Kunde, Projekt, darin 00 Post, 01 Brief,
  02 Recherche, 03 Layout, 04 Reinzeichnung, 05 Übergabe, beim Kunden
  `_Assets`. Eigene Ordnernamen werden am Wort erkannt („Entwurf“, „Druckdaten“).
- Projektliste mit Phase, Termin und offenen Rechnungen, nach dem nächsten
  Termin sortiert, laufend oder abgeschlossen (`status` in `brief.md`).
- Projekt: Übersicht mit neuesten Ständen, Recherche, Brief, Post und
  Verlauf. Die Phase ist der höchste Phasenordner mit Inhalt; Archiver
  verschiebt dafür nie eine Datei.
- Stände: Fassungen einer Datei (V1, V2, KORR, RZ) in einer Reihe, Vergleich
  mit Linie (V), „Neuer Stand“ kopiert mit der nächsten Nummer in denselben
  Ordner (N), Vorschau für INDD, AI, PSD und PDF über Quick Look.
- Post: Mails (.eml, .emlx) mit Betreff, Absender und Datum, Angebote und
  Rechnungen mit Nummer und Betrag aus dem Dateinamen, „offen seit“, Summen,
  „bezahlt“ per Klick (gemerkt in `Works/.archiver/works.json`).
- Brief: `brief.md` lesen und in Works bearbeiten (Termin, Status, nächster
  Schritt, Ansprechpartner). Neues Projekt legt alle Ordner und den Brief an.
- Ablegen aus Finder oder Tray kopiert in die Phase des Reiters unter dem
  Zeiger, ziehen aus Works nach Mail oder InDesign geht wie in Archiver.

## App 0.37.3 (Extension 3.29.3)

- Information › App: Feld „Name (Test)“. Ersetzt den Schriftzug „Archiver“
  in der Navigation, auf Start- und Sperrbild, oben in Information und im
  Fenstertitel, nur auf diesem Mac. Leer stellt „Archiver“ zurück.

## Extension 3.29.3 (App 0.37.2)

- ⌥-Klick auf ein Bild oder Objekt mit Link führt nicht mehr auf die nächste
  Seite: Der Link wird ignoriert, das Bild kommt in die Markierung. Gilt auch
  für Seiten, die schon beim Drücken weiterspringen.

## Extension 3.29.2 (App 0.37.2)

- Neues Icon: „Ar“ weiß auf Schwarz, abgerundet, wie das App-Icon. Vorher
  ein Pfeil im blauen Kreis.

## Extension 3.29.1 (App 0.37.2, 15.10.2026)

- Mehrere Stellen markieren geht jetzt mit ⌥ statt ⌘: ⌥ + Markieren nimmt
  Text dazu, ⌥-Klick ein Bild. ⌘-Klick gehört wieder ganz dem Browser und
  öffnet Links im neuen Tab, auch auf Bildern.

## App 0.37.2 (Extension 3.29.0)

- PDF, Plakat: zwei neue Anordnungen neben dem Raster. Index setzt die
  Bilder ungeschnitten in wechselnden Reihen, Paar zwei pro Reihe. Dazu
  eine Kopfzeile mit Titel, Anzahl, Datum und Seite und ein Feld
  „Beschreibung“ für die erste Seite.
- PDF-Ersteller: „PDF sichern“ steht unten links über die ganze Breite mit
  ⌘↩, „Abbrechen“ klein darunter.
- Tray: mehrere Objekte wählen mit ⌘-Klick, ⇧-Klick oder ⇧ + Pfeilen.
  Ziehen und ⌘C nehmen alle mit.
- Tray beim Herausziehen: Statt fast unsichtbar zu werden, klappt er nach
  oben auf eine Leiste zusammen. Sie zeigt, was du ziehst, der Rest des
  Bildschirms bleibt frei. Nach dem Ablegen klappt er wieder auf.
- Tray und Observer holen das Hauptfenster nicht mehr nach vorn, wenn
  Archiver ausgeblendet war.

## App 0.37.1 · Extension 3.29.0

- Lightbox: Die Leiste unten ist 960 statt 600 px breit, lange Dateinamen
  passen ganz hinein. Ist ein Name noch länger, wird er in der Mitte gekürzt,
  Anfang und Endung bleiben sichtbar.

## Extension 3.29.0 (App 0.37.0)

- Bibliothek beim Ziehen wechseln: Gibt es mehrere Bibliotheken, steht beim
  Ziehen eine Leiste mit ihren Namen über den Zonen. Namen anfahren, kurz
  verweilen, und die Zonen zeigen die Ordner dieser Bibliothek. Gilt für
  Bilder, Text und Instagram. ⇧ springt weiter zur nächsten.

## Extension 3.28.4 (App 0.37.0)

- Pinterest: Ziehen aus der Übersicht zieht jetzt das Bild statt des Pin-Links,
  und Pinterest öffnet dabei nicht mehr „Bild zur Suche hochladen“. Dateien aus
  dem Finder kann man weiter auf Pinterest ziehen.

## App 0.37.0 · Extension 3.28.3

- ⌘Z wirkt nur noch in der Bibliothek, in der der Schritt passiert ist. Nach
  einem Wechsel (⌘1–4, Palette) konnte ein altes „Verschieben“ sonst Dateien
  mit gleichem Pfad in der anderen Bibliothek treffen.
- Zone löschen: Dateien ohne Vorschau (PSD, AI, INDD, ZIP …) wandern sichtbar
  mit. Die Rückfrage nennt sie, „Zuletzt gelöscht“ zeigt sie als Karte und holt
  sie zurück. Bisher waren sie dort unsichtbar.
- Eine gewachsene Bibliothek öffnet sich auch über 20.000 Dateien weiter. Die
  Größenprüfung gilt nur noch für Ordner, die neu zur Bibliothek werden.
- Unsorted: Wer das Leeren nach 7 Tagen aus- und wieder einschaltet, bekommt
  neue Fristen. Vorher wanderte alles länger Liegende sofort nach „Zuletzt
  gelöscht“.
- Vorabversionen: eigener Update-Kanal nur für Lukas' Tests (⌥-Klick auf die
  Version unter ☰ › Information). Beta-Tester bekommen nur, was danach mit
  „freigeben“ unverändert an alle geht.
- Extension 3.28.3: „Board importieren“ auf Pinterest ist entfernt (keine
  Massen-Downloads).
- Beta-Anleitung nennt den Freischalt-Code und den richtigen Weg zum Feedback.

## App 0.36.4 · Extension 3.28.2

- Nichts wird mehr beim Sichern oder Importieren umbenannt. „Beim Ablegen
  umbenennen“ ist entfallen, auch für die Extension. Umbenannt wird nur, wenn du
  es im Umbenennen-Blatt bestätigst.
- „Umbenennen …“ und der Namensvorschlag gibt es jetzt auch in geteilten
  Libraries.
- Die Lightbox hat ein Rechtsklick-Menü: Umbenennen …, Öffnen, Quelle, Im Finder
  zeigen, Kopieren.

## App 0.36.3 · Extension 3.28.2

- Namensvorschlag inhaltlicher, alles lokal: Absender und Titel der Seite,
  Wörter aus Bild- und Seitenadresse, bis zu drei Wörter und die Jahreszahl aus
  dem Bild, bis zu drei Motive, Besonderes (Gruppe, Tier, Schwarzweiß,
  Screenshot) und bis zu zwei Farben, höchstens 80 Zeichen.
- Bildanalyse (Apple Vision) liest das Original statt der Vorschau, erkennt
  zusätzlich Tiere und Gruppen und liest Bildtext auch auf Französisch,
  Italienisch und Spanisch. Alle Bilder laufen einmal neu durch die Analyse (im
  Hintergrund).

## App 0.36.2 · Extension 3.28.2

- Namensvorschlag gründlicher: Absender und Titel der Seite (Instagram: Name
  und erste Wörter der Unterschrift), die größten Wörter im Bild, bis zu zwei
  Motive statt einem, Hauptfarbe. Zu Allgemeines (Struktur, Material, Text)
  fällt weg, höchstens 64 Zeichen. Neu analysierte Bilder liefern den Text mit
  den größten Zeilen zuerst.
- ⌘C mit mehreren Objekten legt auch die Dateien in die Zwischenablage, ⌘V fügt
  alle ein (vorher nur das erste Bild). Vorhandenes wird nicht neu kopiert.

## App 0.36.1 · Extension 3.28.2

- Geteilte Bibliotheken: Einstellungen › Bibliothek zeigt „Unsorted nach 7 Tagen
  gelöscht“ jetzt auch dort. Ab Werk aus; bei „An“ gelten Hinweise und
  Aufräumen wie in einer eigenen Bibliothek.

## App 0.36.0 · Extension 3.28.2

- Einstellungen › Dateinamen: ein Schema für einheitliche Namen. Fünf fertige
  Vorlagen (Datum_Zone_Nr, Zone_Nr, Datum_Quelle_Nr, Zone_Inhalt) und ein
  eigenes Schema aus Bausteinen mit Trenner, Datumsformat, Kleinschreibung und
  Präfix. Ab Werk bleibt jeder Name, wie er ist.
- „Beim Ablegen umbenennen“: neue Objekte bekommen das Schema gleich, auch
  über die Extension.
- Aufräumen: Auswahl › Rechtsklick › „Umbenennen …“ zeigt die Liste alt → neu.
  Uneinheitliche Namen (IMG_…, Bild (3), Screenshot …) sind markiert, einzelne
  Namen lassen sich ändern, Häkchen nehmen Objekte heraus.
- Namensvorschlag aus dem Inhalt (Seitentitel, erkanntes Motiv, Farbe) in der
  Info-Spalte. Er gilt erst, wenn du „Übernehmen“ drückst. Alles läuft lokal.
- Rechtsklick › „Ablegen in …“ der Extension sichert wieder (3.28.2).
- Unsorted leert sich nach 7 Tagen in jedem Ordner namens Unsorted.
- Vier statt drei Bibliotheken (⌘4), dazu „Neue Bibliothek…“ im Finder-Dialog.

## Vorabstand 0.35.1 (nicht veröffentlicht, in 0.36.0 aufgegangen)

- Extension: Rechtsklick › „Ablegen in …“ sichert wieder. Chrome beendet die
  Extension im Hintergrund nach kurzer Ruhe, danach fand der Klick sein Ziel
  nicht mehr. Das Ziel steht jetzt im Menüeintrag selbst.
- Unsorted leert sich nach 7 Tagen in jedem Ordner namens Unsorted, auch wenn
  er selbst angelegt, klein geschrieben, nummeriert (00_Unsorted) oder eine
  Unterzone ist. Bisher zählte nur die genaue Schreibweise „Unsorted“.
- Vier statt drei Bibliotheken, ⌘4 schaltet auf die vierte.
- Einstellungen › Bibliothek › „Neue Bibliothek…“: Ort und Name im Finder
  wählen, Archiver legt den Ordner an und öffnet ihn.

## Extension 3.28.1 (14.10.2026)

- Mehrfachmarkierung: Wer nach dem Markieren mit ⌘ die zuletzt markierte
  Stelle zieht, bekommt jetzt alle Stellen auf die Textkarte. Bisher hob der
  Griff auf die laufende Markierung die früheren Stücke auf.

## App 0.35.0 (14.10.2026)

- Abschnitte: Objekte lassen sich jetzt von einem Abschnitt in einen anderen
  ziehen, auch in einem Unterordner. Der Abschnitt der losen Objekte legt
  direkt in den offenen Ordner.
- Einstellungen › Ansicht › „Active Zones beim Ziehen“: wahlweise
  „Angeschnitten“ wie in der Extension. Die Karten ragen erst unten herein und
  fahren hoch, wenn der Zeiger hineinkommt. Ab Werk bleibt es „Ganz“.
- Unsorted ist immer die Zone, die nach 7 Tagen unter „Zuletzt gelöscht“
  wandert. Eine andere Zone lässt sich dafür nicht mehr wählen, nur an oder
  aus. Unsorted lässt sich nicht umbenennen.

## Extension 3.28.0 (14.10.2026)

- Beim Ziehen hängt das Bild oder Video im eigenen Format am Zeiger, eckig
  statt als abgerundetes Quadrat. Videos zeigen ihr Vorschaubild statt eines
  grauen Kastens.

## Extension 3.27.1 (14.10.2026)

- Instagram-Feed: Beim Ziehen eines Videos hängt sofort das Video am Zeiger,
  nicht mehr der Link „Ton stummgeschaltet“. Gesichert wird die MP4 mit
  Beitrag und Account als Quelle.
- Instagram: Beitragsdaten, die beim Laden der Seite schon da sind, gehen
  nicht mehr verloren, wenn die Extension etwas später startet.

## Extension 3.27.0 (14.10.2026)

- Mehrere Stellen markieren: Mit gedrückter ⌘-Taste weiteren Text markieren
  oder Bilder anklicken (nochmal anklicken nimmt sie raus). Ein Zug in eine
  Zone sichert alles: nur Text wird eine Textkarte, Text und Bild wird Bild
  mit Bildtext, nur Bilder werden einzeln gesichert. Klick daneben oder Esc
  hebt die Markierung auf. ⌘-Klick auf ein Bild in einem Link öffnet dabei
  keinen neuen Tab.
- Instagram-Feed: Videos lassen sich jetzt auch dort ziehen. Bisher lag ein
  großer Knopf („Ton stummschalten“) über dem Video und es entstand eine
  Textkarte mit diesem Text.

## Extension 3.26.0 (14.10.2026)

Instagram ohne eigene Anfragen an Instagram (zum Testen).

- Archiver fragt Instagrams interne Schnittstelle nicht mehr ab. Stattdessen
  liest die Extension passiv mit, was Instagram selbst an Beitragsdaten in den
  Browser lädt (Feed, Beitrag, Reels, Stories), und nimmt daraus Bild oder
  Video in Originalgröße, Videos als MP4 mit Ton.
- Fehlen die Daten für einen Beitrag, wird das angezeigte Bild in der größten
  Variante der Seite gesichert, bei Videos wie bisher ein Standbild.
- Kontoname für die Quelle notfalls aus dem Kopf des Beitrags.
- Nur Adressen von Instagrams Bildservern werden übernommen.

## App 0.34.2 (14.10.2026)

- Liste: Text- und Notizkarten stehen klein links wie die Bilder, nicht mehr
  über die ganze Breite. Im Namen steht der Anfang des Texts.
- Liste: Klick auf Name, Format, Maße, Größe, Quelle oder Gesichert sortiert
  danach, nochmal umgekehrt, ein drittes Mal wieder wie in der Zone.

## App 0.34.1 (14.10.2026)

- „(Stände)“ ist wieder raus: kein Bündeln, keine Stapel, keine Stände-Leiste
  und kein Vergleich in der Lightbox. Raster, Original und Lightbox sind wie
  in 0.33.2.
- Die Ansicht „(Liste)“ bleibt, jetzt auch in „Zuletzt gelöscht“ mit Namen
  und Angaben.

## App 0.34.0 (14.10.2026)

- Neue Ansicht „(Liste)“ neben Raster und Original: Zeilen mit Bild,
  Dateiname, Format, Maße, Größe, Quelle und Datum. Die Größenstufen setzen
  die Bildhöhe.
- Neuer Schalter „(Stände)“ in Raster und Liste: Versionen einer Datei werden
  am Dateinamen erkannt (v2, final, korr, entwurf, Datum, Kopie …) und
  gebündelt. Nur in der Ansicht, keine Datei wird umbenannt.
  - Raster: Stapel mit dem neuesten Stand oben, die Zahl klappt alle Stände
    alt → neu in einem Band auf.
  - Liste: neuester Stand vorne, „(n ältere)“ klappt die älteren darunter auf.
  - Lightbox: Leiste mit allen Ständen, „Vergleichen“ (V) legt zwei Stände
    übereinander, eine Linie zum Ziehen (oder ←/→).

## App 0.33.2 (14.10.2026)

- Unsorted steht auch in Bibliotheken mit eigener Reihenfolge vorn als erste
  Active Zone, wenn es neu dazukommt. Wer es danach verschiebt, behält seinen
  Platz.

## App 0.33.1 (14.10.2026)

- Unsorted: kurz „Nach 7 Tagen gelöscht“, in der Kopfzeile, im Index und
  jetzt auch auf der Zonenkarte in der Übersicht.
- Einstellungen › Bibliothek: wählbar, welche Zone sich nach 7 Tagen leert
  (ab Werk Unsorted, oder „Aus“).
- Unsorted entsteht mit jeder Bibliothek, auch in einem bestehenden
  Ordnersystem, und steht als erste Zone vorn.
- Bild mit Text: farbiger Punkt unten rechts statt Eselsohr.

## App 0.33.0 (14.10.2026)

Unsorted zeigt seine Frist, Bildtext ruhiger im Raster.

- Unsorted: Die Kopfzeile sagt nur noch, dass hier nach 7 Tagen aufgeräumt
  wird. Im Index steht „Leert sich nach 7 Tagen“ hinter dem Namen. Die letzten
  zwei Tage zeigt jedes Objekt oben links „noch n T.“ in Rot.
- Bild mit Text: kein Textauszug mehr unter dem Bild, stattdessen ein
  Eselsohr unten rechts in der Farbe der Textkarte. Der Chip „(+ Text)“
  entfällt.
- ⌘D: Unter „Bildtext“ lässt sich die Farbe der Textkarte wählen, wie bei
  Textkarten und Notizen.

## App 0.32.1 · Extension 3.25.3 (14.10.2026)

Leiste wie in der App, Ziehen überall.

- Zonenleiste am ganzen Mac ist ab Werk aus; einschalten unter
  Einstellungen › Tray. Auch bei bisherigen Installationen aus, bis man sie
  selbst wieder einschaltet.
- Extension 3.25.1: Die Seite scrollt beim Ziehen an den unteren Rand nicht
  mehr unter der Leiste weg, und Ablagen der Seite darunter greifen nicht mehr
  mit. Auch beim Ziehen von Text und auf Instagram fährt die Leiste jetzt aus
  dem Streifen hoch.
- Extension 3.25.2: Zonenkarten genau wie in der Leiste der App (gestrichelter
  Ring statt Outline, „Downloads“ in derselben Schrift). Streifen 90 statt
  30 px, in App und Extension: dieselbe Karte, die unten hereinragt. Die
  Leiste steht beim ersten Ziehen sofort (lädt nicht erst nach) und kommt
  früher auf langsamen Seiten. Nach einem Update zeigt ein noch offener Tab
  keine alte Leiste mehr.
- Extension 3.25.3: Ziehen klappt auch auf Seiten, die Maus- und
  Ziehen-Ereignisse selbst abfangen (Archiver hört jetzt als Erstes zu, am
  Fenster statt am Dokument). Instagram-Stories: gesichert wird die
  Originaldatei (Foto oder Video) aus den Mediendaten statt eines Standbilds.

## App 0.32.0 · Extension 3.25.0 (14.10.2026)

Ablegen, ohne dass etwas im Weg ist.

- Zonenleiste beim Ziehen, am Mac und in der Extension im Browser: Sie erscheint erst als 30 px schmaler Streifen am unteren Rand, Nummer und Name in einer Zeile. Darüber bleibt alles frei, etwa das Eingabefeld eines Chats. Fährt der Zeiger in den Streifen, kommt die volle Leiste; weiter oben wird sie wieder Streifen.
- ⌘V legt die Zwischenablage in die offene Zone oder ihren Ordner: Dateien aus dem Finder, kopierte Bilder und Screenshots, Text als Textkarte (benannt nach seinen ersten Wörtern). Auf der Startseite landet es in Unsorted. Im Tray mit ⌘V in der offenen Zone, ⌘C kopiert dort das gewählte Objekt.
- Unsorted räumt sich selbst auf: Was 7 Tage dort liegt, kommt unter „Zuletzt gelöscht“ und lässt sich von dort noch 7 Tage wiederherstellen. Die Frist zählt ab dem Tag, an dem es in Unsorted auftaucht, für alles Vorhandene also ab dem ersten Start mit 0.32. Geteilte Bibliotheken bleiben unberührt. Der Kopf von Unsorted sagt es dazu.
- Tray: Unterordner einer Zone stehen als Karten über den Objekten, ↵ oder Klick öffnet, ⌫ geht eine Ebene hoch. Der gestrichelte Ring der ersten Reihe wird oben nicht mehr abgeschnitten. Active Zones sind so hoch wie die Karten im Index.
- Neue Unterzeile: „A Visual Library for Creative Research“ im Startbild, „A visual library for creative research“ bzw. „Eine visuelle Bibliothek für kreative Recherche“ im Info-Text von App und Extension.

## App 0.31.2 (14.10.2026)

Menschen finden.

- Gesichter und Personen erkennt die App jetzt mit einer eigenen Erkennung. Bisher kamen sie nur aus der allgemeinen Bilderkennung, die Menschen auf Plakaten, in Grafiken und Ausschnitten oft gar nicht meldet. Ein großes Gesicht gilt zusätzlich als Porträt.
- „Frau“, „Mann“ und „Kind“ finden Bilder mit Personen. Ein Geschlecht erkennt macOS nicht, daher findet „Frau“ alle Personen. „Kopf“ und „Gesicht“ finden Gesichter.
- Alle Bilder laufen dafür einmal neu durch die Erkennung. Bis sie durch ist, steht in der Filterzeile „Bilderkennung: noch n von m“.

## App 0.31.1 (14.10.2026)

Genauere Suche. App und Tray suchen jetzt gleich.

- Gesucht wird nach ganzen Wörtern und ihren Formen: „Kreise“, „Kreisen“ und „Kreis“ finden dasselbe, „rote“ findet Rot. Umlaute gehen auch als ae/oe/ue, ß als ss.
- Keine Treffer mehr mitten im Wort: „rot“ findet weder „Brot“ noch „Protokoll“. Wortanfänge zählen nur beim Wort, das du gerade tippst.
- Wortteile nur in Namen, etwa „Entwurf“ in „Plakatentwurf“. Dateinamen werden an Großbuchstaben und Ziffern getrennt („KonzertPlakat2024“).
- Erkannte Motive zählen erst ab 50 % Sicherheit, in Suche und Motiv-Filter. Bisher reichten 30 %, daher viele falsche Treffer bei „Mensch“, „Pflanze“ und Ähnlichem. Bilder mit älterer Analyse werden dafür einmal im Hintergrund neu erkannt.
- Text im Bild zählt nur als ganzes Wort, Füllwörter wie „mit“ oder „und“ werden übersprungen.
- Treffer im Namen stehen vor Treffern im Bildinhalt, die ganze Suche im Namen ganz vorn. Der Tray sortiert jetzt auch nach Relevanz.

## App 0.31.0 · Extension 3.24.0 (13.10.2026)

Stabilitäts-Release nach dem Prüfbericht zu 0.30.1. Keine neuen Funktionen, dafür weniger Datenverlust, weniger stille Fehler und mehr Tempo bei großen Bibliotheken.

Sicherheit:
- Die Extension reagiert nur noch auf echte Eingaben. Eine Webseite kann kein Overlay öffnen, keine Zonenleiste einblenden und nichts in deine Bibliothek sichern. Das Overlay ist für die Seite unsichtbar (geschlossenes Shadow DOM).
- Die Brücke übernimmt nur Dateien aus Downloads/Archiver-Transfer, nie einen beliebigen Pfad auf dem Mac. Zonennamen mit „..“ werden abgelehnt.
- Rechte wie Zonen anlegen oder Galerie lesen hat nur noch die Seite selbst, nicht eingebettete Frames.

Externe Zonen:
- Ein abgezogenes Laufwerk oder ein umbenannter Projektordner zeigt die Zone als „offline“, statt ihre Objekte samt Bildtexten aus der Bibliothek zu löschen. Dasselbe gilt für Ordner, die gerade nicht lesbar sind.
- Bilder aus externen Zonen erscheinen wieder in App und Observer.
- Systemordner, ganze Laufwerke und ineinanderliegende Zonen werden abgelehnt, mit Hinweis.

Dateien:
- Verschieben auf ein anderes Laufwerk kopiert erst vollständig und löscht dann das Original. Bricht es ab, bleibt das Original unangetastet.
- metadata.json, zones.json und die Einstellungen werden atomar geschrieben, ein Absturz hinterlässt keine halbe Datei.
- Zitat-Text, Löschen mehrerer Objekte und Umbenennen einer Zone nur in Groß/Kleinschreibung funktionieren zuverlässig. Eigene Reihenfolge, Standard-Zone und Hot Zones ziehen beim Umbenennen mit.
- „Zuletzt gelöscht“ greift nur noch auf Dateien im eigenen Papierkorb zu.
- Jede Bibliothek merkt sich ihre eigene Zonen-Reihenfolge. Bisher übernahm eine Bibliothek beim Wechsel die Reihenfolge der vorigen.

Fehler sichtbar:
- Protokoll unter ~/Library/Logs/Archiver/archiver.log.
- Fehler beim Einlesen erscheinen als Hinweis, statt still zu verschwinden.
- Die Zonenleiste am Mac sagt „Gesichert“ erst, wenn es gesichert ist, sonst „Nicht gesichert“ mit Grund.
- Notizen zeigen „Nicht gesichert“ und versuchen es erneut.
- Eine beschädigte Datenbank wird beiseitegelegt und neu aufgebaut. Kaputte Einstellungen werden als settings.json.broken aufgehoben.
- Ein Bild, an dem die Vorschau abstürzt, legt den Vorschau-Dienst nicht mehr lahm.

Tempo:
- Die Extension fragt Zonen und Galerie seltener ab, die App liest dafür nicht mehr jedes Mal den ganzen Ordner ein. Galerie-Vorschauen laden erst, wenn sie sichtbar werden.
- Die Startseite rechnet „zuletzt gesichert“ einmal statt pro Zeile. Lange PDFs geben Speicher frei. Verschieben und Löschen blockieren die Oberfläche nicht mehr.

Extension 3.24.0:
- Antwortet die App nicht, obwohl sie schon verbunden war, sagt die Extension, dass das Bild in Downloads liegt, statt es still dort abzulegen.
- Nicht übernommene Übergaben werden aus Archiver-Transfer geräumt.
- Nach einem Extension-Update sagt die Seite „Bitte neu laden“, statt nichts zu tun.
- Das Rechtsklickmenü zeigt auch leere Ordner.

Veröffentlichen:
- Das Auto-Update an alle Tester geht nur noch raus, wenn beim Start des Workflows „publish“ angehakt ist. Alle Actions sind auf feste Commits gepinnt.

## App 0.30.1 · Extension 3.23.0 (13.10.2026)

Bildtext in der Lightbox:
- Der Bildtext steht auf einer farbigen Karte wie die Textkarten. Einstellungen › Aussehen wählt die Form: Rückseite (Standard, das Bild wird mit T, „Text“ oder dem Eselsohr unten rechts gewendet) oder Doppelseite (Karte links neben dem Bild).
- Bildtext und Textkarten lassen sich in der Lightbox direkt editieren.
- Der Hintergrund der Lightbox verschwimmt, kein Text aus dem Raster scheint mehr durch.

Sichern:
- Ein Bild, das schon in der Bibliothek liegt, wandert beim erneuten Sichern in die gewählte Zone und übernimmt den markierten Text. Bisher blieb es still in seiner alten Zone.
- Die Adresse der offenen Seite in eine Zone gezogen: Linkkarte, dahinter die ganze Seite offline (MHTML). „Gesichert“ in der Lightbox öffnet sie in Chrome.

Externe Zonen:
- Einstellungen › Bibliothek › „Ordner außerhalb der Bibliothek“: ein Projektordner wird zur Zone, die Dateien bleiben, wo sie sind. ✕ löst nur die Zone, nie den Ordner.

## App 0.30.0 · Extension 3.22.0 (13.10.2026)

Handbuch:
- ≡ › Handbuch zeigt alle Funktionen von App, Extension, Observer und Tray samt Tastenkürzeln, auf Deutsch und Englisch. Dasselbe steht unter archiver-app.com/manual.

Extension-Hinweis:
- Ist die Extension veraltet, steht oben rechts „Extension 3.22.0 ist da (Laden)“. Hat sich nach zwei Tagen noch keine gemeldet, steht dort „Keine Extension verbunden“.
- Einstellungen › Extension zeigt die verbundene Version. Der erste Start wartet, bis sich die Extension wirklich meldet.

Verdichtet:
- Die Navigation steht fest oben links. Ordner in einer Zone erscheinen nur noch als Karten, ⌘R lädt sofort neu.
- Der Startton ist standardmäßig aus.
- Video und Audio beim Öffnen abspielen hat einen gemeinsamen Schalter.
- „Unterordner als Zonen“ entfällt, Unterordner erscheinen als Ordner in der Zone.
- Das Feld „Notiz“ in den Details ist im Bildtext aufgegangen. Vorhandene Notizen werden an den Bildtext angehängt. Suche, PDF („Mit Bildtext“) und Paket lesen den Bildtext.
- Textkarten und Notizen haben eine gemeinsame Farbreihe: Rot, Grün, Violett, Cyan, Gelb, Weiß und Schwarz.

Aussehen:
- Regler „Glas“ für die Fenster von App, Tray und Observer. Der Observer ist jetzt ebenfalls ein Glasfenster.
- Regler „Zonen beim Ziehen“ für die Karten in App, Zonenleiste und Extension. Alle Regler laufen in 5-%-Schritten.
- Die Ecken aus Einstellungen › Aussehen gelten auch in der Zonenleiste und in der Extension.

Extension 3.22.0:
- Meldet der App ihre Version.
- Hell mit den Werten der App, Ecken und Zonen-Deckkraft wie in der App.
- Enthält 3.21.1: Bilder mit Kommas in der Adresse (Substack, Cloudinary) laden in voller Größe.

## Extension 3.21.1 (13.10.2026, lokal, erscheint mit der nächsten App)

- Bilder mit Kommas in der Adresse (Substack, Cloudinary) laden wieder in voller Größe. Bisher brach die Bildadresse am Komma ab und der Download schlug fehl, etwa beim Sichern von Bild und markiertem Text.

## App 0.29.1 · Extension 3.21.0 (13.10.2026)

Zuletzt gelöscht:
- Doppelklick, Leertaste oder Enter öffnen die Lightbox. Dort gibt es „Wiederherstellen“ und „Sofort löschen“.
- ⌫ fragt nach, bevor etwas endgültig in den Papierkorb des Macs geht. Das gilt auch für den Chip und das Menü.

Notizen:
- „+“ bei den Verknüpfungen zeigt zuerst alle Zonen als kleine Karten wie im Tray. Klick öffnet die Zone, „‹ Zonen“ führt zurück.

Extension:

- Bild und Text zusammen markiert und gesichert (Ziehen oder Rechtsklick): Gesichert wird das Bild, der markierte Text wird sein Bildtext. Bisher entstand nur eine Textkarte.

## App 0.29.0 (13.10.2026)

Bildtext:
- Text, den man beim Sichern auf der Seite markiert, wird zum Bildtext des Bildes (Extension).
- In der Info gibt es das Feld „Bildtext“. ⌘V mit Text auf ein gewähltes Bild setzt ihn ebenfalls.
- In der Ansicht „Original“ zeigt „(+ Text)“ Kennung, Bildtext und Notiz unter den Bildern.
- In der Lightbox steht der Bildtext links neben dem Bild, unten bündig. „Text“ oder T blendet ihn aus.

Notizen:
- Neue Notiz per Rechtsklick in eine Zone oder ⌘⇧N. Sie liegt als .md-Datei in der Zone.
- Im Raster ein Blatt mit Titel und Text, in der Lightbox schreibt man direkt darauf. Gesichert wird von selbst, der Titel wird beim Verlassen zum Dateinamen.
- Farbe über die Punkte in der Leiste: Papier, Gelb, Rot, Cyan, Schwarz.
- Links neben dem Blatt stehen die verknüpften Objekte. „+“ sucht in der ganzen Bibliothek, Klick öffnet das Objekt.
- Finden hat den Filter „(Notizen)“. .md-Dateien, die schon in Zonen liegen, erscheinen jetzt ebenfalls als Notiz.

## App 0.28.0 (13.10.2026)

Tray (bisher „Palette“):
- Heißt jetzt Tray. Unten links steht „Tray“, die Tasten stehen rechts.
- Hell/Dunkel und die Ecken aus Einstellungen › Aussehen gelten auch im Tray, samt Fensterecke.
- Öffnen holt das Archiver-Fenster nicht mehr nach vorne, auch wenn es nur hinten lag. Nach dem Schließen liegt es wieder hinten.

Aussehen:
- Glas und Grundfarbe des Trays sind jetzt der Standard fürs ganze Fenster (Dunkel 72 %, Hell 78 %). Der Regler in Aussehen startet auf diesem Wert.
- Die Leiste in der Lightbox hat die Maße der Navigation oben.
- Im schmalen Fenster stehen die Active Zones 2 × 2, wie der Index.

Zonen und Ordner:
- Zuletzt gelöscht zeigt dasselbe Raster wie jede Zone, mit Größenstufen. Zone und Resttage stehen im Badge.
- Unterordner haben ⋯ und Rechtsklick: Umbenennen, Im Finder zeigen, Löschen.
- Ordnerkarten lassen sich auf andere Ordnerkarten ziehen und liegen dann darin.
- Lightbox: Klick neben das Objekt schließt sie.

Einstellungen:
- Tray, Observer und Extension haben je einen eigenen Bereich.

## App 0.27.0 (12.10.2026)

Entfernt:
- Getting Started (Tour) und das Farbschema Salbei. Wer Salbei hatte, sieht wieder das Standard-Schema.

Zonen und Ordner:
- Zuletzt gelöscht: Gelöschte Objekte und Zonen liegen 7 Tage in der Bibliothek und lassen sich zurückholen (Doppelklick, Rechtsklick oder „Wiederherstellen“). Danach gehen sie in den Papierkorb des Macs. Zu finden als letzte Zeile im Index und im ≡-Menü.
- Der Titel einer Zone steht oben fest und springt nicht mehr, wenn man in einen Unterordner geht.
- Die Ordner-Chips links oben filtern nur noch, Titel und Ordnerkarten bleiben stehen. Die Karten rechts führen weiter in den Ordner.
- Rechtsklick in eine Zone: „Neuer Unterordner“, auf Objekte: „Neuer Unterordner mit Auswahl“. Der Ordner heißt erst „Neuer Ordner“ und lässt sich gleich umbenennen. Leere Ordner haben jetzt auch eine Karte.
- Objekte lassen sich auf die kleinen Ordnerkarten ziehen und landen im Unterordner. ⌘Z holt sie genau zurück.
- Ganze Ordner aus dem Finder in die App ziehen: in der Übersicht wird daraus eine Zone, in einer Zone oder auf einer Karte ein Unterordner, jeweils mit allen Unterordnern.
- Videos zeigen beim Hover eine kurze Vorschau aus acht Bildern.
- Fehler behoben: In der Übersicht „Karten“ lässt sich eine Zone aus dem Index wieder auf eine Active Zone ziehen (und umgekehrt).

Palette:
- Textkarten erscheinen in ihrer Farbe statt als leere Fläche.
- ⌘1–3 wechselt die Bibliothek, wie in der App.
- Leertaste zeigt das gewählte Objekt groß, innerhalb der Palette. ←→ blättern, Leertaste oder esc schließen.

## App 0.26.1 (12.10.2026)

- Palette aus InDesign: Rahmen markieren, ⌥ Space, Doppelklick oder ↵ auf ein Bild – es landet verknüpft im Rahmen, proportional gefüllt und zentriert. Ohne markierten Rahmen liegt es im Platzieren-Cursor. Beim ersten Mal fragt macOS, ob Archiver InDesign steuern darf.
- esc schließt die Palette und holt das Programm von vorher zurück, nicht mehr das Archiver-Fenster.
- Bild aus dem Finder öffnen: nur der Observer erscheint, das Archiver-Fenster bleibt zu bzw. hinten.

## App 0.26.0 (12.10.2026)

Archiver am ganzen Mac, auch wenn das Fenster zu ist:
- Palette per ⌥ Space in jedem Programm: Active Zones als Karten wie in der App, Suche über Zonen und Objekte, Zone mit 1–4 wechseln. Objekte zieht man direkt heraus, etwa in einen InDesign-Rahmen; dabei wird die Palette fast unsichtbar und lässt Klicks durch. ↵ kopiert, ⌘↵ zeigt im Finder, esc schließt.
- Zonenleiste beim Ziehen: Wer irgendwo Dateien zieht (Finder, Schreibtisch, Mail …), sieht unten die Active Zones 01–04. Loslassen sichert die Dateien in der Zone.
- Einstellungen › Extension & Observer › Am ganzen Mac: Kürzel wählen (⌥ Space, ⌃⌥ Space, ⇧⌥ Space oder aus) und die Leiste an- oder ausschalten.
- Das Fenster schließen beendet Archiver nicht mehr, es läuft für Palette und Leiste weiter. ⌘Q beendet.

## App 0.25.0 (12.10.2026)

- Neues Hell: neutrales Hellgrau (#F4F4F4) ohne Farbstich, alle Grautöne ohne Sättigung. Weiße Bilder behalten ihre Kante.
- Übersetzt aus Dunkel: Schrift mit denselben Kontrasten, Karten eine Stufe vom Grund abgesetzt, Bibliotheks-Reiter zurückgenommen.
- Navigation als weißes Glas mit leichtem Schatten.
- Zonenkarten beim Ziehen als helles Glas ohne Sättigung; wie stark sie decken, regelt weiter Einstellungen › Aussehen.
- Salbei bleibt unverändert.

## App 0.24.9 · Extension 3.20.0 (12.10.2026)

Extension auf dem Stand der App:
- Ziehen: Die Zonenleiste sieht aus wie in der App (Glas, Nummer 01–04, Name und Zahl, Geist-Schrift) und erscheint ohne Verzögerung.
- Shift beim Ziehen wechselt die Bibliothek. Die Bibliotheksleiste ist nur sichtbar, solange Shift gehalten wird; abgelegt wird in die gewählte Bibliothek. Mit nur einer Bibliothek legt Shift wie bisher in Downloads ab.
- Rechtsklick-Menü: oben die drei zuletzt genutzten Zonen, darunter alle Zonen mit ihren Ordnern und Unterordnern.
- Übersicht und Zone im Layout der App: Kopfzeile „Active Zones“, Nummern an den Karten, „n Objekte · Datum“, größeres Raster, Bibliothek oben rechts.

App:
- Die Brücke zur Extension liefert die Bibliotheken und die Ordner jeder Zone und nimmt Objekte für eine andere eingetragene Bibliothek an.

## App 0.24.8 (12.10.2026)

- Eine Fokus-Sprache: Auswahl, Tastaturfokus, Tausch- und Umsortier-Ziel sowie das Ablageziel in der Zonenleiste zeigen denselben gestrichelten Ring. Er folgt der Eckenrundung, „Umlaufend“ gilt überall.
- Objekte umsortieren: Statt einer Linie über oder unter dem Objekt trägt das Ziel den Ring, die gezogenen Objekte nehmen seinen Platz ein.
- Ordner ebenenweise: Im Ordner stehen nur seine eigenen Unterordner, der Pfad „Zone / Ordner / Unterordner“ führt auf jede Ebene. Esc und ⌘↑ gehen eine Ebene hoch.
- ⌘R lädt die Objekte neu. Einstellungen › Aussehen › Neu laden: Sofort oder Welle (diagonal, schnell).
- Einstellungen › Aussehen › Fensterhintergrund: wie stark die Farbe die macOS-Unschärfe deckt, getrennt von den Zonen.
- Übersicht (Karten-Cover, Index-Thumbs, Vorschau) folgt der eigenen Reihenfolge einer Zone.
- Noch nicht geladene Bilder erscheinen als Glasfläche, beim Laden blendet das Glas weich aus.
- Geteilte Bibliothek: Punkt „neu von anderen“ an Objekten und Zonen bis zum ersten Ansehen. In „Alle Objekte“ Filter „Neu von anderen“ und „Von“ (z. B. JP).
- iCloud-Konfliktkopien mit gleichem Inhalt („Bild 2.jpg“) werden erkannt; „Doppel zusammenführen“ legt sie in den Papierkorb (rückgängig machbar).

## App 0.24.7 (12.10.2026)

- Zonenleiste beim Ziehen: Das Glas ist sofort da und blendet mit ein, statt erst nach dem Einblenden scharf zu werden. Die Herkunftszone bleibt Glas und nimmt nur die Schrift zurück.

## App 0.24.6 (12.10.2026)

- Einstellungen neu geordnet: Bibliothek, Ansicht, Aussehen, Start (Öffnung und Startton), Studio, Extension & Observer, Allgemein. Studio enthält nur noch Projekt- und Team-Werkzeuge (Name, geteilte Bibliothek, Notiz, Rechte, Zone als Paket). Moodboard und eigene Reihenfolge sind immer an.
- Einstellungen › Bibliothek › Zonen: Zonen lassen sich wieder per Ziehen sortieren, die anderen rücken nach.
- Active Zones stehen in jeder Übersicht oben: in Karten als Karten, in Vorschau und Index als eigene Zeilen.
- Tausch mit einer Active Zone sichtbar: Beim Ziehen rückt das Ziel an den frei werdenden Platz, beim Loslassen gleiten beide an ihre neuen Plätze.
- Im Unterordner steht „Zone / Ordner“ in der Kopfzeile, ein Klick auf die Zone führt zurück.
- Ordnerkarten richten sich von rechts aus, ohne Lücke am Rand.
- Geteilte Bibliothek: Objekte anderer zeigen im Hover-Schild das Kürzel, z. B. „JP · pinterest.com“.

## App 0.24.5 (12.10.2026)

- Ordner sitzen rechts neben dem Zonentitel und richten sich am Bildraster aus: als Karten über drei Sechstel oder als Liste über zwei Sechstel (Vorschau, Name, Anzahl rechtsbündig). Einstellungen › Darstellung › Ordner in einer Zone: Karten (Standard) oder Liste. In der Liste wandern ↑↓ durch die Einträge.
- Aktiv-Zustände einheitlich: Karten und Listeneinträge zeigen eine Outline, Filter wechseln nur zwischen hell und dunkel.
- Einstellungen › Darstellung › Auswahl-Outline: Ruhig (Standard) oder Umlaufend. Umlaufend wandert die gestrichelte Linie langsam um ausgewählte Objekte und aktive Ordner.
- Kopfzeile: kein doppelter Punkt mehr nach abgekürztem Monat („10. Okt.“).

## App 0.24.4 (11.10.2026)

- Ordnerkarten oben in der Zone: ein Sechstel breit, Bild im Verhältnis 2:1 wie die Zonenkarten, mehr Luft darüber. ↑ aus der obersten Bildreihe springt auf die Karten, ←→ wechselt, Enter filtert, ↓ oder Esc führt zurück.
- Beim Scrollen läuft der Inhalt unter der Navigation weich aus, ohne Kante. Einstellungen › Darstellung › Oberkante beim Scrollen: Verlauf (Standard) oder Glas.

## App 0.24.3 (11.10.2026)

- Navigation und Bibliotheks-Reiter beginnen auf der Oberkante der Fensterknöpfe. Der Abstand vom grünen Knopf zur Leiste entspricht dem Abstand vom Fensterrand zum roten Knopf.
- Zonen mit Unterordnern zeigen oben die Ordner als Karten (Vorschau, Name, Anzahl). Ein Klick filtert auf den Ordner, ein zweiter hebt den Filter auf.
- PDF-Ersteller: Jede Seite passt in der Vorschau ganz ins Fenster und sitzt mittig, für Kontaktbogen, Plakat und Moodboard, quer und hoch.

## App 0.24.2 (11.10.2026)

- Navigation etwas kompakter (Knöpfe 36 px, Rand 5 px, Innenabstand 14 px). Fensterknöpfe, Leiste und Bibliotheks-Reiter liegen auf einer Mittellinie.
- ≡-Menü hängt an der ganzen Leiste: bündig mit ihrer Kante, mindestens so breit wie sie.
- Rechtsklick auf ein Objekt: „Ablegen in“ als eine Spalte untereinander.
- Rechtsklick auf eine Zone in der Übersicht (Karte oder Index-Zeile): Öffnen, Zur Active Zone machen, Als PDF ausgeben, dazu die Einträge aus dem ⋯-Menü.

## App 0.24.1 (11.10.2026)

- Navigation wieder in der großen Fassung wie die Extension-Leiste: Knöpfe 40 px hoch, 16 px Innenabstand, 6 px Rand, Schrift 13,5 px, Menü-Zeichen 20 px. Titelzeile und Fensterknöpfe entsprechend tiefer, Bibliotheks-Reiter gleich hoch.
- Reiter „Objekte“ heißt „Alle Objekte“ (EN „All objects“).

## App 0.24.0 · Extension 3.19.0 (11.10.2026)

Zonen und Ordner:
- Eine Karte pro Zone: Jeder Ordner der obersten Ebene ist eine Zone, seine Unterordner erscheinen darin als Ordner. Unterordner als eigene Zonen gibt es nur noch, wenn sie unter Einstellungen › Bibliothek › Ordnerstruktur eingeschaltet sind (Ebene 2 oder 3).
- In einer Zone mit Ordnern: Chips unter dem Titel filtern nach Ordner (Alle, je Ordner, Ohne Ordner). Umschalter Mosaik (ein Raster) oder Abschnitte (je Ordner eine Kopfzeile), pro Zone gemerkt.
- Karten-Ansicht als Raster: alle Karten einer Reihe gleich hoch, auch „+ Neue Zone“ und Zonen ohne Bild. Karten zeigen die Zahl der Ordner.
- Index-Ansicht: Die Zeile „+ Neue Zone“ ist so hoch wie die übrigen Zeilen.
- Die Standard-Zone heißt jetzt „Unsorted“. Eine bestehende Inbox wird beim Öffnen umbenannt, wenn sie die Standard-Zone ist (nicht in geteilten Bibliotheken).

Oberfläche:
- Oben rechts steht die aktive Bibliothek als zurückgenommener Reiter; ein Klick zeigt die anderen und „Bibliotheken verwalten“.
- Info › App: Lange Bibliothekspfade (z. B. Google Drive) bleiben einzeilig und gekürzt, der volle Pfad steht im Tooltip. Gleiches in Einstellungen › Bibliotheken.
- Freischalt-Code direkt im Sperr-Hinweis eingeben.
- Einstellungen › Darstellung: Video und Audio beim Öffnen abspielen (je An/Aus, Standard Aus), in Lightbox und Betrachter.

Ziehen:
- Große WebP-Bilder werden nicht mehr riesig gezogen: In der App kommt das Ziehbild notfalls über Quick Look, in der Extension wird es auf 220 px verkleinert.
- ⌘Z nach dem Verschieben legt Objekte wieder in ihren genauen Unterordner.

## App 0.23.0 (11.10.2026)

Bestehende Ordnersysteme als Bibliothek:
- Zonen nur bis zur zweiten Ebene (einstellbar 1–3 unter Einstellungen › Bibliothek › Ordnerstruktur). Tiefere Unterordner gehören zu ihrer Zone, die Details zeigen den Unterordner.
- Ordner ohne Bilder, PDFs, Video, Audio oder Text erscheinen nicht als Zone; ganz leere Ordner (neue Zonen) schon.
- Übersprungen werden Links, Document fonts, Backup, Ableton Project Info, Presets, Samples/Imported sowie Programm- und Mediathek-Pakete. Die Liste ist anpassbar.
- In einem bestehenden Ordnersystem legt Archiver keine leere Inbox an.
- Ordner in Google Drive, Dropbox, OneDrive und iCloud Drive lassen sich wieder als Bibliothek wählen (wurden fälschlich als Systemordner abgelehnt).
- Karten-Ansicht: lange Pfade werden gekürzt (voller Name beim Überfahren), das Datum bleibt in einer Zeile.

## App 0.22.0 · Extension 3.18.0 (11.10.2026)

- ⌘Z macht Verschieben, Löschen (holt die Dateien aus dem Papierkorb zurück) und Umbenennen von Zonen rückgängig, mehrfach hintereinander.
- Standard-Zone: In Einstellungen › Bibliothek wählbar, wohin alles ohne Ziel kommt (Extension ohne Zone, lose Bilder beim Import). Ersetzt die Inbox; die Inbox lässt sich dann wie jede Zone löschen.
- Textkarte › Quelle: Die Webseite öffnet sich an der zitierten Stelle, die Stelle ist markiert.
- Öffnungsanimation ohne das blaue Licht am Anfang.
- Kartenfarben wieder so kräftig wie in 0.20.0.
- Neuer Freischalt-Code; der bisherige gilt weiter.

## App 0.21.0 · Extension 3.17.0 (11.10.2026)

- Drei Bibliotheken: In Einstellungen › Bibliothek gibt es drei Plätze mit Name und Ordner. Umschalten mit ⌘1, ⌘2, ⌘3 oder im Menü. Zonen und Hot Zones gehören jeweils zur Bibliothek, die Extension sammelt in die aktive.
- Startschutz: Das Fenster erscheint sofort, die Bibliothek lädt im Hintergrund. Benutzerordner, Laufwerke und Systemordner werden als Bibliothek abgelehnt, ebenso Ordner mit mehr als 20.000 Dateien oder solche, die sich nicht innerhalb von 10 Sekunden lesen lassen. Archiver zeigt dann einen Hinweis statt zu hängen.
- Textkarten liegen als Textdatei (.txt) im Zonenordner und sind im Finder lesbar. Archiver zeigt sie weiter als farbige Karte. Auch im Finder angelegte .txt-Dateien erscheinen als Textkarte.
- Kartenfarben wieder kräftiger.
- Extension: Beim Ziehen von markiertem Text hängt die spätere Textkarte am Zeiger, in ihrer Farbe.
- Zonen ruhiger: kein blaues Glühen beim Ablegen, kein Hochfahren und Anheben, die Leiste blendet nur ein und aus.

## App 0.20.1 · Extension 3.16.5 (11.10.2026)

- Textkarten: Die Farbe wird zu Beginn jeder Markierung im Web zufällig bestimmt, die Markierung zeigt sie sofort (Hintergrund und Schrift wie in der App). Kein Abdunkeln, kein Rahmen mehr. Die gesicherte Karte hat dieselbe Farbe.
- In den Details (Info) lässt sich die Farbe einer Textkarte umstellen: Rot, Grün, Violett, Cyan, Gelb oder Weiß.
- Farben angeglichen und ruhiger (das Rot war zu grell), Schrift im Farbton dunkler.
- Textkarten in der Lightbox größer.
- Zonen-Karten, Mosaik und Index zeigen Textkarten farbig statt weiß.

## App 0.20.0 (11.10.2026)

Neu: Einstellungen › Studio, jede Funktion einzeln abschaltbar.
- Notiz: Feld „Warum diese Referenz?“ im Infopanel, durchsuchbar, im PDF über „Mit Notiz“.
- Moodboard: dritter Stil im PDF-Ersteller. Frei verschieben, über die Ecke skalieren, Doppelklick holt nach vorn, „Anordnen“ setzt neu. Hintergrund hell oder dunkel, Anordnung je Auswahl gemerkt.
- Eigene Reihenfolge: Objekte einer Zone per Ziehen umsortieren, Einfügelinie gestrichelt. Zurücksetzen über ⋯ › „Reihenfolge zurücksetzen“.
- Rechte: Urheber, Lizenz, „nur intern“ (roter Punkt auf der Karte).
- Zone als Paket: ⋯ › „Als Paket exportieren …“, auch für eine Auswahl. Ordner mit Dateien, Quellen.csv, Quellen.md und Kontaktbogen.pdf.
- Geteilte Bibliothek (Server/NAS/Dropbox): Datenbank lokal pro Mac, eine Datei pro Person in .archiver/people/, die neuere Fassung gewinnt. „Abgelegt von“ an neuen Objekten.
- Text- und Zitatkarten in naiven Grundfarben (Rot, Grün, Violett, Cyan, Gelb), Schrift Ton in Ton.

## App 0.19.4 (11.10.2026)

- Lightbox „Ablegen in“: Passen nicht alle Zonen in eine Zeile, verteilen sie sich auf gleich lange Zeilen statt eines einzelnen Rests.
- Bilder lassen sich aus der Lightbox wieder ziehen: die Zonen klappen auf, Loslassen auf einer Zone legt das Bild dorthin (auch in den Finder).

## App 0.19.3 (11.10.2026)

- Öffnung nach der Tour ohne die helle Querlinie: nur das Licht von den Seiten, dann öffnet sich die Oberfläche. Der Startton setzt mit dem Öffnen ein.
- Startton beim App-Start etwas später, im Takt mit der einfahrenden Wortmarke.

## App 0.19.2 · Extension 3.16.3 (11.10.2026)

- Getting Started läuft im normalen Fenster, kein erzwungenes Vollbild mehr.
- Im Kapitel zu den Active Zones bleibt die Leiste unten stehen, bis man weitergeht.
- Neue Öffnung nach der Tour: Im Dunkel drückt blaues Licht von den Seiten herein, eine helle Linie teilt die Fläche quer, dann öffnet sich die Oberfläche (Welle oder Lid).
- Extension: Bild mit gedrückter Shift-Taste auf eine Zone ziehen legt es schlicht in den Download-Ordner, am Archiv vorbei. Die Zonen zeigen dabei „In Downloads sichern“.
- Active Zones beim Darüberziehen: bleiben gläsern und hell wie im Ruhezustand, nur feiner Rand und leicht angehoben, statt dunkelblau (App und Extension).

## App 0.19.1 · Extension 3.16.2 (10.10.2026)

- Getting Started: Die Demo-Bibliothek füllt sich mit Kopien aus der eigenen Bibliothek (vier Active Zones, Inbox, Bilder für das Browser-Schema). Mitgelieferte Grafiken nur, wenn die eigene Bibliothek noch zu leer ist. Suchvorschläge kommen aus der Bildanalyse.
- Öffnung nach der Tour ohne Lichtring und ohne Schriftzug: das Dunkel weicht weich von der Mitte, dann glänzt die Oberfläche blau.
- Gestrichelte Linien sind zurück (App, Observer, Extension).
- Updates werden auch im laufenden Betrieb gesucht: alle 30 Minuten und beim Zurückkehren ins Fenster.

## App 0.19.0 (10.10.2026)

- Getting Started neu: neun Kapitel, fünf zum Mitmachen, im Vollbild. Auf Wunsch in einer Demo-Bibliothek (~/Archiver Demo/, 28 eigene Grafiken in fünf Zonen), getrennt von der eigenen Bibliothek. Am Ende wandert Selbstgesammeltes in die Inbox, die Demo in den Papierkorb.
- Sammeln im Browser als Schema in der Tour: die Geste spielt dreimal von selbst, dann zieht man selbst.
- Nach der Tour verlässt die App das Vollbild, Licht öffnet die Oberfläche (WebGL, Blau, synchron zum Startton). Form Welle oder Lid unter Einstellungen › Darstellung › Öffnung, dort auch zum Abspielen.
- „Archiver“ in der Navigation: kein Hover mehr, auf der Startseite aktiv wie die anderen Einträge.
- Neue Einstellung Navigation: oben oder unten, jeweils links, Mitte oder rechts. Unten weicht sie beim Ziehen den Active Zones aus.

## App 0.18.2 (10.10.2026)

- PDF-Ersteller: neuer Stil „Plakat“. Bilder randlos im Raster, angeschnitten, Name und Quelle klein in Schreibmaschinenschrift im Bild (hell oder dunkel je nach Untergrund). Dazu 16 pro Seite.

## App 0.18.1 · Extension 3.16.1 (10.10.2026)

- Der Betrachter heißt jetzt Observer.
- Neues App-Icon ohne gestrichelten Rahmen, als runde Mac-Kachel.
- Gestrichelte Linien überall durch feine durchgehende ersetzt (App, Observer, Extension).

## App 0.18.0 · Extension 3.16.0 (10.10.2026)

- Freischalt-Code: Archiver funktioniert erst nach Eingabe des Codes, unten in den Einstellungen.
- Einstellungen aufgeräumt: Bibliothek, Darstellung, Extension & Betrachter, Startton und Allgemein als Untermenüs.
- Zonen beim Ziehen: Deckkraft einstellbar (Darstellung), gilt in App und Extension.
- Neues Farbschema „Salbei“ als Option: Periwinkle auf Grau-Grün. Standard bleibt wie bisher.

## App 0.17.6 (10.10.2026)

- Zonen-Menü (⋯) auf Karten wird nicht mehr abgeschnitten, es schwebt über der Seite.
- Einheitlich „Objekte“ statt „Bilder“ bei Zählern, leeren Zonen und Meldungen.
- Einstellungen öffnen verschiebt die Navigation nicht mehr unter die Fensterknöpfe.
- Mehrfachauswahl wie im Finder: Shift + Pfeiltasten erweitert die Auswahl, Shift-Klick wählt einen Bereich auch über Zonen hinweg.
- Übersicht: ⌫ oder Entf löscht die gewählte bzw. überfahrene Zone, nach kurzer Rückfrage.

## App 0.17.5 (10.10.2026)

- Objekte: Filter nach Art oben in der Filterleiste: Bilder, Tracks, Video, PDF.
- Player: minimale Waveform statt Linie, gespielt in voller Farbe (Songs und Videos mit Ton).
- Videos lassen sich zoomen wie Bilder: Z, Doppelklick, ⌘ +/−, Zwei-Finger-Zoom, Ziehen verschiebt.
- Songs: Cover in Lightbox und Vorschau deutlich größer.
- Lightbox: Leiste mit fester Breite, lange Namen gekürzt, kein Umbruch mehr bei Videos.

## App 0.17.4 · Extension 3.15.4 (10.10.2026)

- Navigation: „Finden“ heißt jetzt „Objekte“. Der Zonen-Button entfällt, ein Klick auf „Archiver“ führt zur Übersicht.
- Ablegen: Die Zielkarte glüht kurz zart blau von hinten nach, dann blendet die Leiste aus (App und Extension).
- Vorschau: Die Leiste unten hat eine feste Breite, lange Dateinamen werden gekürzt und bricht bei Videos nicht mehr um.
- Instagram: Als Quelle wird der Beitrag gespeichert (instagram.com/p/… bzw. /reel/…), nicht mehr der Feed.

## App 0.17.3
- Rechtsklick-Menü: alle Zonen direkt sichtbar als Kästchen in zwei Spalten, Active Zones zuerst mit Taste 1–4, kein Untermenü mehr.

## Extension 3.15.3
- Instagram: eigenes Ziehen für Karussell-Bilder und Videos, weil Instagram das Ziehen des Browsers unterbindet. Ab ein paar Pixeln Bewegung erscheinen die Zonen und eine kleine Vorschau, ein Klick bleibt ein Klick, Esc bricht ab.

## Extension 3.15.2
- Instagram komplett über die Beitragsdaten: Bilder aus Karussells (über den Dateinamen zugeordnet, in Originalgröße), Videos im Feed, im Beitrag und im Karussell (über die Länge zugeordnet) als MP4 mit Ton. Standbilder von Videos werden auf Instagram nicht mehr statt des Videos gesichert.

## Extension 3.15.1
- Videos ziehen als echtes Video: Hat das Video eine Dateiadresse, wird die Datei gesichert. Auf Instagram fragt die Extension wie gängige Downloader die Beitragsdaten bei Instagram ab (mit der Anmeldung im Browser) und lädt die MP4 mit Ton im Hintergrund, auch aus Karussells und Reels. Klappt das nicht, wird das aktuelle Bild gesichert.
- Ersetzt die Versuche aus 3.14.4 (Seitendaten) und 3.15.0 (Aufnahme).

## Extension 3.14.3
- Instagram-Karussell und Bilder mit draggable="false" lassen sich ziehen; die Wisch-Geste der Seite blockiert das Ziehen nicht mehr.
- Videos ohne Standbild (Instagram u. a.): Ziehen und Ablegen sichert das aktuelle Bild des Videos als Foto.

## Extension 3.14.2
- Ziehen auf Seiten, die Bilder unter einer Fläche verstecken (Instagram u. a.): Die Extension findet das Bild unter dem Mauszeiger und macht es ziehbar. Die Active Zones erscheinen wie gewohnt.

## App 0.17.2
- Rechtsklick auf ein Bild: Vorschau, Öffnen, Quelle öffnen, Ablegen in (Active Zones 1–4, übrige unter „Weitere Zonen ›“), Kopieren, Im Finder zeigen, Als PDF ausgeben, In den Papierkorb. Gilt für die ganze Auswahl.
- „Archiver“-Hover: nur noch Blautöne, wabert langsam hin und her.

## App 0.17.1
- Hell: neutrales Hellgrau statt fast Weiß. Auf dem Mac bleibt der Hintergrund halbtransparent, nur grau getönt.
- Navigation: 4 px Innenabstand. Beim Hover gleitet eine Fläche von Button zu Button.
- „Archiver“ oben links: beim Hover fließt ein Regenbogen mit viel Hellblau durch die Schrift.

## App 0.17.0
- Suche neu: Wörter werden gegen Name, Zone, Quelle, erkannte Motive, Text im Bild und Farben gesucht, alle Wörter müssen passen. Treffer sind nach Relevanz sortiert, „rot“ findet keine „Rotation“ mehr. Deutsche Begriffe werden auf die Motiv-Erkennung übersetzt.
- „Finden“ ist eine eigene Seite: Suchfeld, Filter nach Farbe, Motiv und Quelle mit Anzahl, Treffer als eine sortierte Fläche. Die Lightbox blättert durch die Treffer.
- PDF-Ersteller (⌘P): Vorschau der Seiten, 1, 2, 4, 6 oder 12 pro Seite, A4 quer oder hoch, Bildunterschriften an/aus. Referenzen per Ziehen umsortieren, mit × weglassen, Titel frei wählbar.

## Extension 3.14.1
- Zurück auf den Stand von 3.13.0 mit Overlay (Tastenkürzel), Vollansicht und Einstellungen, statt der zu stark reduzierten 3.14.0.
- Mit App zeigen Overlay und Vollansicht nur die vier Active Zones zum Sortieren. Rechts in der Leiste führt „Archiver öffnen ↗“ in die App, dort liegt alles Weitere. Das Popup öffnet mit App ebenfalls die App.
- Einstellungen mit App: nur die vier Active Zones (01–04), ohne „Zone hinzufügen“.
- Helles Erscheinungsbild: Zeilen in den Einstellungen ohne graue Fläche.

## App 0.16.0 · Extension 3.14.0
- Hot Zones heißen jetzt Active Zones, in App und Extension.
- Tasten 1–4 legen in die vier Active Zones, passend zu den Karten der Übersicht und der Leiste beim Ziehen. 5–9 entfallen, alle anderen Zonen erreichst du über Ziehen oder „Ablegen in“.
- Startansicht: Übersicht, dann Karten, Vorschau, Index.
- Suche: Das Feld zeigt jetzt, was sie kann (Motiv, Farbe, Text im Bild).
- Neu: Als PDF ausgeben (⌘P oder Menü ≡). Die Auswahl, sonst die offene Zone oder Suche, wird ein Kontaktbogen auf A4 quer: Raster mit Dateiname und Quelle, Titel und Datum oben. Ein oder zwei Objekte erscheinen groß in voller Auflösung. Das PDF öffnet sich danach.
- Extension 3.14.0, Light-Version: Ohne App ein kleines Archiv mit Zonen und Vollansicht. Mit App sammelt sie nur, das Popup zeigt die vier Active Zones und öffnet die App. Entfallen sind das Overlay per Tastenkürzel, die Einstellungen für Kürzel und Erscheinungsbild (folgt dem System) und der Pinterest-Import.

## App 0.15.0 · Extension 3.13.0
- Zonen › Übersicht (neue Startansicht): Oben stehen die vier Hot Zones als große Karten (01–04), darunter der Index mit allen übrigen Zonen. Eine Zone auf eine andere ziehen tauscht beide, so wird eine Zone aus dem Index zur Hot Zone. Weniger als vier Zonen: Der freie Platz legt eine neue Zone an.
- Hot Zones gelten überall gleich: beim Ziehen in der App, in der Extension (nur noch diese vier statt aller Zonen) und als Tasten 1–4. Die Bibliothek merkt sie sich in .archiver/zones.json, die Extension liest sie über die App.
- Wird eine Hot Zone gelöscht, rückt die nächste aus dem Index nach. Umbenennen behält den Platz.
- Lightbox „Ablegen in“: die vier Hot Zones zuerst, durch eine feine Linie vom Rest getrennt.
- Einstellungen › Zonen: nur noch Umbenennen, Löschen und Neu. Die Reihenfolge entsteht in der Übersicht, die ersten vier Nummern sind hervorgehoben.
- Erste Schritte: neuer Schritt zu den Hot Zones.
- Extension 3.13.0: Mit App zeigt die Leiste beim Ziehen genau die vier Hot Zones. Die eigene Reihenfolge der Extension entfällt, in ihren Einstellungen sind die Hot Zones markiert.

## App 0.14.0
- Eigener Player für Audio und Video statt der Apple-Steuerung, in Lightbox und Betrachter. Die Navigationsleiste wird zweistöckig: oben Play/Pause, Zeit, Zeitlinie (klicken oder ziehen), Dauer und Ton an/aus, darunter die gewohnte Navigation. Leertaste startet und stoppt, ein Klick ins Video ebenso.

## App 0.13.1
- Audio ohne Cover: Alle Tracks ohne eingebettetes Cover (egal ob WAV, AIFF oder MP3) zeigen dasselbe Standardbild, eine ruhige Platte auf Dunkel, statt Wellenform oder Format-Kachel. Vorhandene Vorschauen werden beim ersten Start einmal neu gerechnet.

## App 0.13.0
- Ziehen: Die Zone, in der die Objekte schon liegen, leuchtet nicht mehr auf. In der Leiste unten ist sie zurückgenommen.
- Einstellungen › Zonen: Zonen lassen sich per Ziehen an der ganzen Zeile sortieren (das Ziehen ging vorher verloren). Die Pfeile ▲ ▼ sind weg. Ein Klick auf den Namen benennt weiter um.
- Details-Panel (⌘D) hat jetzt Gestalt und Inhalt der Info-Spalte aus der Lightbox: Kopf mit Name und Größe, Blöcke „Archiver“ und „Datei“, darunter Im Finder zeigen, Öffnen und In den Papierkorb. Die Zone ist dort direkt wählbar.

## App 0.12.0 · Extension 3.12.0
- Audio: WAV, MP3, AIFF, M4A und FLAC gehören jetzt in die Bibliothek. Die Vorschau ist das eingebettete Cover, ohne Cover eine ruhige Wellenform. Auf der Karte steht die Dauer („♪ 3:12“).
- Lightbox und Betrachter spielen Audio ab: Cover groß, darunter Titel, Interpret und Album und der Player. Leertaste startet und stoppt (auch bei Videos in der Lightbox).
- Details zeigen Titel, Interpret, Album und das Format (z. B. „96 kHz · 24 Bit · Stereo“).
- Archiver erscheint im Finder unter „Öffnen mit“ auch für Audiodateien.
- Lightbox: Bei PDF, Video und Audio bleibt das Etikett (Seite bzw. Dauer) stehen, statt beim Öffnen der Details auf „Einpassen“ zu springen.
- Extension 3.12.0: Links auf Audiodateien und direkt geöffnete Audiodateien werden als Datei gesichert, auch per Rechtsklick auf einen Audio-Player.

## App 0.11.0
- Startton: Lukas' eigener Sound ist der neue Standard („Archiver (Standard)“) und ersetzt „Glas“.
- Einstellungen › Anzeige › Ecken: Eckig, Weich, Rund (bisher) oder Sehr rund. Gilt für Navigation, Karten, Leisten, Knöpfe, Menüs und den Betrachter, die Bilder selbst bleiben eckig.
- Kartenansicht: Die per Pfeiltaste gewählte Karte hat jetzt die gestrichelte Linie wie ausgewählte Bilder statt der blauen.
- Menü-Icon in der Navigation etwas kleiner (17 statt 20 px breit).

## App 0.10.0
- Menü ≡ aufgeräumt: + Neue Zone (⌘N), Details (⌘D, vorher Info-Panel), Einstellungen (⌘,), Information (vorher Über), Getting Started (vorher Tour). Die Kürzel stehen in Klammern hinter den Einträgen. Importieren und Feedback senden sind in die Einstellungen gewandert.
- Startton: Beim Start klingt leise „Glas“, zwei weiche Sinustöne im Quintabstand. Unter Einstellungen › Startton lässt er sich abschalten, die Lautstärke regeln, probehören und durch eine eigene Datei ersetzen (WAV, AIFF, MP3, M4A, CAF, bis 10 MB).
- PDF-Vorschauen im Raster und auf den Karten ohne hellen Rand. Die Vorschauen werden beim ersten Start einmal neu gerechnet.
- Lightbox-Leiste: neuer Knopf „Quelle“ öffnet die Ursprungsseite (nur sichtbar, wenn es eine gibt). Der blaue Knopf zeigt die Zone des Objekts statt „Archiver“. ⌘D öffnet auch hier die Details.

## App 0.9.4
- Textkarten in der Originalansicht: gleicher Abstand wie Bilder und dieselbe gestrichelte Auswahl. Das Quellen-Etikett entfällt, die Quelle steht schon unten auf der Karte.
- Update-Animation entfernt. Nach einem Update erscheint nur noch kurz „Archiver … ist da“.

## App 0.9.3
- Textkarten sind in Archiver echter Text statt Bild: Im Raster stehen Kennung, Zitat, Quelle und Datum als Schrift, in der Lightbox groß und markierbar. Markieren und ⌘C kopiert die Auswahl, ohne Auswahl kopiert ⌘C den ganzen Text (und das Bild für Figma & Co.). Die Datei bleibt ein PNG, damit Finder, Figma und andere Programme sie weiter öffnen.
- Ältere Textkarten ohne gespeicherten Text bekommen ihn aus der Texterkennung.

## App 0.9.2
- Update-Animation neu (Referenz layout/update-bloom-referenz.png): Statt des Röntgen-Bands ziehen weiche blaue Lichtformen schräg von links unten nach rechts oben über die Oberfläche. Die Seite tritt dabei ins Nachtblau zurück, die Formen glühen am Rand und sind innen tiefer. Danach kommt „Archiver … ist da“.

## Extension 3.11.1
- Markierter Text: Statt abgedunkelter, unscharfer Zeilen liegt jetzt eine gestrichelte Outline um den ganzen markierten Block, der Rest der Seite wird dunkler. Das Blau der Markierung ist weg.
- Den Block anfassen und ziehen: Am Zeiger hängt genau dieser Block (verkleinert, wenn er groß ist), Loslassen auf einer Zone sichert ihn als Textkarte mit Text.

## App 0.9.1
- PDF: Die helle Linie am rechten und unteren Seitenrand ist weg. Vor allem bei dunklen Seiten blitzte dort der weiße Seitengrund durch, weil die Seite auf volle Pixel gerundet wurde.
- Kartenansicht: Leertaste auf einer markierten Karte öffnet die Lightbox mit den Bildern dieser Karte, ←→ blättert darin, Leertaste oder Esc führt zurück zur Karte. Vorher sprang sie immer in die erste Karte.
- Erstes Update mit Röntgen-Streifen: 0.9.0 hat sich seine Version gemerkt, 0.9.1 zeigt beim ersten Start den Streifen.

## App 0.9.0 + Extension 3.11.0
- Kartenansicht: ←→ wählt Karten aus, ↑↓ springt zur Karte darüber oder darunter, Enter öffnet die Zone. Esc aus einer Zone führt zurück auf die Startseite, die Karte bleibt ausgewählt.
- Nach einem Update läuft beim ersten Start ein Röntgen-Streifen schräg über die Oberfläche, danach kommt „Archiver 0.9.0 ist da“. Mit „Bewegung reduzieren“ im System entfällt der Streifen.
- PDF: Seiten, die oben und unten hereinragen, sind abgedunkelt und unscharf und werden zur Mitte hin scharf.
- PDF: Die weiße Linie am rechten Rand ist weg (Scrollbalken ausgeblendet).
- Video in der Lightbox zeigt vor dem Abspielen das Standbild statt nur des Play-Knopfs.
- Textkarten: Die Extension speichert den Text mit. Info (App und Lightbox) zeigt ihn markierbar mit „Text kopieren“. ⌘C legt Text und Bild zusammen ab, Texteditoren nehmen den Text, Figma & Co. das Bild. Gilt für Karten ab dieser Version.
- Extension: Markierter Text bleibt im Fokus, der Rest der Seite wird abgedunkelt und leicht unscharf. Klick daneben hebt es auf.

## App 0.8.0 + Extension 3.10.0
- PDF in der App: Im Raster erscheint Seite 1 als Vorschau mit Seitenzahl („12 S.“). Lightbox und Betrachter zeigen alle Seiten untereinander: Scrollen oder ↑↓ blättert, ←→ wechselt die Datei, Z bzw. Doppelklick zoomt, ⌘0 passt ein. In der Leiste stehen Seite und Zoom („S. 3/12 · Einpassen“). Info zeigt Seiten und Seitengröße in mm.
- Video in der App: mp4, m4v, mov und webm im Raster mit Standbild und Dauer („▶ 1:23“). In Lightbox und Betrachter wird das Video abgespielt, im Betrachter startet und stoppt es die Leertaste.
- Vorschau, Seitenzahl, Dauer und Maße kommen von macOS (Quick Look, PDFKit, AVFoundation). Bildinhalt-Suche und Texterkennung laufen auch über Seite 1 bzw. das Standbild.
- Betrachter: PDF und Video lassen sich im Finder mit Archiver öffnen und sichern. Unter Einstellungen › Betrachter gibt es dafür PDF, MP4 und MOV.
- Extension: Links auf PDFs und Videodateien (Ziehen oder Rechtsklick) werden als Datei gesichert statt als Link-Karte. Ein geöffnetes PDF sichert Rechtsklick › Ablegen in, ebenso eine Videodatei per Rechtsklick auf das Video.
- Extension: YouTube und Vimeo (Link, Tab oder Rechtsklick aufs Video) werden zur Video-Karte mit Vorschaubild, Titel und Domain. Herunterladen geht dort nicht.
- Neue Unterzeile: „Visual Library for Creatives“ im Startbild, „Eine visuelle Bibliothek für Kreative“ im Info-Text von App und Extension.

## App 0.7.8
- Lightbox mit der Leiste aus dem Betrachter: unten mittig ‹ 1 / 12 ›, Dateiname, Zoom, Info und „● Archiver“. Die Bildunterschrift und die großen Pfeile links und rechts entfallen.
- Info (Knopf oder I) öffnet rechts die Spalte wie im Betrachter: Zone, Quelle, Bild-URL, Datum, Maße, Größe, Format, Im Finder zeigen, Öffnen. Das Bild rückt dabei nach links.
- „● Archiver“ (oder Enter) klappt „Ablegen in“ mit allen Zonen auf, ein Klick verschiebt das Bild. 1–9 geht weiter direkt.
- Betrachter: „Ablegen in“ stand seit 0.7.6 als „dropBar“ da, jetzt wieder richtig beschriftet.

## Extension 3.9.3
- Markierten Text wirklich ziehen: Auf die Markierung drücken und ziehen blendet die Zonen ein, am Zeiger hängt eine Textvorschau. Loslassen auf einer Zone sichert den Text als Karte, Esc bricht ab, ein einfacher Klick hebt die Markierung wie gewohnt auf. Der Griff „In Zone ziehen“ aus 3.9.2 entfällt.

## Extension 3.9.2
- Markierten Text sichern: Nach dem Markieren erscheint an der Markierung ein kleiner Griff „In Zone ziehen“. Ihn auf eine Zone ziehen sichert den Text als Karte. Vorher ließ sich Markiertes auf manchen Seiten nicht ziehen, ein neuer Klick hat stattdessen neu markiert.

## App 0.7.7
- „Archiver“ sitzt jetzt in der Leiste, durch eine Linie von den Reitern getrennt. Ein Klick führt zum Start (Zonen als Karten).
- Startseite ohne Bibliotheksnamen, nur noch „Deine Referenzen in …“.
- Jeder Start beginnt auf der Startseite, nicht in der zuletzt geöffneten Zone.

## App 0.7.6
- Navigation oben links: „Archiver“ neben den Fensterknöpfen, rechts daneben Zonen, Alle Objekte, Suche und ≡ (Menü klappt nach unten). Die Leiste unten entfällt. Im Vollbild rückt die Zeile nach links.
- Start immer in der Karten-Ansicht. Umschalten gilt bis zum nächsten Start, die Einstellung „Startseite“ entfällt.
- Hot Zones: Beim Ziehen fahren unten die ersten vier Zonen als große Karten ein, wie in der Extension. Welche vier, bestimmt die Reihenfolge der Zonen.
- Tour zum Mitmachen (7 Schritte): erste Zone anlegen, Chrome-Erweiterung verbinden (mit Status), einmal ein Bild ablegen. Startet einmal automatisch, auch für alle, die die alte Tour schon gesehen haben.

## App 0.7.5
- Beta: ≡ › Feedback senden öffnet eine fertige Mail an feedback@archiver-app.com mit Version und drei kurzen Fragen.
- Release-Seite mit Installationsanleitung für Tester:innen (docs/beta.md), die Extension liegt dort als Archiver-Extension.zip.

## App 0.7.4
- Fensterecken: Die Versuche, die Ecken beim Füllen des Bildschirms eckig zu machen, sind wieder raus. Die Ecken bleiben rund wie bei allen Mac-Apps, im echten Vollbild (grüner Knopf) sind sie eckig.

## App 0.7.3
- Fensterecken, zweiter Anlauf: Zusätzlich zur Eckmaske wird die Rundung am Fenster und an den Ebenen des Fensterrahmens auf 0 gesetzt, solange das Fenster den Bildschirm füllt.

## App 0.7.2
- Einstiegstour: fünf Hinweise direkt an der Oberfläche (Zonen, Ablegen, Tasten, Suche, Menü). Startet nach dem Einrichten bzw. einmal nach dem Update, später unter ≡ › Tour. Enter / → weiter, ← zurück, Esc beendet.
- Leiste unten: erster Knopf heißt „Zonen“ statt Bibliotheksname.
- Menü ≡ und Zonen-Menüs im Hell-Design hell.
- Karten umsortieren: Die Zielkarte wird gestrichelt umrandet und abgetönt statt nur einer Linie darüber.

## App 0.7.1 + Extension 3.9.1
- Fensterecken: eckig, sobald das Fenster bis an die Bildschirmränder reicht (auch ohne Vollbild, mit Spielraum für Dock und Menüleiste); die Form wird beim Ziehen und Verschieben neu berechnet.
- Filterleiste nur noch unter „Alle Objekte“ und in den Einstellungen an- und abschaltbar (Darstellung › Filter in Alle Objekte, Standard: aus).
- Menü-Zeichen als drei gezeichnete Linien, deutlich größer (App und Extension). Eine ältere CSS-Regel hatte es bisher klein gehalten.
- Hell-Design neutral: Weiß und Grau ohne Gelbstich, auf dem Mac wieder halbtransparent über hellem Glas. Fensterrahmen folgt Hell/Dunkel.
- Startanimation: „Archiver“ und „Visual Researcher for Creatives“ in einer Zeile, gleich groß, kleiner. Im Hell-Design auf hellem Grund.
- App-Icon füllt die ganze Fläche, macOS legt es nicht mehr in einen grauen Rahmen.

## App 0.7.0 + Extension 3.9.0
- Suche nach Bildinhalt: Die App erkennt im Hintergrund, was auf einem Bild ist (Apple Vision, lokal), welcher Text darin steht und welche Farben es hat. Die Suche findet das mit, auch mit deutschen Begriffen („Gebäude“, „Blau“, „Plakat“).
- Filter in „Alle Objekte“, Zonen und Suche: Farbe, Quer / Hoch / Quadrat, Format, Zeitraum (7 Tage, 30 Tage, Jahr) und „Doppelt“.
- Hell / Dunkel / System für App, Betrachter und Extension. Hell ist Papierweiß mit fast schwarzer Schrift, die Auswahl gestrichelt schwarz. App: Einstellungen › Darstellung; Extension: Einstellungen › Erscheinungsbild.
- Erster Start in drei Schritten: Bibliothek wählen, Extension verbinden (erkennt die Verbindung selbst), erstes Bild ablegen.
- Auto-Update: Die App prüft beim Start, ob es eine neue Version gibt, und installiert sie auf Klick. Updates sind signiert und kommen aus lr-practice/Archiver-Releases.
- Extension: Rechtsklick „Ablegen in › Zone“ für Bilder, auch solche, die sich nicht ziehen lassen (Hintergrundbilder, Bilder unter Overlays). „Ausschnitt sichern“ (⌥⇧S oder Rechtsklick): Rahmen aufziehen, Zone mit 1–9 wählen.
- Extension: Markierten Text, Links und ganze Seiten als Karte sichern (Rechtsklick oder Ziehen auf eine Zone); ohne Bild wird der sichtbare Seitenausschnitt gespeichert.
- Extension: Liegt ein Bild schon in der Bibliothek, sagt die Meldung, in welcher Zone.

## App 0.6.1 + Extension 3.8.4
- Originalansicht: feste Reihenfolge. Bild 1, 2, 3 … laufen Spalte für Spalte nach rechts und beginnen dann wieder links, ← → springen nicht mehr hin und her.
- Menü-Zeichen ≡ größer (App und Extension).

## App 0.6.0 + Extension 3.8.3
- Neue Startanimation: schwarze Fläche, „Archiver“ gleitet von links herein, darunter „Visual Researcher for Creatives“, dann blendet alles aus. Startet erst, wenn das Fenster sichtbar ist.
- Lightbox zoomt wie der Betrachter: Zwei-Finger-Zoom und ⌘ + Scrollen zum Mauszeiger, Ziehen verschiebt, Doppelklick wechselt Einpassen ↔ 100 %, ⌘+ / ⌘− / ⌘0, Esc passt erst ein. Zoomwert unten rechts.
- ⌘C kopiert die ausgewählten Bilder bzw. das Bild in der Lightbox (auch im Betrachter) als Bilddaten in die Zwischenablage.
- Originalansicht: eigene Spalten statt CSS-Spalten. Die Reihenfolge läuft zeilenweise von links nach rechts, ← → folgen ihr, ↑ ↓ bleiben in der Spalte.
- + / − auf der Tastatur ändern die Bildgröße wieder (auch auf der Startseite).
- Karten-Ansicht: Zonen-Karten per Ziehen umsortieren (gilt wie Einstellungen › Zonen auch für 1–9 und den Betrachter).
- Klick in den Hintergrund hebt die Auswahl auch in einer geöffneten Zone auf.
- „Ablegen in“-Leiste beim Ziehen: Der Drop nimmt das zuletzt markierte Ziel, die Leiste verschwindet erst danach. Vorher landete der Drop oft in der eigenen Zone darunter, nichts wurde verschoben.
- Fensterecken: Füllt das Fenster den ganzen Bildschirm (Vollbild oder „Füllen“), ist es eckig und reicht bis in die Ecken; kleinere Fenster bleiben abgerundet.
- App und Extension: „Mehr“ ist jetzt das Zeichen ≡ (Tooltip „Mehr“).
- Extension: Knopf „Board importieren“ im Popup im Stil der übrigen Knöpfe.

## Extension 3.8.1: Pinterest-Boards importieren

- Auf einer Pinterest-Board-Seite zeigt das Popup „Board importieren“. Die Seite scrollt durch, sammelt alle Pins (ohne „Mehr Ideen“) und legt sie in einer Zone mit dem Board-Namen ab. Erstmal als Entwicklungshilfe.

## 3.8.0 + App 0.5.0: aus einem Guss
- Ein Verb: überall „Ablegen in“ (App, Betrachter, Extension-Zonenleiste); Meldungen „In … abgelegt“.
- Eine Navigation: Einstellungen und Info liegen in App und Extension unter „Mehr“; eine offene Seite steht als aktiver Schritt in der Leiste.
- Eine Farbregel: Auswahl gestrichelt in Weiß, die übrigen Bilder treten zurück (abgedunkelt); Blau nur für Ablageziele und den Archiver-Knopf im Betrachter.
- Extension: Zonen werden sofort übernommen (kein „Speichern“-Knopf mehr).
- App: Zonen in den Einstellungen löschen (mit Rückfrage). Bildgröße auf der Startseite pro Ansicht (Index, Vorschau, Karten) getrennt; − / + steuern in der Vorschau die Größe der Vorschaubilder. „Ablegen in“ zeigt die Tastenzahlen 1–9.
- Betrachter: rauszoomen bis unter „Einpassen“, die Zoomstufe bleibt beim Blättern. Rechts nur noch „● Archiver“: grau = noch nicht abgelegt (Klick: Ablegen in), blau = in der Bibliothek (Klick: in der App zeigen); der linke „Archiver“-Knopf entfällt. Info gleitet als Seite von rechts herein wie in der App. Zoomwert als Mono-Steuerung „(EINPASSEN)“.

## App 0.4.3
- Einstellungen › Zonen wie in der Extension: Reihenfolge per Griff oder ▲ ▼ (gilt auch für die Tasten 1–9 und den Betrachter), Klick auf den Namen benennt um, „+ Zone hinzufügen“.
- Einstellungen in einklappbaren Blöcken (Bibliothek, Zonen, Darstellung, Betrachter, Extension, Tasten); der Zustand bleibt gemerkt.
- Betrachter: Bild ziehen zeigt über der Leiste „Ablegen in“ mit allen Zonen wie in der App; das Bild lässt sich auch in andere Programme ziehen. „In Archiver sichern“ öffnet dieselbe Leiste.
- Behoben: In der Originalansicht war der Auswahlrahmen in der obersten Reihe abgeschnitten.

## App 0.4.2
- Einstellungen › Betrachter: ein Schalter pro Format (JPEG, PNG, GIF, WebP, TIFF, HEIC, AVIF, BMP, SVG) statt einem Knopf für alle. „Aus“ erklärt, wie man im Finder zurück zu Vorschau wechselt.
- Betrachter-Zoom: Zwei-Finger-Zoom und ⌘ + Scrollen zum Mauszeiger, Ziehen und Scrollen verschiebt, Doppelklick oder Leertaste wechselt zwischen Einpassen und 100 %, ⌘+ / ⌘− / ⌘0. Der Zoomwert steht in der Leiste.
- Betrachter-Info (Taste I oder „Info“): Maße, Dateigröße, Format, Datum, Ordner und Zone in Archiver, dazu „Im Finder zeigen“.
- Betrachter startet zuverlässig mit dem angeklickten Bild (Abgleich über die Datei statt über die Schreibweise des Namens) und schneller: Das Bild erscheint sofort, Ordner und Zonen laden danach. Nachbarbilder werden vorgeladen, Sortierung wie im Finder (Zahlen natürlich).

## App 0.4.1
- Einstellungen › Betrachter: „Als Standard festlegen“ macht Archiver im Finder zum Standardprogramm für alle Bildformate (JPEG, PNG, GIF, WebP, TIFF, HEIC, AVIF, BMP, SVG). „Immer öffnen mit“ im Finder gilt dagegen nur für eine Datei.

## App 0.4.0
- Betrachter: Archiver lässt sich im Finder über „Öffnen mit“ für JPEG, PNG, GIF, WebP, TIFF, HEIC, AVIF, BMP und SVG wählen. Das Bild öffnet in einem eigenen Fenster, ← → blättern durch den Ordner, Klick oder Leertaste zeigt die Originalgröße.
- „In Archiver sichern“ kopiert das Bild in eine Zone (oder Taste 1–9). Liegt es schon in der Bibliothek, steht dort „In Archiver zeigen“: die App kommt nach vorn und wählt das Bild in seiner Zone aus. „Archiver“ links öffnet die App.
- Wird die App mit einem Bild gestartet, erscheint nur der Betrachter; ein Klick aufs Dock-Symbol holt die App.
- Info: „So funktioniert’s“ mit Sammeln, Bibliothek, Sortieren, Ansehen, Teilen und Betrachter.

## App 0.3.2
- „Alle Bilder“ heißt jetzt „Alle Objekte“ (englisch „All items“).
- Einstellungen › Darstellung: Sprache Deutsch / English. Ohne Auswahl gilt weiter die Systemsprache.

## 3.7.4 + App 0.3.1
- App: Einstellungen und Info gleiten wie in der Extension von rechts herein und schieben die Übersicht nach links (⋯ › Einstellungen / Info, Esc oder „Archiver“ schließt). Inhalt in Blöcken mit feinen Linien, typografisch reduziert; Info mit Zahlen zur Sammlung und Version.
- Extension: knappere Innenabstände in Einstellungen und Info, Seitentitel 28 px wie in der App.

## 3.7.3
- Einstellungen und Information gleiten von rechts herein und schieben die Übersicht nach links; die Karten brechen dabei neu um, die Navigation wandert mit.
- Popup (Klick auf das Erweiterungs-Symbol) mit denselben Grauwerten und Abständen wie die Seiten: dunkler Grund, Blöcke, Kopfzeile wie die Navigation.
- Behoben: Karten wurden bei wenig Platz in der Höhe abgeschnitten; in der Info stand „[object HTMLSpanElement]“ statt der Tastenkombination.

## 3.7.2
- Einstellungen und Information sind keine schwebenden Panels mehr, sondern eigene Seiten anstelle der Übersicht. „Archiver“ in der Navigation führt zurück.
- Typografisch reduziert nach Referenz: Titel weiß mit grauem Satz dahinter in einer Größe, Inhalte in Blöcken mit feinen Linien statt Flächen. Info zeigt „So funktioniert’s“ und „Tasten“ als Zeilen.
- Navigation nur noch mit Archiver › Zone, Einstellungen und Information. Suche und Größenregler entfernt (die gemerkte Bildgröße gilt weiter).
- Kleinere Eckenradien in Extension und App (Karten und Navigation 8 px statt 10 bis 12 px).

## 3.7.1
- Übersicht: Zonenkarten füllen immer die ganze Breite (höchstens vier pro Reihe, bei weniger Zonen werden sie breiter). Leere Zonen haben eine ruhige Fläche statt Bild, alle Karten sind gleich hoch.
- Einstellungen und Information gehen als abgerundete Fläche direkt über der Navigation auf, im Stil der Navigation. Info mit großer Wortmarke, Claim und Versionsnummer, Text auf den aktuellen Stand gebracht.

## 3.7.0 + App 0.3.0 (Phase 4, ein Design)
- App: Das Prototyp-Design ist jetzt die einzige Variante. Schwebende Navigation unten mit Pfad (Archiver › Zone), Ansichten Index, Vorschau und Karten, Größe in jeder Ansicht einstellbar. Zonen-Aktionen (Umbenennen, Im Finder zeigen, Löschen) im ⋯-Menü, auch innerhalb einer Zone.
- App: Einstellungen als eigene Seite (Bibliothek, Verbindung zur Extension, Startseite, Bilder, Infopanel, Tasten) und eine Info-Seite mit großer Wortmarke und Zahlen zur Sammlung.
- App: Ablageposition beim Ziehen aus dem Finder stimmt auf Retina-Displays wieder.
- Extension: Übersicht in Overlay und Vollansicht im Karten-Stil der App. Jede Zone ist eine abgerundete Karte mit bis zu drei Vorschaubildern, Klick öffnet die Zone. Navigation schwebt unten (Archiver › Zone, Suche, Größe, Einstellungen, Information). Beim Ziehen eines Bildes erscheint eine Leiste mit allen Zonen als Ziel.
- Extension: Ablage-Zonen beim Ziehen auf Webseiten als schwebende, abgerundete Karten.

## 3.6.0 + App 0.2.0 (Phase 3, Brücke)
- Extension und App arbeiten zusammen. Ist die App installiert und eine Bibliothek eingerichtet, erkennt die Extension das automatisch: Zonen kommen aus der Bibliothek, gesicherte Bilder landen direkt im Zonen-Ordner, samt Quelle (Seite, Titel, Bild-URL). Ohne App bleibt die Extension eigenständig wie bisher.
- Technik: Chrome Native Messaging. Die App meldet sich bei jedem Start als Host `app.archiver.bridge` bei Chrome, Chrome Beta/Canary, Chromium, Brave, Edge, Vivaldi und Arc an. Chrome startet dasselbe Programm ohne Fenster, das direkt auf dem Bibliotheksordner arbeitet. Die App muss dafür nicht laufen; läuft sie, erscheint das Bild sofort.
- Gespeichert wird über einen kurzen Umweg: Chrome lädt das Bild nach `Downloads/Archiver-Transfer`, die App verschiebt es in die Zone. Der Ordner bleibt leer, der Eintrag im Download-Verlauf wird entfernt. Bilder, die schon in der Bibliothek liegen, werden nicht doppelt angelegt.
- Galerie im Overlay und in der Vollansicht zeigt mit App den Inhalt der Bibliothek (Vorschaubilder kommen von der App). Verschieben zwischen Zonen und „Im Finder zeigen“ wirken direkt auf die Bibliothek.
- Einstellungen mit App: „Mit der Archiver-App verbunden“. Zonen lassen sich sortieren und neu anlegen (wird ein neuer Ordner), umbenennen und löschen geht in der App. Die eigene Reihenfolge der Extension bleibt erhalten.
- Extension hat jetzt eine feste ID (`key` im Manifest), damit die App sie erkennt. Neue Berechtigung `nativeMessaging`.

## App 0.1.0 (Phase 2, erste Testversion)
- Neue Desktop-App für den Mac in `app/` (Tauri 2). Die Bibliothek ist ein normaler Ordner (Standard `~/Archiver`), jeder Unterordner eine Zone, `Inbox` ist die Standard-Zone.
- Aufbau wie die Vollansicht der Extension: Zonen als Kacheln mit gestrichelter Outline (bis 4 Spalten, ab der fünften Zone eine weitere Reihe), jede Kachel zeigt ihre Bilder. Klick auf den Zonennamen zeigt nur diese Zone, „‹ Alle Zonen“ oder Esc führt zurück. Fenster halbtransparent mit macOS-Unschärfe, Infopanel schwebt rechts.
- Raster (quadratisch oder Originalformat), Größe per − / +, Lightbox mit ←/→ und Zoom, Infopanel mit Quelle, Maßen, Format, Größe und Datum, Suche über Dateiname, Seitentitel und Adresse.
- Sortieren: Tasten 1–9 verschieben die Auswahl in die jeweilige Zone und springen zum nächsten Bild (schneller Inbox-Durchgang). Bilder auf eine andere Zonen-Kachel ziehen verschiebt sie im Dateisystem. ⌫ legt in den Papierkorb.
- Startseite ist die Übersicht „Zonen“: nur die Kacheln mit großem Namen, Bildanzahl und letztem Speicherdatum, ohne Bilder. Klick öffnet die Zone, Bilder lassen sich weiter direkt auf eine Kachel ziehen. „Inhalte“ zeigt alle Zonen mit ihren Bildern.
- Zonen-Menü (⋯) auf jeder Kachel: Umbenennen, Im Finder zeigen, In den Papierkorb (mit Bestätigung). Inbox lässt sich nicht umbenennen oder löschen.
- Dezenter Ladebildschirm beim Start (gestrichelte Zone mit Wortmarke). Stärkere Transparenz auf dem Mac.
- Zonen anlegen und per Doppelklick auf den Namen umbenennen. Änderungen im Finder erscheinen sofort (Ordner-Watcher).
- Bilder aus der App direkt in Finder, Figma oder InDesign ziehen. Bilder aus dem Finder auf eine Zone ziehen kopiert sie in die Bibliothek.
- Import aus der Extension (`archiver-export.json`, mit Quellen) oder aus einem beliebigen Ordner. Doppelte Bilder werden erkannt.
- Eigene Vorschaubilder in `.archiver/thumbs`, Datenbank `.archiver/library.db`, Quellen zusätzlich in `.archiver/metadata.json`. Quellen hängen am Inhalt (Hash) und bleiben beim Umbenennen oder Verschieben im Finder erhalten.

## 3.5.0
- Einstellungen: neuer Abschnitt „Desktop-App" mit „Für die App exportieren". Legt `archiver-export.json` in den Downloads ab, mit Zonen, Speicherort und Quelle jedes gesicherten Bildes. Die App importiert damit alles samt Quellen.

## 3.4.0
- Eine Galerie für alles: Overlay und Vollansicht nutzen denselben Baustein (`ui.js`), ebenso Einstellungen und Information (auch im Popup). Rund 1.600 Zeilen weniger, Änderungen wirken künftig überall gleichzeitig.
- Overlay im Shadow DOM: Seiten-CSS kann das Overlay nicht mehr verbiegen und umgekehrt. Funktioniert auch auf Seiten mit strenger Content-Security-Policy. `content.css` entfällt, dafür `overlay.css`.
- Englisch als Standardsprache, Deutsch zusätzlich (`_locales/en`, `_locales/de`). Chrome wählt nach Browsersprache.
- Tastenkombination zusätzlich als Chrome-Befehl (Alt+Shift+A, änderbar unter `chrome://extensions/shortcuts`).
- Fix macOS: Die eigene Kombination mit Option (z. B. Option+Shift+A) wird jetzt auch erkannt, wenn Option das Zeichen verändert.
- Einstellungen im Overlay: Link „Downloads-Ordner ändern" wie im Popup.
- Bildgröße der Galerie wird für Overlay und Vollansicht gemeinsam gemerkt.
- Overlay und Vollansicht sind jetzt dieselbe Ansicht: gleiche Toolbar („Archiver", Filter, Größe, Einstellungen, Information), gleiche Karten, Panel rechts unter der Toolbar. Unterschied nur der Hintergrund: im Overlay halbtransparent über der Seite, in der Vollansicht solide.
- Das Overlay per Tastenkombination zeigt keine Zonenleiste mehr. Die Zonenleiste erscheint nur noch beim Ziehen eines Bildes. Bilder vom Desktop lassen sich im Overlay direkt auf eine Zonen-Karte ziehen.
- Zonen-Karten füllen die ganze Höhe: eine Spalte pro Zone (bis 4), ab der fünften Zone eine weitere Reihe. Jede Karte scrollt für sich.
- Klick auf den Namen einer Zone zeigt nur diese Zone, „‹ Alle Zonen" oder Esc führt zurück.
- Zonenleiste: Downloads-Zone gestrichelt wie alle anderen, keine Extralinie mehr.

## 3.3.0
- Quelle wird mitgesichert: Beim Ziehen aus einer Webseite speichert Archiver Seiten-URL und Seitentitel im Index. Beim Verschieben bleiben Quelle und ursprüngliches Sicherungsdatum erhalten.
- Lightbox (Overlay und Vollansicht) zeigt die Domain der Quellseite als Link.
- Overlay: Einstellungen und Information öffnen als schwebendes Panel rechts (wie in der Vollansicht), die Galerie bleibt sichtbar. Die beiden Buttons sitzen in der Toolbar und überdecken Filter und Größenregler nicht mehr.
- Galerie (Overlay und Vollansicht): Klick aufs Bild öffnet die Lightbox, „+" beim Hovern zeigt die Datei im Finder (vorher umgekehrt).
- Keine Konsolen-Ausgaben mehr auf fremden Seiten. Diagnose über `DEBUG` in `shared.js` einschaltbar.

## 3.2.0
Ausgangsstand, siehe `docs/HANDOFF-v3.2.0.md`.

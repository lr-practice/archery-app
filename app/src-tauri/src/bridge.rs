// bridge.rs
// Brücke zur Chrome-Extension über Chrome Native Messaging.
//
// Chrome startet dasselbe Programm wie die App, nur mit der Herkunft der
// Extension als Argument ("chrome-extension://<id>/"). Dann öffnet sich kein
// Fenster: Das Programm liest Anfragen von stdin und antwortet auf stdout
// (je 4 Byte Länge + JSON), bis Chrome die Verbindung schließt.
//
// Safari (Safari-Extension, in Arbeit): Safari erreicht per Native Messaging nur die
// App-Erweiterung in Archiver.app, nicht dieses Programm. Die laufende App
// beantwortet dieselben Befehle deshalb auch über HTTP auf 127.0.0.1
// (http_bridge.rs). Beide Wege teilen sich `dispatch` und die offenen
// Bibliotheken.
//
// Damit Chrome das Programm findet, legt die App bei jedem Start eine kleine
// Manifest-Datei in den NativeMessagingHosts-Ordner jedes installierten
// Chromium-Browsers. Läuft die App nicht, funktioniert die Brücke trotzdem:
// Sie arbeitet direkt auf dem Bibliotheksordner. Eine laufende App bemerkt
// neue Dateien über ihren Ordner-Watcher.

use crate::library::{now_ms, Library, Source, Zone};
use base64::Engine;
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub const HOST_NAME: &str = "app.archery.bridge";
pub const APP_IDENTIFIER: &str = "app.archery.desktop";
// Bis 0.43: alte Kennung und alter Hostname, werden beim Start übernommen bzw. entfernt.
const OLD_HOST_NAME: &str = "app.archiver.bridge";
const OLD_APP_IDENTIFIER: &str = "app.archiver.desktop";

/// Einstellungen und Studio-Daten vom alten Ordner (app.archiver.desktop) in den
/// neuen übernehmen (0.44.0). Nur wenn es den neuen noch nicht gibt.
pub fn migrate_app_dir() {
    let Some(base) = dirs::config_dir() else { return };
    let (old, new) = (base.join(OLD_APP_IDENTIFIER), base.join(APP_IDENTIFIER));
    if old.is_dir() && !new.exists() {
        let _ = std::fs::rename(old, new);
    }
}
const LIST_LIMIT: usize = 500;
/// Chrome nimmt vom Host höchstens 1 MB je Nachricht; etwas Luft bleibt
const MAX_OUT: usize = 900 * 1024;
const SCAN_EVERY: Duration = Duration::from_secs(30);

/// Erlaubte Extensions. Die erste ID ergibt sich aus dem "key" in
/// extension/manifest.json (entpackte Version). Die Chrome-Web-Store-ID kommt
/// in Phase 4 dazu.
const EXTENSION_IDS: &[&str] = &["gddcmkfpdfkpnieegdplpilmoodghgdd"];

/// Version der Extension, die diese App mindestens braucht. Meldet sich eine
/// ältere (oder eine ohne Versionsangabe, vor 3.22.0), zeigt die App einen Hinweis.
pub const EXTENSION_MIN: &str = "3.24.0";

/// Wurde das Programm von Chrome als Native-Messaging-Host gestartet?
pub fn is_host_invocation() -> bool {
    std::env::args().skip(1).any(|a| a.starts_with("chrome-extension://"))
}

// ---- Registrierung in Chrome ----

/// Ordner, in denen Chromium-Browser nach Native-Messaging-Hosts suchen
/// (nur die, deren Browser auf diesem Rechner installiert ist).
fn host_dirs() -> Vec<(&'static str, PathBuf)> {
    let mut out = Vec::new();
    #[cfg(target_os = "macos")]
    if let Some(base) = dirs::home_dir().map(|h| h.join("Library/Application Support")) {
        for (name, browser) in [
            ("Chrome", "Google/Chrome"),
            ("Chrome Beta", "Google/Chrome Beta"),
            ("Chrome Dev", "Google/Chrome Dev"),
            ("Chrome Canary", "Google/Chrome Canary"),
            ("Chromium", "Chromium"),
            ("Brave", "BraveSoftware/Brave-Browser"),
            ("Edge", "Microsoft Edge"),
            ("Vivaldi", "Vivaldi"),
            ("Arc", "Arc/User Data"),
        ] {
            let dir = base.join(browser);
            if dir.is_dir() {
                out.push((name, dir.join("NativeMessagingHosts")));
            }
        }
    }
    #[cfg(target_os = "linux")]
    if let Some(base) = dirs::config_dir() {
        for (name, browser) in [
            ("Chrome", "google-chrome"),
            ("Chrome Beta", "google-chrome-beta"),
            ("Chromium", "chromium"),
            ("Brave", "BraveSoftware/Brave-Browser"),
        ] {
            let dir = base.join(browser);
            if dir.is_dir() {
                out.push((name, dir.join("NativeMessagingHosts")));
            }
        }
    }
    out
}

/// Browser, für die die Brücke eingerichtet ist (Anzeige in den Einstellungen).
pub fn connected_browsers() -> Vec<String> {
    host_dirs()
        .into_iter()
        .filter(|(_, dir)| dir.join(format!("{HOST_NAME}.json")).is_file())
        .map(|(name, _)| name.to_string())
        .collect()
}

/// Schreibt das Host-Manifest für alle gefundenen Browser. Läuft bei jedem
/// App-Start, damit der Pfad auch nach dem Verschieben der App stimmt.
pub fn install_manifests() -> Vec<PathBuf> {
    let Ok(exe) = std::env::current_exe() else { return Vec::new() };
    let manifest = json!({
        "name": HOST_NAME,
        "description": "Archery",
        "path": exe.to_string_lossy(),
        "type": "stdio",
        "allowed_origins": EXTENSION_IDS.iter().map(|id| format!("chrome-extension://{id}/")).collect::<Vec<_>>(),
    });
    let text = serde_json::to_vec_pretty(&manifest).unwrap();
    let mut written = Vec::new();
    for (_, dir) in host_dirs() {
        let _ = std::fs::remove_file(dir.join(format!("{OLD_HOST_NAME}.json")));
        let file = dir.join(format!("{HOST_NAME}.json"));
        if std::fs::read(&file).ok().as_deref() == Some(text.as_slice()) {
            written.push(file);
            continue;
        }
        if std::fs::create_dir_all(&dir).is_ok() && std::fs::write(&file, &text).is_ok() {
            written.push(file);
        }
    }
    written
}

// ---- Welche Extension hat sich zuletzt gemeldet? ----
// Der Host schreibt bei jedem "hello" Version und Zeit in extension.json neben
// die Einstellungen der App. Die App liest das für den Hinweis „fehlt/veraltet“.

fn extension_file() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join(APP_IDENTIFIER).join("extension.json"))
}

fn note_extension(msg: &Value) {
    let Some(file) = extension_file() else { return };
    let before: Value = std::fs::read_to_string(&file).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(Value::Null);
    // Safari zusätzlich vermerken, damit Einstellungen › Mac & Browser es zeigt
    let safari = if str_arg(msg, "browser") == Some("safari") { json!(now_ms()) } else { before.get("safari").cloned().unwrap_or(Value::Null) };
    let entry = json!({ "version": str_arg(msg, "ext"), "seen": now_ms(), "safari": safari });
    if let Some(dir) = file.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(file, serde_json::to_vec(&entry).unwrap());
}

/// Zuletzt gemeldete Extension (Version, Zeit) und die Mindestversion.
pub fn extension_status() -> Value {
    let seen: Value = extension_file()
        .and_then(|f| std::fs::read_to_string(f).ok())
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(Value::Null);
    json!({
        "version": seen.get("version").cloned().unwrap_or(Value::Null),
        "seen": seen.get("seen").cloned().unwrap_or(Value::Null),
        "safari": seen.get("safari").cloned().unwrap_or(Value::Null),
        "safariBridge": crate::http_bridge::ENABLED,
        "min": EXTENSION_MIN,
    })
}

// ---- Host ----

/// Bibliotheksordner aus den Einstellungen der App.
fn library_root() -> Option<PathBuf> {
    let root = PathBuf::from(app_settings()?.library?);
    root.is_dir().then_some(root)
}

/// Bibliothek für eine Anfrage: `library` (Pfad) nur, wenn sie in der App als
/// Bibliothek eingetragen ist (⇧ beim Ziehen in der Extension), sonst die aktive.
fn library_root_for(msg: &Value) -> Option<PathBuf> {
    if let Some(want) = str_arg(msg, "library") {
        let known = app_settings()
            .map(|s| s.libraries.iter().any(|l| l.path.as_deref() == Some(want)) || s.library.as_deref() == Some(want))
            .unwrap_or(false);
        let root = PathBuf::from(want);
        if known && root.is_dir() {
            return Some(root);
        }
    }
    library_root()
}

/// Alle Bibliotheken aus den Einstellungen, die es gibt, die aktive markiert.
fn library_list() -> Vec<Value> {
    let Some(s) = app_settings() else { return vec![] };
    let active = s.library.clone();
    let mut out: Vec<Value> = s
        .libraries
        .iter()
        .filter_map(|l| {
            let path = l.path.clone()?;
            if !Path::new(&path).is_dir() {
                return None;
            }
            let name = if l.name.trim().is_empty() {
                Path::new(&path).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
            } else {
                l.name.clone()
            };
            Some(json!({ "name": name, "path": path, "active": active.as_deref() == Some(path.as_str()) }))
        })
        .collect();
    if out.is_empty() {
        if let Some(root) = library_root() {
            let name = root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            out.push(json!({ "name": name, "path": root.to_string_lossy(), "active": true }));
        }
    }
    out
}

#[derive(Deserialize, Default)]
struct LibrarySlotSetting {
    #[serde(default)]
    name: String,
    #[serde(default)]
    path: Option<String>,
}

#[derive(Deserialize)]
struct AppSettings {
    library: Option<String>,
    #[serde(default)]
    user: Option<String>,
    #[serde(default)]
    libraries: Vec<LibrarySlotSetting>,
    #[serde(default)]
    naming: crate::naming::Naming,
}

fn app_settings() -> Option<AppSettings> {
    let file = dirs::config_dir()?.join(APP_IDENTIFIER).join("settings.json");
    serde_json::from_str(&std::fs::read_to_string(file).ok()?).ok()
}

/// Chrome schickt höchstens 64 MiB an den Host; mehr ist ein Fehler im Strom.
const MAX_IN: usize = 64 * 1024 * 1024;

/// Nächste Nachricht. `None`: Chrome hat die Verbindung geschlossen.
/// `Some(Err)`: diese eine Nachricht ist unbrauchbar, die Brücke läuft weiter (0.31.0).
fn read_message(input: &mut impl Read) -> Option<Result<Value, String>> {
    let mut len = [0u8; 4];
    input.read_exact(&mut len).ok()?;
    let len = u32::from_ne_bytes(len) as usize;
    if len > MAX_IN {
        // Länge passt nicht zu dem, was Chrome schicken darf: der Strom ist nicht mehr lesbar
        crate::log::error("bridge", format!("Nachricht mit {len} Bytes, Verbindung beendet"));
        return None;
    }
    let mut buf = vec![0u8; len];
    input.read_exact(&mut buf).ok()?;
    Some(serde_json::from_slice(&buf).map_err(|e| format!("bad-message: {e}")))
}

fn write_message(out: &mut impl Write, value: &Value) -> std::io::Result<()> {
    let bytes = serde_json::to_vec(value).unwrap();
    out.write_all(&(bytes.len() as u32).to_ne_bytes())?;
    out.write_all(&bytes)?;
    out.flush()
}

/// Offene Bibliotheken, geteilt von stdin-Host und HTTP-Brücke. Der Mutex
/// reiht die Anfragen der HTTP-Brücke (je Verbindung ein Thread) hintereinander.
static SLOT: Mutex<Vec<(PathBuf, Library)>> = Mutex::new(Vec::new());

/// Eine Anfrage beantworten: `{ ok: true, … }` oder `{ ok: false, error }`,
/// dazu die `id` der Anfrage. `data`: Datei, die die HTTP-Brücke für
/// "save_data" in einen eigenen Zwischenordner geschrieben hat.
pub(crate) fn dispatch(msg: &Value, data: Option<&Path>) -> Value {
    let id = msg.get("id").cloned().unwrap_or(Value::Null);
    let result = match msg.get("error").and_then(|e| e.as_str()) {
        Some(e) => Err(e.to_string()),
        None => {
            let mut slot = SLOT.lock().unwrap_or_else(|e| e.into_inner());
            handle(msg, &mut slot, data)
        }
    };
    let mut reply = match result {
        Ok(mut v) => {
            v["ok"] = json!(true);
            v
        }
        Err(e) => {
            crate::log::error("bridge", format!("{}: {e}", msg.get("cmd").and_then(|t| t.as_str()).unwrap_or("?")));
            json!({ "ok": false, "error": e })
        }
    };
    reply["id"] = id;
    reply
}

/// Hauptschleife im Host-Modus. Endet, wenn Chrome stdin schließt.
pub fn run_host() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut input = stdin.lock();
    let mut output = stdout.lock();
    while let Some(msg) = read_message(&mut input) {
        let msg = msg.unwrap_or_else(|e| json!({ "type": "", "error": e }));
        let id = msg.get("id").cloned().unwrap_or(Value::Null);
        let mut reply = dispatch(&msg, None);
        // Zu groß für Chrome: Chrome würde die Verbindung kappen, alle offenen
        // Anfragen gingen verloren. Stattdessen ein klarer Fehler (0.31.0).
        if serde_json::to_vec(&reply).map(|b| b.len() > 1024 * 1024).unwrap_or(true) {
            crate::log::error("bridge", "Antwort größer als 1 MB");
            reply = json!({ "ok": false, "error": "reply-too-large", "id": id });
        }
        if write_message(&mut output, &reply).is_err() {
            break;
        }
    }
}

/// Öffnet die Bibliothek der Anfrage bei Bedarf (aktive oder per ⇧ gewählte).
/// Geöffnete Bibliotheken bleiben offen, damit der Wechsel per ⇧ schnell ist.
fn library<'a>(slot: &'a mut Vec<(PathBuf, Library)>, msg: &Value) -> Result<&'a Library, String> {
    let root = library_root_for(msg).ok_or("Keine Bibliothek eingerichtet")?;
    let i = match slot.iter().position(|(r, _)| r == &root) {
        Some(i) => i,
        None => {
            slot.push((root.clone(), Library::open(&root)?));
            slot.len() - 1
        }
    };
    // Name für „abgelegt von“ kann sich in der App jederzeit ändern
    let settings = app_settings();
    slot[i].1.set_user(settings.as_ref().and_then(|s| s.user.clone()));
    Ok(&slot[i].1)
}

/// Ordner innerhalb der Zonen (bis zwei Ebenen tief) fürs Rechtsklickmenü der Extension.
/// Ordner je Zone (bis zwei Ebenen) fürs Rechtsklickmenü. Aus den Ordnern
/// selbst statt aus allen Objekten der Bibliothek (0.31.0): schneller, und auch
/// leere, gerade in der App angelegte Ordner stehen im Menü.
fn zone_folders(lib: &Library, zones: &[Zone]) -> Value {
    let mut sets: std::collections::BTreeMap<String, std::collections::BTreeSet<String>> = Default::default();
    for dir in lib.folders() {
        // die längste passende Zone (Unterzonen haben eigene Einträge)
        let Some(zone) = zones.iter().filter(|z| !z.path.is_empty() && dir.starts_with(&format!("{}/", z.path))).max_by_key(|z| z.path.len()) else {
            continue;
        };
        let rest = &dir[zone.path.len() + 1..];
        if zones.iter().any(|z| z.path == dir) {
            continue; // selbst eine Zone
        }
        let parts: Vec<&str> = rest.split('/').take(2).collect();
        let set = sets.entry(zone.path.clone()).or_default();
        set.insert(parts[0].to_string());
        if parts.len() > 1 {
            set.insert(format!("{}/{}", parts[0], parts[1]));
        }
    }
    Value::Object(sets.into_iter().map(|(zone, set)| (zone, json!(set.into_iter().collect::<Vec<_>>()))).collect())
}

/// Die Brücke liest den Ordner höchstens alle 30 s neu ein. Vorher lief bei
/// jedem Ziehen und jeder Galerie ein ganzer Scan, auch auf Netzlaufwerken
/// (0.31.0). Läuft die App, hält ihr Watcher die Datenbank ohnehin aktuell.
fn scan_now_and_then(lib: &Library) -> Result<(), String> {
    static LAST: Mutex<Vec<(PathBuf, Instant)>> = Mutex::new(Vec::new());
    {
        let mut last = LAST.lock().unwrap_or_else(|e| e.into_inner());
        if last.iter().any(|(r, at)| r == &lib.root && at.elapsed() < SCAN_EVERY) {
            return Ok(());
        }
        last.retain(|(r, _)| r != &lib.root);
        last.push((lib.root.clone(), Instant::now()));
    }
    lib.scan().map(|_| ())
}

/// Zonen wie in der App: ohne Bibliotheksordner selbst und ohne leere Elternordner.
fn visible_zones(zones: Vec<Zone>) -> Vec<Zone> {
    let paths: Vec<String> = zones.iter().map(|z| z.path.clone()).collect();
    zones
        .into_iter()
        .filter(|z| !z.path.is_empty())
        .filter(|z| z.count > 0 || !paths.iter().any(|p| p.starts_with(&format!("{}/", z.path))))
        .collect()
}

fn str_arg<'a>(msg: &'a Value, key: &str) -> Option<&'a str> {
    msg.get(key).and_then(|v| v.as_str()).filter(|s| !s.is_empty())
}

fn handle(msg: &Value, slot: &mut Vec<(PathBuf, Library)>, data: Option<&Path>) -> Result<Value, String> {
    let cmd = str_arg(msg, "cmd").unwrap_or("");
    match cmd {
        "hello" => {
            note_extension(msg);
            let root = library_root();
            Ok(json!({
                "app": "Archiver",
                "version": env!("CARGO_PKG_VERSION"),
                "library": root.as_ref().map(|r| r.to_string_lossy().into_owned()),
                "name": root.as_ref().and_then(|r| r.file_name()).map(|n| n.to_string_lossy().into_owned()),
                "libraries": library_list(),
            }))
        }
        "zones" => {
            let lib = library(slot, msg)?;
            scan_now_and_then(lib)?;
            // Reihenfolge und Hot Zones aus der Übersicht der App
            let (order, hot) = lib.zone_order();
            let mut zones = visible_zones(lib.zones()?);
            let rank = |p: &str| order.iter().position(|o| o == p).unwrap_or(order.len());
            zones.sort_by_key(|z| rank(&z.path));
            let hot: Vec<&String> = hot.iter().filter(|h| zones.iter().any(|z| &z.path == *h)).collect();
            let folders = zone_folders(lib, &zones);
            Ok(json!({
                "zones": zones,
                "hot": hot,
                "alpha": lib.look_alpha(),
                "radius": lib.look_radius(),
                "default": lib.default_zone(),
                "folders": folders,
                "libraries": library_list(),
            }))
        }
        "create_zone" => {
            let lib = library(slot, msg)?;
            let name = str_arg(msg, "name").ok_or("Name fehlt")?;
            Ok(json!({ "zone": lib.create_zone(name)? }))
        }
        "save" => {
            // Das Seitenarchiv gehört zur Bildanfrage: scheitert die, darf es nicht
            // als Waise im Zwischenordner liegen bleiben (0.31.0)
            let archive = str_arg(msg, "archive").map(|a| transfer_file(a, Some("mhtml")));
            let saved = library(slot, msg).and_then(|lib| {
                let file = transfer_file(str_arg(msg, "file").ok_or("Datei fehlt")?, None)?;
                save(lib, msg, &file, archive.as_ref().and_then(|a| a.as_ref().ok()))
            });
            if let Some(Ok(a)) = &archive {
                if a.exists() {
                    let _ = std::fs::remove_file(a);
                }
                remove_if_empty(a.parent());
            }
            let mut saved = saved?;
            if let Some(Err(e)) = archive {
                warn(&mut saved, "archive", &e);
            }
            Ok(saved)
        }
        // Safari: die Datei kam als Bytes über die HTTP-Brücke, sonst wie "save"
        // (ohne Seitenarchiv, Safari kennt kein pageCapture)
        "save_data" => {
            let file = data.ok_or("Daten fehlen")?;
            let lib = library(slot, msg)?;
            save(lib, msg, file, None)
        }
        "list" => {
            let lib = library(slot, msg)?;
            scan_now_and_then(lib)?;
            let fallback = lib.default_zone();
            let zone = str_arg(msg, "zone").unwrap_or(&fallback);
            let offset = msg.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let all: Vec<_> = lib.items_under(Some(zone).filter(|z| !z.is_empty()))?.into_iter().filter(|i| i.zone == zone).collect();
            let total = all.len();
            let items: Vec<Value> = all
                .into_iter()
                .skip(offset)
                .take(LIST_LIMIT)
                .map(|i| {
                    json!({
                        "path": i.path,
                        "hash": i.hash,
                        "filename": i.filename,
                        "imageUrl": i.image_url,
                        "pageUrl": i.page_url,
                        "pageTitle": i.page_title,
                        "ts": i.captured_at.unwrap_or(i.added_at),
                        "quote": i.quote.as_ref().map(|q| q.chars().take(400).collect::<String>()),
                        "tone": i.tone,
                    })
                })
                .collect();
            // Chrome nimmt vom Host höchstens 1 MB je Nachricht: lieber kürzen und
            // „next“ mitgeben, als dass die ganze Antwort verloren geht (0.31.0)
            let (items, next) = fit_reply(items, offset, total);
            Ok(json!({ "items": items, "next": next, "total": total }))
        }
        "preview" => {
            let lib = library(slot, msg)?;
            let (abs, hash) = lib.locate(str_arg(msg, "path").ok_or("Pfad fehlt")?)?;
            let (mime, bytes) = lib.small_preview(&hash, &abs).ok_or("Keine Vorschau")?;
            let data = base64::engine::general_purpose::STANDARD.encode(bytes);
            Ok(json!({ "hash": hash, "dataUrl": format!("data:{mime};base64,{data}") }))
        }
        "move" => {
            let lib = library(slot, msg)?;
            let paths: Vec<String> = msg
                .get("paths")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|p| p.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let zone = str_arg(msg, "zone").ok_or("Zone fehlt")?;
            Ok(json!({ "moved": lib.move_items(&paths, zone)? }))
        }
        "reveal" => {
            let lib = library(slot, msg)?;
            let (abs, _) = lib.locate(str_arg(msg, "path").ok_or("Pfad fehlt")?)?;
            reveal_in_finder(&abs);
            Ok(json!({}))
        }
        "open_app" => {
            open_app();
            Ok(json!({}))
        }
        other => Err(format!("Unbekannter Befehl: {other}")),
    }
}

/// So viele Einträge, wie unter das Limit passen; `next` ist der Versatz für die
/// nächste Seite oder null, wenn alles drin ist.
fn fit_reply(items: Vec<Value>, offset: usize, total: usize) -> (Vec<Value>, Option<usize>) {
    let mut size = 200; // Hülle der Antwort
    let mut out = Vec::new();
    for item in items {
        size += item.to_string().len() + 1;
        if size > MAX_OUT && !out.is_empty() {
            break;
        }
        out.push(item);
    }
    let end = offset + out.len();
    (out, (end < total).then_some(end))
}

/// Bild (oder Text-/Linkkarte) aus dem Zwischenordner übernehmen. `file` ist
/// schon geprüft: aus Downloads/Archery-Transfer (Chrome) oder aus dem
/// Zwischenordner der HTTP-Brücke (Safari).
pub(crate) fn save(lib: &Library, msg: &Value, file: &Path, archive: Option<&PathBuf>) -> Result<Value, String> {
    let fallback = lib.default_zone();
    let zone = str_arg(msg, "zone").unwrap_or(&fallback);
    crate::library::check_rel(zone)?;
    let source = Source {
        image_url: str_arg(msg, "imageUrl").filter(|u| u.starts_with("http")).map(String::from),
        page_url: str_arg(msg, "pageUrl").map(String::from),
        page_title: str_arg(msg, "pageTitle").map(String::from),
        captured_at: msg.get("capturedAt").and_then(|v| v.as_i64()).or_else(|| Some(now_ms())),
        quote: str_arg(msg, "quote").map(|q| q.chars().take(20_000).collect()),
    };
    let mut warnings: Vec<Value> = Vec::new();
    let (mut path, duplicate) = lib.adopt_file(file, str_arg(msg, "name"), zone, &source)?;
    let mut moved_here = false;
    remove_if_empty(file.parent());
    // Schon in der Bibliothek, aber auf eine andere Zone gezogen (0.30.1):
    // das vorhandene Bild wandert dorthin, statt still liegen zu bleiben.
    // Liegt es in einer externen Zone (Projektordner draußen), bleibt es dort:
    // Herausziehen würde Verknüpfungen in InDesign & Co. brechen (0.31.0).
    let external = lib.external_zones().iter().any(|(z, _, _)| path.starts_with(&format!("{z}/")));
    if duplicate && !external && str_arg(msg, "zone").is_some() && !path.starts_with(&format!("{zone}/")) {
        let _ = std::fs::create_dir_all(lib.root.join(zone));
        if let Some(moved) = lib.move_items(&[path.clone()], zone)?.pop() {
            path = moved;
            moved_here = true;
        }
    }
    // Textkarte: Farbe, die schon beim Markieren gezeigt wurde; Bildtext der Seite (0.29).
    // Mitgebrachter Bildtext gilt auch für ein schon vorhandenes Bild, die Farbe nicht.
    let tone = crate::studio::clean_tone(str_arg(msg, "tone")).filter(|_| !duplicate);
    let caption = str_arg(msg, "caption").map(|c| c.trim().to_string()).filter(|c| !c.is_empty());
    if tone.is_some() || caption.is_some() {
        if let Ok((_, hash)) = lib.locate(&path) {
            let patch = crate::studio::NotePatch { tone, caption, ..Default::default() };
            if let Err(e) = lib.set_note(&[hash], &patch) {
                warnings.push(json!({ "what": "caption", "error": e }));
            }
        }
    }
    // Linkkarte der offenen Seite: die ganze Seite als MHTML dahinter (0.30.1)
    if let Some(archive) = archive {
        if let Err(e) = lib.locate(&path).and_then(|(_, hash)| lib.attach_web_archive(&hash, archive)) {
            warnings.push(json!({ "what": "archive", "error": e }));
        }
    }
    // „duplicate“ heißt für die Extension: lag schon in dieser Zone, nichts verschoben.
    // „warnings“: Bild ist da, aber Bildtext oder Seitenarchiv fehlen (0.31.0)
    let mut out = json!({ "path": path, "duplicate": duplicate && !moved_here });
    if !warnings.is_empty() {
        out["warnings"] = json!(warnings);
    }
    Ok(out)
}

fn warn(out: &mut Value, what: &str, error: &str) {
    if !out["warnings"].is_array() {
        out["warnings"] = json!([]);
    }
    out["warnings"].as_array_mut().unwrap().push(json!({ "what": what, "error": error }));
}

/// Datei aus dem Zwischenordner der Extension (Downloads/Archery-Transfer, früher Archiver-Transfer).
/// Die Brücke übernimmt (und löscht) nur, was Chrome dort für uns abgelegt hat,
/// nie einen beliebigen Pfad auf dem Mac.
fn transfer_file(path: &str, ext: Option<&str>) -> Result<PathBuf, String> {
    let given = PathBuf::from(path);
    let meta = std::fs::symlink_metadata(&given).map_err(|e| format!("{}: {e}", given.display()))?;
    if !meta.file_type().is_file() {
        return Err(format!("Keine Datei: {}", given.display()));
    }
    let real = given.canonicalize().map_err(|e| e.to_string())?;
    let in_transfer = real
        .parent()
        .map(|p| p.ancestors().any(|a| crate::library::is_transfer_dir(a.file_name().and_then(|n| n.to_str()))))
        .unwrap_or(false);
    if !in_transfer {
        return Err(format!("Datei liegt nicht in {}: {}", crate::library::TRANSFER_DIR, given.display()));
    }
    if let Some(ext) = ext {
        if !real.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case(ext)) {
            return Err(format!("Erwartet .{ext}: {}", given.display()));
        }
    }
    Ok(real)
}

/// Den Zwischenordner der Extension in den Downloads wieder entfernen, wenn er leer ist.
fn remove_if_empty(dir: Option<&Path>) {
    if let Some(dir) = dir {
        if crate::library::is_transfer_dir(dir.file_name().and_then(|n| n.to_str())) {
            let _ = std::fs::remove_dir(dir); // schlägt fehl, solange noch etwas drin liegt
        }
    }
}

/// Die App selbst öffnen: die Brücke läuft aus Archiver.app/Contents/MacOS.
fn open_app() {
    let Ok(exe) = std::env::current_exe() else { return };
    if let Some(bundle) = exe.ancestors().find(|p| p.extension().and_then(|e| e.to_str()) == Some("app")) {
        let _ = std::process::Command::new("open").arg(bundle).spawn();
    }
}

fn reveal_in_finder(path: &Path) {
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg("-R").arg(path).spawn();
    #[cfg(not(target_os = "macos"))]
    let _ = std::process::Command::new("xdg-open").arg(path.parent().unwrap_or(path)).spawn();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_roundtrip() {
        let mut buf = Vec::new();
        write_message(&mut buf, &json!({ "cmd": "hello", "id": 1 })).unwrap();
        let back = read_message(&mut buf.as_slice()).unwrap().unwrap();
        assert_eq!(back["cmd"], "hello");
    }

    #[test]
    fn list_fits_into_one_chrome_message() {
        let items: Vec<Value> = (0..400).map(|i| json!({ "path": format!("Z/{i}.png"), "quote": "x".repeat(4000) })).collect();
        let (page, next) = fit_reply(items, 0, 400);
        assert!(page.len() < 400 && !page.is_empty());
        assert_eq!(next, Some(page.len()));
        assert!(serde_json::to_vec(&json!({ "items": page })).unwrap().len() < 1024 * 1024);
        let small: Vec<Value> = (0..3).map(|i| json!({ "path": i })).collect();
        assert_eq!(fit_reply(small, 10, 13).1, None, "letzte Seite");
    }

    #[test]
    fn folders_per_zone_without_items() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        for d in ["Typo/Serif/Alt", "Typo/Grotesk", "Farbe"] {
            std::fs::create_dir_all(root.join(d)).unwrap();
        }
        std::fs::write(root.join("Typo/a.txt"), "Zone mit Inhalt").unwrap();
        let lib = Library::open(&root).unwrap();
        lib.scan().unwrap();
        let zones = visible_zones(lib.zones().unwrap());
        let map = zone_folders(&lib, &zones);
        let typo: Vec<&str> = map["Typo"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
        assert_eq!(typo, ["Grotesk", "Serif", "Serif/Alt"], "auch leere Ordner, bis zwei Ebenen");
        assert!(map.get("Farbe").is_none());
        assert_eq!(lib.items_under(Some("Typo")).unwrap().len(), 1);
        assert!(lib.items_under(Some("Typ")).unwrap().is_empty(), "nur ganze Ordnernamen");
    }

    #[test]
    fn bad_message_does_not_end_the_bridge() {
        // kaputtes JSON: Fehler für diese Nachricht, die nächste kommt trotzdem an
        let mut buf = Vec::new();
        buf.extend_from_slice(&3u32.to_ne_bytes());
        buf.extend_from_slice(b"{x}");
        write_message(&mut buf, &json!({ "cmd": "weiter" })).unwrap();
        let mut input = buf.as_slice();
        assert!(read_message(&mut input).unwrap().is_err());
        assert_eq!(read_message(&mut input).unwrap().unwrap()["cmd"], "weiter");
        // unmögliche Länge: Ende statt 4 GB Speicher anzufordern
        let huge = u32::MAX.to_ne_bytes();
        assert!(read_message(&mut huge.as_slice()).is_none());
    }

    #[test]
    fn takes_only_files_from_transfer_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let transfer = tmp.path().join("Downloads").join(crate::library::TRANSFER_DIR);
        std::fs::create_dir_all(transfer.join("Zone")).unwrap();
        let ok = transfer.join("Zone/bild.jpg");
        std::fs::write(&ok, b"x").unwrap();
        let page = transfer.join("page-1.mhtml");
        std::fs::write(&page, b"x").unwrap();
        let outside = tmp.path().join("Vertrag.pdf");
        std::fs::write(&outside, b"x").unwrap();

        assert!(transfer_file(ok.to_str().unwrap(), None).is_ok());
        assert!(transfer_file(page.to_str().unwrap(), Some("mhtml")).is_ok());
        assert!(transfer_file(ok.to_str().unwrap(), Some("mhtml")).is_err());
        assert!(transfer_file(outside.to_str().unwrap(), None).is_err());
        // Umweg über .. oder einen Symlink im Transfer-Ordner zählt nicht
        let sneaky = format!("{}/../../Vertrag.pdf", transfer.display());
        assert!(transfer_file(&sneaky, None).is_err());
        #[cfg(unix)]
        {
            let link = transfer.join("link.pdf");
            std::os::unix::fs::symlink(&outside, &link).unwrap();
            assert!(transfer_file(link.to_str().unwrap(), None).is_err());
        }
        assert!(transfer_file(transfer.to_str().unwrap(), None).is_err());
        // Ältere Extensions (bis 3.31.1) legen noch unter Archiver-Transfer ab
        let old = tmp.path().join("Downloads").join(crate::library::TRANSFER_DIR_OLD);
        std::fs::create_dir_all(&old).unwrap();
        std::fs::write(old.join("alt.jpg"), b"x").unwrap();
        assert!(transfer_file(old.join("alt.jpg").to_str().unwrap(), None).is_ok());
    }

    #[test]
    fn hides_root_and_empty_parents() {
        let z = |p: &str, c| Zone { path: p.into(), name: p.into(), count: c, external: false };
        let out = visible_zones(vec![z("Inbox", 1), z("", 2), z("Kunden", 0), z("Kunden/A", 3), z("Leer", 0)]);
        let paths: Vec<_> = out.iter().map(|z| z.path.as_str()).collect();
        assert_eq!(paths, ["Inbox", "Kunden/A", "Leer"]);
    }
}

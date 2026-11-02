// lib.rs
// Tauri-Seite der App: Befehle für die Oberfläche, Einstellungen (welcher
// Bibliotheksordner), Dateisystem-Watcher und Vorschaubild-Worker.
//
// Ereignisse an die Oberfläche:
//   "library-changed"  Bestand hat sich geändert → Oberfläche lädt neu
//   "thumbs-ready"     [{ hash, thumb }] neue Vorschaubilder

pub mod atlas;
pub mod bridge;
pub mod http_bridge;
pub mod library;
pub mod log;
pub mod naming;
pub mod pdf;
pub mod studio;
mod system;
mod viewer;
#[cfg(target_os = "macos")]
mod macos;

use library::{ImportReport, Item, Library, Zone};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

type Watcher = notify_debouncer_full::Debouncer<notify::RecommendedWatcher, notify_debouncer_full::RecommendedCache>;

#[derive(Default)]
struct AppState {
    library: Mutex<Option<Arc<Library>>>,
    watcher: Mutex<Option<Watcher>>,
    thumbs: Mutex<Option<mpsc::Sender<()>>>,
    /// Bild, das gerade im Betrachter offen ist
    viewer: Mutex<Option<PathBuf>>,
}

#[derive(Serialize, Deserialize, Default)]
struct Settings {
    library: Option<String>,
    /// Name oder Kürzel für „abgelegt von“ und die geteilte Bibliothek
    #[serde(default, skip_serializing_if = "Option::is_none")]
    user: Option<String>,
    /// Vier feste Plätze für Bibliotheken (bis 0.35 drei), zwischen denen man umschaltet
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    libraries: Vec<LibrarySlot>,
    /// Palette per Kürzel und Zonenleiste beim Ziehen am ganzen Mac
    #[serde(default)]
    system: system::SystemPrefs,
    /// Einheitliche Dateinamen (0.36.0)
    #[serde(default)]
    naming: naming::Naming,
    /// Vorabversionen statt nur Beta-Updates (0.37.0). Nur auf Lukas' Mac an,
    /// geschaltet mit ⌥-Klick auf die Version unter ☰ › Information.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    prerelease: bool,
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct LibrarySlot {
    #[serde(default)]
    name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

const LIBRARY_SLOTS: usize = 4;

fn folder_name(path: &str) -> String {
    Path::new(path).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

/// Immer vier Plätze. Ältere Einstellungen: die bisherige Bibliothek wird Platz 1.
fn library_slots(s: &Settings) -> Vec<LibrarySlot> {
    let mut slots = s.libraries.clone();
    slots.resize(LIBRARY_SLOTS, LibrarySlot::default());
    slots.truncate(LIBRARY_SLOTS);
    if slots.iter().all(|l| l.path.is_none()) {
        if let Some(path) = &s.library {
            slots[0] = LibrarySlot { name: folder_name(path), path: Some(path.clone()) };
        }
    }
    slots
}

// ---- Schutz vor Ordnern, die keine Bibliothek sein können ----
// Archiver liest und beobachtet den ganzen Ordner. Der Benutzerordner, ein
// Laufwerk oder ein Systemordner würde den Start minutenlang blockieren.

const LIBRARY_MAX_FILES: usize = 20_000;
const LIBRARY_COUNT_SECS: u64 = 10;

/// Ebenen unterhalb von ~/Library/CloudStorage bzw. ~/Library/Mobile Documents,
/// None für alles andere. Beispiel: …/CloudStorage/GoogleDrive-x/Meine Ablage/Projekt = 3
fn cloud_depth(root: &Path) -> Option<usize> {
    let home = dirs::home_dir().and_then(|h| h.canonicalize().ok())?;
    for base in ["Library/CloudStorage", "Library/Mobile Documents"] {
        if let Ok(rest) = root.strip_prefix(home.join(base)) {
            return Some(rest.components().count());
        }
    }
    None
}

fn check_library(root: &Path) -> Result<(), String> {
    check_folder(root, true)
}

/// Wie check_library, zählt aber nur bei Ordnern, die noch keine Bibliothek
/// sind. Eine gewachsene Bibliothek (mit .archery) öffnet sich also auch über
/// 20.000 Dateien weiter; vorher landete man beim Start auf „Bibliothek anlegen“.
fn check_library_open(root: &Path) -> Result<(), String> {
    library::migrate_meta_dir(root);
    check_folder(root, !root.join(library::META_DIR).is_dir())
}

fn check_folder(root: &Path, count_files: bool) -> Result<(), String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    if !root.is_dir() {
        return Err("library-missing".into());
    }
    let mut blocked: Vec<PathBuf> = ["/", "/Users", "/Volumes", "/home", "/private", "/usr", "/opt", "/bin", "/etc"]
        .iter()
        .map(PathBuf::from)
        .collect();
    let mut system: Vec<PathBuf> = ["/System", "/Library", "/Applications"].iter().map(PathBuf::from).collect();
    if let Some(home) = dirs::home_dir().and_then(|h| h.canonicalize().ok()) {
        for d in ["Desktop", "Documents", "Downloads", "Pictures", "Movies", "Music", "Public"] {
            blocked.push(home.join(d));
        }
        system.push(home.join("Library"));
        system.push(home.join("Applications"));
        blocked.push(home);
    }
    // ein ganzes Laufwerk (/Volumes/Name) ist ebenfalls zu groß
    let is_volume = root.parent() == Some(Path::new("/Volumes"));
    // Cloud-Ordner (Google Drive, Dropbox, OneDrive, iCloud Drive) liegen unter
    // ~/Library, sind aber normale Ordner. Gesperrt bleibt nur ihre oberste Ebene
    // (das ganze Konto); die Menge prüft danach die Zählung.
    let cloud = cloud_depth(&root);
    if cloud.is_some_and(|d| d <= 1) {
        return Err("library-system".into());
    }
    if blocked.contains(&root) || is_volume || (cloud.is_none() && system.iter().any(|s| root.starts_with(s))) {
        return Err("library-system".into());
    }
    if !count_files {
        return Ok(());
    }
    // Vorab zählen: zu viele Dateien oder zu langsam → ablehnen.
    // Geteilte Bibliotheken liegen oft im Netz, dort zählt nur die Menge.
    let shared = studio::is_shared(&root);
    let deadline = std::time::Instant::now() + Duration::from_secs(LIBRARY_COUNT_SECS);
    let mut count = 0usize;
    let walk = walkdir::WalkDir::new(&root)
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| e.file_name() != library::META_DIR);
    for _ in walk.filter_map(|e| e.ok()) {
        count += 1;
        if count > LIBRARY_MAX_FILES {
            return Err("library-large".into());
        }
        if !shared && count % 256 == 0 && std::time::Instant::now() > deadline {
            return Err("library-slow".into());
        }
    }
    Ok(())
}

fn settings_path(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|d| d.join("settings.json"))
}

/// Einstellungen lesen und schreiben geht über diese Sperre: open_library
/// (Hintergrund) und set_system_prefs (Hauptthread) überschrieben sich sonst.
static SETTINGS_LOCK: Mutex<()> = Mutex::new(());

fn load_settings(app: &AppHandle) -> Settings {
    let Some(p) = settings_path(app) else { return Settings::default() };
    let Ok(text) = std::fs::read_to_string(&p) else { return Settings::default() };
    match serde_json::from_str::<Settings>(&text) {
        Ok(mut s) => {
            s.system.settle();
            s
        }
        Err(e) => {
            // Kaputte Datei nicht still überschreiben: beiseitelegen, dann Standard
            log::error("settings", format!("{}: {e}", p.display()));
            let _ = std::fs::copy(&p, p.with_extension("json.broken"));
            Settings::default()
        }
    }
}

fn save_settings(app: &AppHandle, s: &Settings) -> Result<(), String> {
    let p = settings_path(app).ok_or("Kein Einstellungsordner")?;
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    library::write_atomic(&p, &serde_json::to_vec_pretty(s).map_err(|e| e.to_string())?)
}

/// Einstellungen lesen, ändern, schreiben, ohne dass ein anderer Thread dazwischenkommt
fn update_settings<T>(app: &AppHandle, f: impl FnOnce(&mut Settings) -> T) -> Result<T, String> {
    let _guard = SETTINGS_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut s = load_settings(app);
    let out = f(&mut s);
    save_settings(app, &s)?;
    Ok(out)
}

fn current(state: &State<AppState>) -> Result<Arc<Library>, String> {
    state.library.lock().unwrap().clone().ok_or_else(|| "Keine Bibliothek geöffnet".into())
}

/// Dateiarbeit im Hintergrund-Pool: eine lange Kopie (anderes Laufwerk, großer
/// Ordner) belegt sonst einen der wenigen Befehls-Threads, und die Oberfläche
/// wartet auf alles andere mit (0.31.0)
async fn blocking<T: Send + 'static>(f: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|e| e.to_string())?
}

/// Bestand neu einlesen und bei Änderungen die Oberfläche und den Vorschau-Worker anstoßen.
fn rescan(app: &AppHandle, lib: &Library) {
    match lib.scan() {
        Ok(true) => {
            let _ = app.emit("library-changed", ());
        }
        Ok(false) => {}
        Err(e) => report(app, "scan", &e),
    }
    kick_thumbs(app);
}

/// Fehler ins Protokoll und als Hinweis an die Oberfläche (0.31.0)
/// Dieselbe Meldung höchstens einmal pro Minute, sonst meldet ein abgezogenes
/// Laufwerk bei jedem Einlesen erneut.
fn report(app: &AppHandle, area: &str, msg: &str) {
    static LAST: Mutex<Option<(String, std::time::Instant)>> = Mutex::new(None);
    log::error(area, msg);
    let mut last = LAST.lock().unwrap_or_else(|e| e.into_inner());
    if last.as_ref().is_some_and(|(m, at)| m == msg && at.elapsed() < Duration::from_secs(60)) {
        return;
    }
    *last = Some((msg.to_string(), std::time::Instant::now()));
    let _ = app.emit("library-error", msg.to_string());
}

fn kick_thumbs(app: &AppHandle) {
    if let Some(tx) = app.state::<AppState>().thumbs.lock().unwrap().as_ref() {
        let _ = tx.send(());
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ThumbReady {
    hash: String,
    thumb: String,
}

/// Ein Hintergrund-Thread rechnet fehlende Vorschaubilder, verteilt auf alle Kerne,
/// und meldet sie in kleinen Paketen an die Oberfläche.
fn start_thumb_worker(app: AppHandle, lib: Arc<Library>) -> mpsc::Sender<()> {
    let (tx, rx) = mpsc::channel::<()>();
    let analysis = start_analysis_worker(app.clone(), lib.clone());
    std::thread::spawn(move || {
        while rx.recv().is_ok() {
            let _ = analysis.send(());
            while rx.try_recv().is_ok() {}
            let Ok(todo) = lib.missing_thumbs() else { continue };
            let workers = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).clamp(1, 8);
            for batch in todo.chunks(workers * 3) {
                let ready = Mutex::new(Vec::new());
                std::thread::scope(|s| {
                    for part in batch.chunks(3) {
                        let (lib, ready) = (&lib, &ready);
                        s.spawn(move || {
                            for (hash, path) in part {
                                // Ein Decoder, der bei einer kaputten Datei abstürzt, darf
                                // nicht den ganzen Worker mitnehmen (0.31.0)
                                let made = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| lib.make_thumb(hash, path)));
                                match made {
                                    Ok(Some(thumb)) => ready.lock().unwrap_or_else(|e| e.into_inner()).push(ThumbReady { hash: hash.clone(), thumb }),
                                    Ok(None) => {}
                                    Err(_) => {
                                        log::error("thumbs", format!("Vorschau abgestürzt: {}", path.display()));
                                        lib.mark_thumb_failed(hash);
                                    }
                                }
                            }
                        });
                    }
                });
                let ready = ready.into_inner().unwrap();
                if !ready.is_empty() {
                    let _ = app.emit("thumbs-ready", ready);
                }
            }
            let _ = analysis.send(());
        }
    });
    tx
}

/// Zweiter Hintergrund-Thread: Farben, Bildinhalt und Text je Vorschaubild,
/// eins nach dem anderen (Vision ist rechenintensiv). Ergebnisse in Paketen.
fn start_analysis_worker(app: AppHandle, lib: Arc<Library>) -> mpsc::Sender<()> {
    let (tx, rx) = mpsc::channel::<()>();
    std::thread::spawn(move || {
        while rx.recv().is_ok() {
            while rx.try_recv().is_ok() {}
            let Ok(todo) = lib.missing_analysis() else { continue };
            let mut ready = Vec::new();
            for (hash, thumb) in todo {
                let colors = library::color_shares(&thumb).unwrap_or_default();
                #[cfg(target_os = "macos")]
                let (labels, text) = macos::analyze_image(&lib.original_for_hash(&hash).unwrap_or_else(|| thumb.clone()));
                #[cfg(not(target_os = "macos"))]
                let (labels, text): (Vec<String>, String) = (Vec::new(), String::new());
                let a = library::Analysis { hash, labels: labels.join(", "), text, colors };
                if lib.save_analysis(&a).is_ok() {
                    ready.push(a);
                }
                if ready.len() >= 12 {
                    let _ = app.emit("analysis-ready", std::mem::take(&mut ready));
                }
            }
            if !ready.is_empty() {
                let _ = app.emit("analysis-ready", ready);
            }
        }
    });
    tx
}

/// Externe Zonen (0.30.1) sind Verknüpfungen. Das Asset-Protokoll prüft den
/// aufgelösten Pfad, deshalb muss jedes Ziel draußen eigens freigegeben werden,
/// sonst laden Originale, Videos und PDFs daraus nicht (0.31.0).
fn allow_external(app: &AppHandle, lib: &Library) {
    for target in lib.external_targets() {
        let _ = app.asset_protocol_scope().allow_directory(&target, true);
    }
}

/// Öffnet eine Bibliothek, startet Watcher und Vorschau-Worker und liest den Bestand ein.
fn activate(app: &AppHandle, root: &Path) -> Result<(), String> {
    let lib = Arc::new(Library::open(root)?);
    lib.set_user(load_settings(app).user);
    // „Zuletzt gelöscht“: was älter als 7 Tage ist, geht in den Papierkorb des Macs
    let purge = lib.clone();
    std::thread::spawn(move || purge.purge_deleted(library::DELETED_DAYS));
    // Unsorted räumt sich nach 7 Tagen selbst auf (0.32): beim Öffnen und dann
    // stündlich, solange diese Bibliothek offen ist
    let inbox = Arc::downgrade(&lib);
    let inbox_app = app.clone();
    std::thread::spawn(move || {
        while let Some(lib) = inbox.upgrade() {
            if lib.expire_inbox(library::DELETED_DAYS).unwrap_or(0) > 0 {
                let _ = inbox_app.emit("library-changed", ());
            }
            drop(lib);
            std::thread::sleep(std::time::Duration::from_secs(3600));
        }
    });
    let state = app.state::<AppState>();
    *state.watcher.lock().unwrap() = None;

    app.asset_protocol_scope().allow_directory(root, true).map_err(|e| e.to_string())?;
    allow_external(app, &lib);

    let tx = start_thumb_worker(app.clone(), lib.clone());
    *state.thumbs.lock().unwrap() = Some(tx);
    *state.library.lock().unwrap() = Some(lib.clone());

    let watch_app = app.clone();
    let watch_lib = lib.clone();
    // Die Brücke zur Extension trägt neue Bilder selbst in die Datenbank ein.
    // Dann findet der Scan nichts Neues, die Oberfläche muss trotzdem neu laden.
    let last_sig = Mutex::new(lib.signature());
    let mut debouncer = notify_debouncer_full::new_debouncer(
        Duration::from_millis(400),
        None,
        move |res: notify_debouncer_full::DebounceEventResult| {
            let events = match res {
                Ok(events) => events,
                Err(errors) => {
                    for e in errors {
                        log::error("watcher", e);
                    }
                    return;
                }
            };
            let relevant = events.iter().any(|ev| {
                ev.paths.iter().any(|p| !p.components().any(|c| c.as_os_str() == library::META_DIR))
            });
            if relevant {
                let changed = log::check("scan", watch_lib.scan()).unwrap_or(false);
                let sig = watch_lib.signature();
                let mut last = last_sig.lock().unwrap();
                if changed || sig != *last {
                    let _ = watch_app.emit("library-changed", ());
                }
                *last = sig;
                kick_thumbs(&watch_app);
            }
        },
    )
    .map_err(|e| e.to_string())?;
    debouncer.watch(root, notify::RecursiveMode::Recursive).map_err(|e| e.to_string())?;
    // FSEvents folgt keinen Verknüpfungen: externe Zonen eigens beobachten
    for target in lib.external_targets() {
        let _ = debouncer.watch(&target, notify::RecursiveMode::Recursive);
    }
    *state.watcher.lock().unwrap() = Some(debouncer);

    // Geteilt: Änderungen der anderen kommen übers Netz oft ohne Ereignis an,
    // darum regelmäßig nachsehen, solange diese Bibliothek offen ist.
    if lib.shared {
        let poll_app = app.clone();
        let poll_lib = Arc::downgrade(&lib);
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(15));
            let Some(lib) = poll_lib.upgrade() else { break };
            let current = poll_app.state::<AppState>().library.lock().unwrap().clone();
            if !current.map(|c| Arc::ptr_eq(&c, &lib)).unwrap_or(false) {
                break;
            }
            let before = lib.signature();
            if lib.scan().unwrap_or(false) || lib.signature() != before {
                let _ = poll_app.emit("library-changed", ());
                kick_thumbs(&poll_app);
            }
        });
    }

    let scan_app = app.clone();
    std::thread::spawn(move || rescan(&scan_app, &lib));
    Ok(())
}

// ---- Befehle ----

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StartInfo {
    library: Option<String>,
    /// Bibliothek aus den Einstellungen, noch nicht geöffnet
    pending: Option<String>,
    suggested_library: String,
    extension_export: Option<String>,
}

#[tauri::command]
async fn start_info(app: AppHandle, state: State<'_, AppState>) -> Result<StartInfo, String> {
    let home = dirs::home_dir().unwrap_or_default();
    let export = dirs::download_dir().unwrap_or_else(|| home.join("Downloads")).join("archery-export.json");
    Ok(StartInfo {
        library: state.library.lock().unwrap().as_ref().map(|l| l.root.to_string_lossy().into_owned()),
        pending: load_settings(&app).library.filter(|p| Path::new(p).is_dir()),
        suggested_library: home.join("Archery").to_string_lossy().into_owned(),
        extension_export: export.is_file().then(|| export.to_string_lossy().into_owned()),
    })
}

#[tauri::command]
fn bridge_browsers() -> Vec<String> {
    bridge::connected_browsers()
}

#[tauri::command]
fn extension_status() -> serde_json::Value {
    bridge::extension_status()
}

#[tauri::command]
async fn pick_folder(app: AppHandle) -> Option<String> {
    app.dialog().file().blocking_pick_folder().and_then(|p| p.into_path().ok()).map(|p| p.to_string_lossy().into_owned())
}

#[tauri::command]
async fn pick_export_file(app: AppHandle) -> Option<String> {
    let mut dialog = app.dialog().file().add_filter("Archery Export", &["json"]);
    if let Some(dir) = dirs::download_dir() {
        dialog = dialog.set_directory(dir);
    }
    dialog.blocking_pick_file().and_then(|p| p.into_path().ok()).map(|p| p.to_string_lossy().into_owned())
}

#[tauri::command]
async fn open_library(app: AppHandle, path: String) -> Result<(), String> {
    let worker = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        check_library_open(Path::new(&path))?;
        activate(&worker, Path::new(&path))?;
        update_settings(&worker, |settings| {
            let mut slots = library_slots(settings);
            // Neue Bibliothek bekommt einen Platz: den ersten freien, sonst den bisher aktiven
            if !slots.iter().any(|l| l.path.as_deref() == Some(path.as_str())) {
                let i = slots
                    .iter()
                    .position(|l| l.path.is_none())
                    .or_else(|| slots.iter().position(|l| l.path.is_some() && l.path == settings.library))
                    .unwrap_or(0);
                slots[i] = LibrarySlot { name: folder_name(&path), path: Some(path.clone()) };
            }
            settings.libraries = slots;
            settings.library = Some(path.clone());
        })?;
        // Hauptfenster, Palette und Leiste ziehen nach (auch beim Wechsel aus der Palette)
        let _ = worker.emit("library-switched", path);
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LibrarySlotInfo {
    name: String,
    path: Option<String>,
    active: bool,
}

#[tauri::command]
fn libraries(app: AppHandle) -> Vec<LibrarySlotInfo> {
    let settings = load_settings(&app);
    library_slots(&settings)
        .into_iter()
        .map(|l| LibrarySlotInfo { active: l.path.is_some() && l.path == settings.library, name: l.name, path: l.path })
        .collect()
}

/// Platz umbenennen, neuen Ordner zuweisen (wird vorher geprüft) oder leeren.
#[tauri::command]
async fn set_library_slot(app: AppHandle, index: usize, name: Option<String>, path: Option<String>, clear: Option<bool>) -> Result<(), String> {
    if index >= LIBRARY_SLOTS {
        return Err("Unbekannter Platz".into());
    }
    if let Some(p) = path.clone() {
        tauri::async_runtime::spawn_blocking(move || check_library_open(Path::new(&p)))
            .await
            .map_err(|e| e.to_string())??;
    }
    update_settings(&app, |settings| {
        let mut slots = library_slots(settings);
        let slot = &mut slots[index];
        if clear.unwrap_or(false) {
            if slot.path.is_some() && slot.path == settings.library {
                return Err("library-active".to_string());
            }
            *slot = LibrarySlot::default();
        }
        if let Some(p) = path {
            if slot.name.trim().is_empty() || slot.path.as_deref().map(folder_name).as_deref() == Some(slot.name.as_str()) {
                slot.name = folder_name(&p);
            }
            slot.path = Some(p);
        }
        if let Some(n) = name {
            slot.name = n.trim().to_string();
        }
        settings.libraries = slots;
        Ok(())
    })?
}

// ---- Einheitliche Dateinamen (F10) und Namensvorschlag (F11), 0.36.0 ----

#[tauri::command]
fn naming_prefs(app: AppHandle) -> naming::Naming {
    load_settings(&app).naming
}

#[tauri::command]
fn set_naming_prefs(app: AppHandle, prefs: naming::Naming) -> Result<(), String> {
    update_settings(&app, |s| s.naming = prefs.clone())
}

/// Vorschau fürs Umbenennen-Blatt und den Vorschlag in der Info-Spalte. `scheme`
/// überschreibt das Schema aus den Einstellungen, ohne es zu speichern.
#[tauri::command]
async fn name_plan(app: AppHandle, state: State<'_, AppState>, paths: Vec<String>, scheme: Option<String>) -> Result<Vec<library::NamePlan>, String> {
    let lib = current(&state)?;
    let mut n = load_settings(&app).naming;
    if let Some(s) = scheme {
        n.preset = s;
    }
    let name = folder_name(&lib.root.to_string_lossy());
    tauri::async_runtime::spawn_blocking(move || lib.name_plan(&paths, &n, &name)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn rename_items(app: AppHandle, state: State<'_, AppState>, items: Vec<(String, String)>) -> Result<Vec<(String, String)>, String> {
    let lib = current(&state)?;
    let done = tauri::async_runtime::spawn_blocking(move || lib.rename_items(&items)).await.map_err(|e| e.to_string())??;
    if !done.is_empty() {
        let _ = app.emit("library-changed", ());
    }
    Ok(done)
}

/// Neue, leere Bibliothek (0.36.0): Ort und Name im Finder wählen, Ordner anlegen,
/// auf den Platz legen. Öffnen übernimmt danach open_library (legt Unsorted an).
#[tauri::command]
async fn create_library(app: AppHandle, index: usize, name: String) -> Result<Option<String>, String> {
    if index >= LIBRARY_SLOTS {
        return Err("Unbekannter Platz".into());
    }
    let mut dialog = app.dialog().file().set_file_name(name.trim());
    if let Some(dir) = dirs::home_dir() {
        dialog = dialog.set_directory(dir);
    }
    let Some(path) = dialog.blocking_save_file().and_then(|p| p.into_path().ok()) else { return Ok(None) };
    if path.exists() && !path.is_dir() {
        return Err("library-file".into());
    }
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    let p = path.to_string_lossy().into_owned();
    set_library_slot(app, index, None, Some(p.clone()), None).await?;
    Ok(Some(p))
}

// ---- Demo-Bibliothek für die Einstiegstour ----
// Eigener Ordner ~/Archery Demo/ (bis 0.43.2 ~/Archiver Demo, wird übernommen). Gefüllt mit Kopien aus der eigenen Bibliothek
// (vier Active Zones, Inbox, dazu Bilder für das Browser-Schema in .web/);
// ist die eigene Bibliothek noch zu leer, mit den mitgelieferten Grafiken
// (src-tauri/demo). Geöffnet wird sie, ohne die eigene Bibliothek in den
// Einstellungen zu ersetzen. .seed.json merkt sich, was nicht selbst gesammelt wurde.

const DEMO_BUNDLED: [(&str, &str); 5] = [
    ("typografie", "Typografie"),
    ("plakate", "Plakate"),
    ("verpackung", "Verpackung"),
    ("identitaet", "Identität"),
    ("inbox", library::INBOX),
];
const DEMO_PER_ZONE: usize = 8;
const DEMO_INBOX: usize = 6;
const DEMO_WEB: usize = 6;

fn demo_root() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    let (old, root) = (home.join("Archiver Demo"), home.join("Archery Demo"));
    if !root.exists() && old.is_dir() {
        let _ = std::fs::rename(&old, &root);
    }
    root
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DemoInfo {
    root: String,
    zones: Vec<String>,
    web: Vec<DemoWeb>,
}

/// Bild für das Browser-Schema: `file` liegt in .web/ (zum Ablegen),
/// `show` ist zum Anzeigen (Pfad in der eigenen Bibliothek oder Name der
/// mitgelieferten Grafik unter app/src/tour/).
#[derive(Serialize)]
struct DemoWeb {
    file: String,
    show: String,
}

/// Was in die Demo kommt: (Quelle, Zone, Dateiname)
fn demo_plan(app: &AppHandle, own: Option<&Library>) -> Result<(Vec<(PathBuf, String, String)>, Vec<PathBuf>), String> {
    let mut plan = Vec::new();
    let mut web = Vec::new();
    if let Some(lib) = own {
        let items: Vec<Item> = lib.items()?.into_iter().filter(|i| library::is_image(Path::new(&i.abs))).collect();
        let (order, hot) = lib.zone_order();
        let mut zones: Vec<String> = if hot.is_empty() { order } else { hot };
        zones.retain(|z| !z.is_empty() && z != library::INBOX && items.iter().any(|i| &i.zone == z));
        if zones.len() < 4 {
            let mut counts: Vec<(String, usize)> = Vec::new();
            for i in &items {
                if i.zone.is_empty() || i.zone == library::INBOX || zones.contains(&i.zone) {
                    continue;
                }
                match counts.iter_mut().find(|(z, _)| z == &i.zone) {
                    Some(c) => c.1 += 1,
                    None => counts.push((i.zone.clone(), 1)),
                }
            }
            counts.sort_by(|a, b| b.1.cmp(&a.1));
            zones.extend(counts.into_iter().map(|(z, _)| z).take(4 - zones.len()));
        }
        zones.truncate(4);
        if zones.len() >= 2 {
            let mut used = std::collections::HashSet::new();
            for z in &zones {
                let name = z.rsplit('/').next().unwrap_or(z).to_string();
                for i in items.iter().filter(|i| &i.zone == z).take(DEMO_PER_ZONE) {
                    used.insert(i.path.clone());
                    plan.push((PathBuf::from(&i.abs), name.clone(), i.filename.clone()));
                }
            }
            // Inbox: unsortiert, möglichst aus der eigenen Inbox, sonst aus dem Rest
            let rest: Vec<&Item> = items.iter().filter(|i| !used.contains(&i.path)).collect();
            let mut inbox: Vec<&Item> = rest.iter().copied().filter(|i| i.zone == library::INBOX).take(DEMO_INBOX).collect();
            for i in rest.iter().copied().filter(|i| i.zone != library::INBOX) {
                if inbox.len() >= DEMO_INBOX {
                    break;
                }
                inbox.push(i);
            }
            for i in &inbox {
                used.insert(i.path.clone());
                plan.push((PathBuf::from(&i.abs), library::INBOX.to_string(), i.filename.clone()));
            }
            web = items.iter().filter(|i| !used.contains(&i.path)).take(DEMO_WEB).map(|i| PathBuf::from(&i.abs)).collect();
            if plan.len() >= 12 {
                return Ok((plan, web));
            }
            plan.clear();
            web.clear();
        }
    }
    // Zu wenig Eigenes: mitgelieferte Grafiken
    let res = app.path().resource_dir().map_err(|e| e.to_string())?.join("demo");
    for (dir, zone) in DEMO_BUNDLED {
        let Ok(entries) = std::fs::read_dir(res.join(dir)) else { continue };
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if !name.starts_with('.') {
                plan.push((e.path(), zone.to_string(), name));
            }
        }
    }
    if let Ok(entries) = std::fs::read_dir(res.join("web")) {
        web = entries.flatten().map(|e| e.path()).collect();
        web.sort();
    }
    Ok((plan, web))
}

#[tauri::command]
async fn open_demo(app: AppHandle, state: State<'_, AppState>) -> Result<DemoInfo, String> {
    let own = state.library.lock().unwrap().clone();
    let (plan, web) = demo_plan(&app, own.as_deref())?;
    let root = demo_root();
    if root.exists() {
        // Reste einer früheren Demo
        let _ = library::to_trash(&root);
    }
    let mut seed = Vec::new();
    let mut zones: Vec<String> = Vec::new();
    for (src, zone, name) in plan {
        let dir = root.join(&zone);
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let dest = library::unique_path(&dir, &name);
        if std::fs::copy(&src, &dest).is_ok() {
            seed.push(format!("{}/{}", zone, dest.file_name().unwrap_or_default().to_string_lossy()));
            if !zones.contains(&zone) {
                zones.push(zone);
            }
        }
    }
    // Inbox ans Ende: die ersten vier sind die Active Zones
    if let Some(p) = zones.iter().position(|z| z == library::INBOX) {
        let inbox = zones.remove(p);
        zones.push(inbox);
    }
    let web_dir = root.join(".web");
    std::fs::create_dir_all(&web_dir).map_err(|e| e.to_string())?;
    let bundled = app.path().resource_dir().ok();
    let mut web_out = Vec::new();
    for src in web {
        let name = src.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let dest = library::unique_path(&web_dir, &name);
        if std::fs::copy(&src, &dest).is_ok() {
            let from_bundle = bundled.as_ref().is_some_and(|r| src.starts_with(r));
            web_out.push(DemoWeb {
                file: dest.to_string_lossy().into_owned(),
                show: if from_bundle { format!("tour/{name}") } else { src.to_string_lossy().into_owned() },
            });
        }
    }
    std::fs::write(root.join(".seed.json"), serde_json::to_vec(&seed).unwrap()).map_err(|e| e.to_string())?;
    activate(&app, &root)?;
    Ok(DemoInfo { root: root.to_string_lossy().into_owned(), zones, web: web_out })
}

/// Zurück zur eigenen Bibliothek. Was in der Demo selbst gesammelt wurde,
/// wandert in deren Inbox; ohne `keep` geht der Demo-Ordner in den Papierkorb.
#[tauri::command]
async fn close_demo(app: AppHandle, keep: bool) -> Result<usize, String> {
    let root = demo_root();
    let own = load_settings(&app).library.map(PathBuf::from);
    let seed: std::collections::HashSet<String> = std::fs::read(root.join(".seed.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    let mut moved = 0;
    if let Some(own) = own.as_ref().filter(|o| o.is_dir() && root.is_dir()) {
        let inbox = own.join(library::INBOX);
        std::fs::create_dir_all(&inbox).map_err(|e| e.to_string())?;
        for e in walkdir::WalkDir::new(&root)
            .min_depth(1)
            .into_iter()
            .filter_entry(|e| !e.file_name().to_string_lossy().starts_with('.'))
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let rel = e.path().strip_prefix(&root).unwrap_or(e.path()).to_string_lossy().replace('\\', "/");
            if seed.contains(&rel) {
                continue;
            }
            let name = e.file_name().to_string_lossy().into_owned();
            let dest = library::unique_path(&inbox, &name);
            library::move_file(e.path(), &dest)?;
            moved += 1;
        }
    }
    *app.state::<AppState>().watcher.lock().unwrap() = None;
    if !keep && root.is_dir() {
        library::to_trash(&root).map_err(|e| e.to_string())?;
    }
    if let Some(own) = own.filter(|o| o.is_dir()) {
        activate(&app, &own)?;
    }
    Ok(moved)
}

/// Tour, Kapitel „Sammeln im Browser“: Ein Bild aus dem Schema landet wirklich in der Demo-Zone.
/// Es zählt als selbst gesammelt.
#[tauri::command]
async fn demo_collect(file: String, zone: String) -> Result<(), String> {
    let root = demo_root();
    let src = PathBuf::from(&file);
    if !src.starts_with(root.join(".web")) || file.contains("..") || library::check_rel(&zone).is_err() {
        return Err("Ungültiger Pfad".into());
    }
    let name = src.file_name().ok_or("Ungültiger Name")?.to_string_lossy().into_owned();
    let dir = root.join(&zone);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::copy(&src, library::unique_path(&dir, &name)).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn set_fullscreen(window: tauri::WebviewWindow, on: bool) -> Result<(), String> {
    window.set_fullscreen(on).map_err(|e| e.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    root: String,
    name: String,
    zones: Vec<Zone>,
    items: Vec<Item>,
    /// Eigene Reihenfolge je Zone (Hashes), per Ziehen in der Zone gesetzt
    order: std::collections::HashMap<String, Vec<String>>,
    shared: bool,
    user: Option<String>,
    /// Zone für alles ohne Ziel (Standard: Inbox)
    default_zone: String,
    /// Zonentiefe und übersprungene Ordner
    structure: library::Structure,
    /// Alle Ordner unterhalb der Zonen, auch leere (0.27)
    folders: Vec<String>,
    /// Anzahl unter „Zuletzt gelöscht“
    deleted: usize,
    /// Reihenfolge der Zonen dieser Bibliothek (.archiver/zones.json), 0.31.0
    zone_order: Vec<String>,
    /// Seit wann ein Objekt in Unsorted liegt (Hash → ms), 0.33
    inbox_seen: std::collections::HashMap<String, i64>,
    /// Zone, die sich nach 7 Tagen leert; None = aus (0.33.1)
    expire_zone: Option<String>,
}

#[tauri::command]
async fn set_structure(app: AppHandle, state: State<'_, AppState>, structure: library::Structure) -> Result<(), String> {
    let lib = current(&state)?;
    tauri::async_runtime::spawn_blocking(move || lib.set_structure(&structure)).await.map_err(|e| e.to_string())??;
    let _ = app.emit("library-changed", ());
    kick_thumbs(&app);
    Ok(())
}

#[tauri::command]
async fn set_default_zone(app: AppHandle, state: State<'_, AppState>, zone: String) -> Result<(), String> {
    current(&state)?.set_default_zone(&zone)?;
    let _ = app.emit("library-changed", ());
    Ok(())
}

#[tauri::command]
async fn set_expire_zone(app: AppHandle, state: State<'_, AppState>, zone: String) -> Result<(), String> {
    current(&state)?.set_expire_zone(&zone)?;
    let _ = app.emit("library-changed", ());
    Ok(())
}

#[tauri::command]
async fn snapshot(state: State<'_, AppState>) -> Result<Snapshot, String> {
    let lib = current(&state)?;
    // Liest alle Objekte und läuft durch die Ordner: nicht auf den Befehls-Threads
    blocking(move || {
        let user = lib.user.lock().unwrap().clone();
        Ok(Snapshot {
            root: lib.root.to_string_lossy().into_owned(),
            name: lib.root.file_name().and_then(|n| n.to_str()).unwrap_or("Archery").to_string(),
            zones: lib.zones()?,
            items: lib.items()?,
            order: lib.item_order(),
            shared: lib.shared,
            user,
            default_zone: lib.default_zone(),
            structure: lib.structure(),
            folders: lib.folders(),
            deleted: lib.deleted_count(),
            zone_order: lib.zone_order().0,
            inbox_seen: lib.inbox_seen(),
            expire_zone: lib.expire_zone(),
        })
    })
    .await
}

// ---- Notiz, Rechte, geteilte Bibliothek, Reihenfolge, Paket, Moodboard (studio.rs) ----

#[tauri::command]
async fn set_note(app: AppHandle, state: State<'_, AppState>, hashes: Vec<String>, patch: studio::NotePatch) -> Result<(), String> {
    let lib = current(&state)?;
    tauri::async_runtime::spawn_blocking(move || lib.set_note(&hashes, &patch)).await.map_err(|e| e.to_string())??;
    let _ = app.emit("library-changed", ());
    Ok(())
}

#[tauri::command]
async fn set_quote(app: AppHandle, state: State<'_, AppState>, hash: String, quote: String) -> Result<(), String> {
    let lib = current(&state)?;
    tauri::async_runtime::spawn_blocking(move || lib.set_quote(&hash, &quote)).await.map_err(|e| e.to_string())??;
    let _ = app.emit("library-changed", ());
    Ok(())
}

#[tauri::command]
async fn set_item_order(state: State<'_, AppState>, zone: String, hashes: Vec<String>) -> Result<(), String> {
    current(&state)?.set_item_order(&zone, &hashes)
}

/// Name merken und die Bibliothek als geteilt markieren (oder zurück).
/// Wechselt der Modus, öffnet die App die Bibliothek neu: die Datenbank zieht
/// dabei vom Bibliotheksordner auf den Mac (geteilt) bzw. zurück.
#[tauri::command]
async fn set_sharing(app: AppHandle, state: State<'_, AppState>, shared: bool, user: Option<String>) -> Result<(), String> {
    let user = user.map(|u| u.trim().to_string()).filter(|u| !u.is_empty());
    update_settings(&app, |s| s.user = user.clone())?;
    let lib = current(&state)?;
    lib.set_user(user);
    if lib.shared == shared {
        return Ok(());
    }
    let root = lib.root.clone();
    // Aktuellen Stand sichern, damit die neue Datenbank ihn wiederfindet
    lib.write_metadata_to(&lib.metadata_path())?;
    if shared {
        lib.write_metadata_to(&lib.person_file())?;
    }
    studio::set_shared(&root, shared)?;
    drop(lib);
    activate(&app, &root)?;
    let _ = app.emit("library-changed", ());
    Ok(())
}

/// Zone oder Auswahl als Paket: Ordner wählen, Dateien, Quellen.csv/.md und
/// Kontaktbogen hineinlegen, dann im Finder zeigen.
#[tauri::command]
async fn export_package(
    app: AppHandle,
    state: State<'_, AppState>,
    paths: Vec<String>,
    title: String,
    meta: String,
    sheet: bool,
) -> Result<Option<studio::PackageReport>, String> {
    let lib = current(&state)?;
    let mut dialog = app.dialog().file();
    if let Some(dir) = dirs::desktop_dir() {
        dialog = dialog.set_directory(dir);
    }
    let Some(dest) = dialog.blocking_pick_folder().and_then(|p| p.into_path().ok()) else {
        return Ok(None);
    };
    let report = tauri::async_runtime::spawn_blocking(move || lib.export_package(&paths, &title, &meta, &dest, sheet))
        .await
        .map_err(|e| e.to_string())??;
    let _ = app.opener().reveal_item_in_dir(&report.path);
    Ok(Some(report))
}

#[tauri::command]
async fn export_moodboard(app: AppHandle, title: String, meta: String, file_name: String, board: pdf::Board) -> Result<Option<String>, String> {
    let mut dialog = app.dialog().file().add_filter("PDF", &["pdf"]).set_file_name(&file_name);
    if let Some(dir) = dirs::download_dir() {
        dialog = dialog.set_directory(dir);
    }
    let Some(path) = dialog.blocking_save_file().and_then(|p| p.into_path().ok()) else {
        return Ok(None);
    };
    let target = path.clone();
    tauri::async_runtime::spawn_blocking(move || std::fs::write(&target, pdf::moodboard(&title, &meta, &board)))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;
    let _ = app.opener().open_path(path.to_string_lossy(), None::<&str>);
    Ok(Some(path.to_string_lossy().into_owned()))
}

/// Atlas als Bild (0.42.0): PNG der ganzen Anordnung, Sichern-Dialog wie beim Moodboard
#[tauri::command]
async fn export_cloud_image(app: AppHandle, file_name: String, image: atlas::Sheet) -> Result<Option<String>, String> {
    let mut dialog = app.dialog().file().add_filter("PNG", &["png"]).set_file_name(&file_name);
    if let Some(dir) = dirs::download_dir() {
        dialog = dialog.set_directory(dir);
    }
    let Some(path) = dialog.blocking_save_file().and_then(|p| p.into_path().ok()) else {
        return Ok(None);
    };
    let target = path.clone();
    tauri::async_runtime::spawn_blocking(move || atlas::write_png(&image, &target))
        .await
        .map_err(|e| e.to_string())??;
    let _ = app.opener().reveal_item_in_dir(&path);
    Ok(Some(path.to_string_lossy().into_owned()))
}

#[tauri::command]
async fn move_items(app: AppHandle, state: State<'_, AppState>, paths: Vec<String>, zone: String, exact: Option<bool>) -> Result<Vec<String>, String> {
    let lib = current(&state)?;
    let moved = blocking(move || lib.move_items_to(&paths, &zone, exact.unwrap_or(false))).await?;
    let _ = app.emit("library-changed", ());
    Ok(moved)
}

#[tauri::command]
async fn trash_items(app: AppHandle, state: State<'_, AppState>, paths: Vec<String>) -> Result<Vec<(String, String)>, String> {
    let lib = current(&state)?;
    let n = blocking(move || lib.trash_items(&paths)).await?;
    let _ = app.emit("library-changed", ());
    Ok(n)
}

#[tauri::command]
async fn create_folder(app: AppHandle, state: State<'_, AppState>, parent: String, name: String) -> Result<String, String> {
    let f = current(&state)?.create_folder(&parent, &name)?;
    let _ = app.emit("library-changed", ());
    Ok(f)
}

/// Notiz-Objekte (0.29)
#[tauri::command]
async fn create_note(app: AppHandle, state: State<'_, AppState>, dir: String, title: String) -> Result<String, String> {
    let p = current(&state)?.create_note(&dir, &title)?;
    let _ = app.emit("library-changed", ());
    Ok(p)
}

#[tauri::command]
async fn read_note(state: State<'_, AppState>, path: String) -> Result<String, String> {
    current(&state)?.read_note(&path)
}

#[derive(serde::Serialize)]
struct NoteSaved {
    path: String,
    hash: String,
}

#[tauri::command]
async fn write_note(app: AppHandle, state: State<'_, AppState>, path: String, text: String, rename: bool) -> Result<NoteSaved, String> {
    let (path, hash) = current(&state)?.write_note(&path, &text, rename)?;
    if rename {
        let _ = app.emit("library-changed", ());
    }
    Ok(NoteSaved { path, hash })
}

#[tauri::command]
async fn deleted_items(state: State<'_, AppState>) -> Result<Vec<library::DeletedItem>, String> {
    let lib = current(&state)?;
    tauri::async_runtime::spawn_blocking(move || lib.deleted_items()).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn restore_deleted(app: AppHandle, state: State<'_, AppState>, files: Vec<String>) -> Result<usize, String> {
    let lib = current(&state)?;
    let n = blocking(move || lib.restore_deleted(&files)).await?;
    let _ = app.emit("library-changed", ());
    kick_thumbs(&app);
    Ok(n)
}

#[tauri::command]
async fn remove_deleted(app: AppHandle, state: State<'_, AppState>, files: Vec<String>) -> Result<usize, String> {
    let lib = current(&state)?;
    let n = blocking(move || lib.remove_deleted(&files)).await?;
    let _ = app.emit("library-changed", ());
    Ok(n)
}

#[tauri::command]
async fn create_zone(app: AppHandle, state: State<'_, AppState>, name: String) -> Result<String, String> {
    let z = current(&state)?.create_zone(&name)?;
    let _ = app.emit("library-changed", ());
    Ok(z)
}

#[tauri::command]
async fn rename_zone(app: AppHandle, state: State<'_, AppState>, zone: String, name: String) -> Result<String, String> {
    let lib = current(&state)?;
    let z = blocking(move || lib.rename_zone(&zone, &name)).await?;
    let _ = app.emit("library-changed", ());
    Ok(z)
}

#[tauri::command]
async fn move_folder(app: AppHandle, state: State<'_, AppState>, from: String, into: String) -> Result<String, String> {
    let lib = current(&state)?;
    let p = blocking(move || lib.move_folder(&from, &into)).await?;
    let _ = app.emit("library-changed", ());
    Ok(p)
}

/// Deckkraft der Zonen beim Ziehen (App und Extension)
#[tauri::command]
async fn set_look(state: State<'_, AppState>, radius: String, alpha: Option<f64>) -> Result<(), String> {
    current(&state)?.set_look(&radius, alpha)
}

/// Reihenfolge der Zonen und die vier Hot Zones (für die Extension)
#[tauri::command]
async fn set_zone_order(state: State<'_, AppState>, order: Vec<String>, hot: Vec<String>) -> Result<(), String> {
    current(&state)?.set_zone_order(&order, &hot)
}

#[tauri::command]
async fn restore_items(app: AppHandle, state: State<'_, AppState>, items: Vec<(String, String)>) -> Result<usize, String> {
    let lib = current(&state)?;
    let n = blocking(move || lib.restore_items(&items)).await?;
    let _ = app.emit("library-changed", ());
    kick_thumbs(&app);
    Ok(n)
}

#[tauri::command]
async fn zone_other_files(state: State<'_, AppState>, zone: String) -> Result<usize, String> {
    let lib = current(&state)?;
    blocking(move || Ok(lib.zone_other_files(&zone))).await
}

#[tauri::command]
async fn trash_zone(app: AppHandle, state: State<'_, AppState>, zone: String) -> Result<(), String> {
    let lib = current(&state)?;
    blocking(move || lib.trash_zone(&zone)).await?;
    let _ = app.emit("library-changed", ());
    Ok(())
}

#[tauri::command]
async fn reveal_zone(app: AppHandle, state: State<'_, AppState>, zone: String) -> Result<(), String> {
    let lib = current(&state)?;
    library::check_rel(&zone)?;
    let dir = if zone.is_empty() { lib.root.clone() } else { lib.root.join(&zone) };
    app.opener().open_path(dir.to_string_lossy(), None::<&str>).map_err(|e| e.to_string())
}

fn after_import(app: &AppHandle, report: ImportReport) -> ImportReport {
    let _ = app.emit("library-changed", ());
    kick_thumbs(app);
    report
}

#[tauri::command]
async fn import_extension_export(app: AppHandle, state: State<'_, AppState>, path: String) -> Result<ImportReport, String> {
    let lib = current(&state)?;
    let report = tauri::async_runtime::spawn_blocking(move || lib.import_extension_export(Path::new(&path)))
        .await
        .map_err(|e| e.to_string())??;
    Ok(after_import(&app, report))
}

#[tauri::command]
async fn import_folder(app: AppHandle, state: State<'_, AppState>, path: String) -> Result<ImportReport, String> {
    let lib = current(&state)?;
    let report = tauri::async_runtime::spawn_blocking(move || lib.import_folder(Path::new(&path)))
        .await
        .map_err(|e| e.to_string())??;
    Ok(after_import(&app, report))
}

#[tauri::command]
async fn import_files(app: AppHandle, state: State<'_, AppState>, paths: Vec<String>, zone: String) -> Result<ImportReport, String> {
    let lib = current(&state)?;
    let files: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let report = tauri::async_runtime::spawn_blocking(move || lib.import_files(&files, &zone))
        .await
        .map_err(|e| e.to_string())??;
    Ok(after_import(&app, report))
}

/// ⌘V (0.32): Zwischenablage in eine Zone (Dateien, Bilddaten oder Text).
/// Gelesen wird auf dem Hauptthread (AppKit), importiert im Hintergrund.
#[tauri::command]
async fn paste_into(app: AppHandle, state: State<'_, AppState>, zone: String) -> Result<ImportReport, String> {
    let lib = current(&state)?;
    #[cfg(target_os = "macos")]
    let pasted = {
        let (tx, rx) = mpsc::channel();
        app.run_on_main_thread(move || {
            let _ = tx.send(macos::read_pasteboard());
        })
        .map_err(|e| e.to_string())?;
        let (files, image, text) = rx.recv().map_err(|e| e.to_string())?;
        library::Pasted { files: files.into_iter().map(PathBuf::from).collect(), image, text }
    };
    #[cfg(not(target_os = "macos"))]
    let pasted = library::Pasted::default();
    let report = tauri::async_runtime::spawn_blocking(move || lib.paste_into(&zone, pasted))
        .await
        .map_err(|e| e.to_string())??;
    Ok(after_import(&app, report))
}

#[tauri::command]
async fn drag_icon(state: State<'_, AppState>, hash: String, path: String, color: Option<String>) -> Result<Option<String>, String> {
    let lib = current(&state)?;
    Ok(match color {
        Some(c) => lib.color_icon(&c),
        None => lib.drag_icon(&hash, Path::new(&path)),
    })
}

#[tauri::command]
async fn reveal(app: AppHandle, path: String) -> Result<(), String> {
    app.opener().reveal_item_in_dir(path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_url(app: AppHandle, url: String) -> Result<(), String> {
    if !(url.starts_with("https://") || url.starts_with("http://") || url.starts_with("mailto:")) {
        return Err("Nur Web- und Mail-Adressen".into());
    }
    app.opener().open_url(url, None::<&str>).map_err(|e| e.to_string())
}

/// Plakat, Index und Paar: Felder wie im PDF-Ersteller gesetzt.
#[tauri::command]
async fn export_flow(app: AppHandle, file_name: String, flow: pdf::Flow) -> Result<Option<String>, String> {
    let mut dialog = app.dialog().file().add_filter("PDF", &["pdf"]).set_file_name(&file_name);
    if let Some(dir) = dirs::download_dir() {
        dialog = dialog.set_directory(dir);
    }
    let Some(path) = dialog.blocking_save_file().and_then(|p| p.into_path().ok()) else {
        return Ok(None);
    };
    let target = path.clone();
    tauri::async_runtime::spawn_blocking(move || std::fs::write(&target, pdf::flow_sheet(&flow)))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;
    let _ = app.opener().open_path(path.to_string_lossy(), None::<&str>);
    Ok(Some(path.to_string_lossy().into_owned()))
}

/// Kontaktbogen: fragt nach dem Speicherort, schreibt das PDF und öffnet es.
/// Gibt den Pfad zurück, oder None, wenn der Dialog abgebrochen wurde.
#[tauri::command]
async fn export_pdf(
    app: AppHandle,
    title: String,
    meta: String,
    file_name: String,
    items: Vec<pdf::SheetItem>,
    layout: Option<pdf::Layout>,
) -> Result<Option<String>, String> {
    let layout = layout.unwrap_or_else(|| pdf::Layout::auto(items.len()));
    let mut dialog = app.dialog().file().add_filter("PDF", &["pdf"]).set_file_name(&file_name);
    if let Some(dir) = dirs::download_dir() {
        dialog = dialog.set_directory(dir);
    }
    let Some(path) = dialog.blocking_save_file().and_then(|p| p.into_path().ok()) else {
        return Ok(None);
    };
    let target = path.clone();
    tauri::async_runtime::spawn_blocking(move || std::fs::write(&target, pdf::contact_sheet(&title, &meta, &items, layout)))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;
    let _ = app.opener().open_path(path.to_string_lossy(), None::<&str>);
    Ok(Some(path.to_string_lossy().into_owned()))
}

/// Externe Zonen (0.30.1): Ordner außerhalb der Bibliothek, als Zone verknüpft
#[derive(Serialize)]
struct ExternalZone {
    zone: String,
    path: String,
    online: bool,
}

#[tauri::command]
async fn external_zones(state: State<'_, AppState>) -> Result<Vec<ExternalZone>, String> {
    Ok(current(&state)?.external_zones().into_iter().map(|(zone, path, online)| ExternalZone { zone, path, online }).collect())
}

#[tauri::command]
async fn link_folder(app: AppHandle, state: State<'_, AppState>, path: String) -> Result<String, String> {
    let lib = current(&state)?;
    let worker = lib.clone();
    let zone = tauri::async_runtime::spawn_blocking(move || {
        // Dieselben Schranken wie für eine Bibliothek: kein Benutzer-, System-
        // oder Laufwerksordner, nicht mehr als 20.000 Dateien (0.31.0)
        let folder = std::path::Path::new(&path);
        check_library(folder).map_err(|e| e.replacen("library-", "zone-", 1))?;
        worker.link_folder(folder)
    })
    .await
    .map_err(|e| e.to_string())??;
    allow_external(&app, &lib);
    if let Some(debouncer) = state.watcher.lock().unwrap().as_mut() {
        for target in lib.external_targets() {
            let _ = debouncer.watch(&target, notify::RecursiveMode::Recursive);
        }
    }
    let _ = app.emit("library-changed", ());
    Ok(zone)
}

#[tauri::command]
async fn unlink_folder(app: AppHandle, state: State<'_, AppState>, zone: String) -> Result<(), String> {
    current(&state)?.unlink_folder(&zone)?;
    let _ = app.emit("library-changed", ());
    Ok(())
}

/// Gesicherte Webseite einer Linkkarte (0.30.1), sonst None
#[tauri::command]
async fn web_archive(state: State<'_, AppState>, hash: String) -> Result<Option<String>, String> {
    Ok(current(&state)?.web_archive(&hash).map(|p| p.to_string_lossy().into_owned()))
}

/// MHTML öffnet Chrome; ohne Chrome das Standardprogramm
#[tauri::command]
async fn open_web_archive(app: AppHandle, state: State<'_, AppState>, hash: String) -> Result<(), String> {
    let path = current(&state)?.web_archive(&hash).ok_or("Keine gesicherte Seite")?;
    let path = path.to_string_lossy().into_owned();
    if app.opener().open_path(path.clone(), Some("Google Chrome")).is_err() {
        app.opener().open_path(path, None::<&str>).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn open_file(app: AppHandle, state: State<'_, AppState>, path: String) -> Result<(), String> {
    // Nur Objekte der Bibliothek öffnen, nie einen beliebigen Pfad oder ein Programm (0.31.0)
    let lib = current(&state)?;
    let rel = PathBuf::from(&path)
        .strip_prefix(&lib.root)
        .map(|r| r.to_string_lossy().replace('\\', "/"))
        .map_err(|_| format!("Nicht in der Bibliothek: {path}"))?;
    let (abs, _) = lib.locate(&rel)?;
    if !library::is_media(&abs) || !abs.is_file() {
        return Err(format!("Kein Objekt: {path}"));
    }
    app.opener().open_path(abs.to_string_lossy(), None::<&str>).map_err(|e| e.to_string())
}

// ---- Updates ----
// Die App fragt beim Start im öffentlichen Repo Archiver-Releases nach
// latest.json. Gibt es eine neuere Version, zeigt die Oberfläche einen Hinweis;
// „Installieren“ lädt das signierte Paket, ersetzt die App und startet neu.

#[derive(Serialize)]
struct UpdateInfo {
    version: String,
    notes: Option<String>,
}

// ---- Am ganzen Mac: Palette und Zonenleiste (system.rs) ----

/// Fehler aus der Oberfläche ins Protokoll (~/Library/Logs/Archery), 0.31.0
#[tauri::command]
fn log_error(area: String, msg: String) {
    let area: String = area.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-').take(24).collect();
    log::error(&format!("ui-{area}"), msg.chars().take(2000).collect::<String>());
}

#[tauri::command]
fn system_prefs(app: AppHandle) -> system::SystemPrefs {
    load_settings(&app).system
}

#[tauri::command]
fn set_system_prefs(app: AppHandle, prefs: system::SystemPrefs) -> Result<(), String> {
    update_settings(&app, |s| s.system = prefs.clone())?;
    system::apply(&app, &prefs)
}

/// Hell / Dunkel aus den Einstellungen: Fensterrahmen und Glas passend setzen.
/// Dunkel bleibt das HUD-Glas, Hell nimmt das helle Seitenleisten-Glas.
/// Ecke des Trays in Punkt, aus Einstellungen › Aussehen (0.28)
static TRAY_RADIUS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(12);

#[tauri::command]
fn set_appearance(window: tauri::WebviewWindow, light: bool, radius: Option<f64>) {
    let _ = window.set_theme(Some(if light { tauri::Theme::Light } else { tauri::Theme::Dark }));
    if let Some(r) = radius.filter(|_| window.label() == system::PALETTE) {
        TRAY_RADIUS.store(r.clamp(0.0, 24.0) as u32, std::sync::atomic::Ordering::Relaxed);
    }
    if window.label() == "main" || window.label() == system::PALETTE || window.label() == viewer::LABEL {
        use tauri::window::{Effect, EffectState, EffectsBuilder};
        let effect = if light { Effect::Sidebar } else { Effect::HudWindow };
        let radius = if window.label() == system::PALETTE { TRAY_RADIUS.load(std::sync::atomic::Ordering::Relaxed) as f64 } else { 0.0 };
        let mut fx = EffectsBuilder::new().effect(effect).state(EffectState::Active);
        if radius > 0.0 {
            fx = fx.radius(radius);
        }
        let _ = window.set_effects(fx.build());
    }
}

/// Vorabversionen: eigener Kanal im selben Release-Repository (Tag „vorab“,
/// kein „latest“). Die Beta-Tester sehen ihn nie; was dort geprüft ist, gibt
/// „freigeben“ im Build-Workflow unverändert an alle frei.
const PRERELEASE_FEED: &str = "https://github.com/lr-practice/Archiver-Releases/releases/download/vorab/vorab.json";

/// Erst der Vorab-Kanal (falls an), dann der Beta-Kanal. Ist die Beta schon
/// weiter als die letzte Vorabversion, gewinnt die Beta.
async fn find_update(app: &AppHandle) -> Result<Option<tauri_plugin_updater::Update>, String> {
    use tauri_plugin_updater::UpdaterExt;
    if load_settings(app).prerelease {
        let url = tauri::Url::parse(PRERELEASE_FEED).map_err(|e| e.to_string())?;
        if let Ok(updater) = app.updater_builder().endpoints(vec![url]).and_then(|b| b.build()) {
            match updater.check().await {
                Ok(Some(update)) => return Ok(Some(update)),
                Ok(None) => {}
                Err(e) => log::warn("update", format!("Vorab-Kanal: {e}")),
            }
        }
    }
    app.updater().map_err(|e| e.to_string())?.check().await.map_err(|e| e.to_string())
}

#[tauri::command]
fn update_channel(app: AppHandle) -> bool {
    load_settings(&app).prerelease
}

#[tauri::command]
fn set_update_channel(app: AppHandle, prerelease: bool) -> Result<(), String> {
    update_settings(&app, |s| s.prerelease = prerelease)
}

#[tauri::command]
async fn update_check(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    let update = find_update(&app).await?;
    Ok(update.map(|u| UpdateInfo { version: u.version, notes: u.body }))
}

#[tauri::command]
async fn update_install(app: AppHandle) -> Result<(), String> {
    let Some(update) = find_update(&app).await? else {
        return Ok(());
    };
    update.download_and_install(|_, _| {}, || {}).await.map_err(|e| e.to_string())?;
    app.restart();
}

// ---- Version wählen (0.38.1) ----
// Jede Beta-Freigabe liegt in Archiver-Releases unter vX.Y.Z mit latest.json,
// jede Vorabversion ab 0.38.0 zusätzlich unter vorab-X.Y.Z mit vorab.json.
// Die Liste kommt über curl (in macOS enthalten) von der GitHub-API; die
// Installation läuft wie ein Update, nur mit fester Quelle und auch rückwärts.

const RELEASES_API: &str = "https://api.github.com/repos/lr-practice/Archiver-Releases/releases?per_page=60";
const RELEASES_DL: &str = "https://github.com/lr-practice/Archiver-Releases/releases/download";

#[derive(Serialize, Debug, PartialEq)]
struct VersionEntry {
    version: String,
    tag: String,
    date: String,
    prerelease: bool,
}

/// Versionen aus der Antwort der API, neueste zuerst. Vorabversionen nur, wenn
/// der Vorab-Kanal an ist; gibt es eine Version auch als Beta, zählt die Beta.
fn parse_versions(json: &str, with_pre: bool) -> Vec<VersionEntry> {
    let Ok(serde_json::Value::Array(list)) = serde_json::from_str::<serde_json::Value>(json) else {
        return Vec::new();
    };
    let mut out: Vec<VersionEntry> = Vec::new();
    for r in &list {
        let tag = r["tag_name"].as_str().unwrap_or("");
        let has = |name: &str| r["assets"].as_array().is_some_and(|a| a.iter().any(|x| x["name"] == name));
        let (version, prerelease) = if let Some(v) = tag.strip_prefix("vorab-") {
            if !with_pre || !has("vorab.json") {
                continue;
            }
            (v, true)
        } else if let Some(v) = tag.strip_prefix('v') {
            if !has("latest.json") {
                continue;
            }
            (v, false)
        } else {
            continue;
        };
        if semver::Version::parse(version).is_err() {
            continue;
        }
        let date = r["published_at"].as_str().unwrap_or("").chars().take(10).collect();
        out.push(VersionEntry { version: version.into(), tag: tag.into(), date, prerelease });
    }
    out.sort_by(|a, b| {
        let (va, vb) = (semver::Version::parse(&a.version).unwrap(), semver::Version::parse(&b.version).unwrap());
        vb.cmp(&va).then(a.prerelease.cmp(&b.prerelease))
    });
    out.dedup_by(|b, a| a.version == b.version);
    out
}

#[tauri::command]
async fn update_versions(app: AppHandle) -> Result<Vec<VersionEntry>, String> {
    let pre = load_settings(&app).prerelease;
    let body = blocking(|| {
        let out = std::process::Command::new("/usr/bin/curl")
            .args(["-fsSL", "--max-time", "15", "-H", "Accept: application/vnd.github+json", RELEASES_API])
            .output()
            .map_err(|e| e.to_string())?;
        if !out.status.success() {
            return Err("offline".into());
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    })
    .await?;
    Ok(parse_versions(&body, pre))
}

#[tauri::command]
async fn update_install_version(app: AppHandle, tag: String) -> Result<(), String> {
    use tauri_plugin_updater::UpdaterExt;
    let feed = if tag.starts_with("vorab-") { "vorab.json" } else { "latest.json" };
    if tag.is_empty() || !tag.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-') {
        return Err("bad tag".into());
    }
    let url = tauri::Url::parse(&format!("{RELEASES_DL}/{tag}/{feed}")).map_err(|e| e.to_string())?;
    let updater = app
        .updater_builder()
        .endpoints(vec![url])
        .and_then(|b| b.version_comparator(|current, release| release.version != current).build())
        .map_err(|e| e.to_string())?;
    let Some(update) = updater.check().await.map_err(|e| e.to_string())? else {
        return Err("installed".into());
    };
    update.download_and_install(|_, _| {}, || {}).await.map_err(|e| e.to_string())?;
    app.restart();
}

// ---- Startton ----
// Standard ist der eingebettete Archiver-Ton (von Lukas, sounds/standard.wav). Eine eigene Datei liegt als Kopie in
// <Konfiguration>/startton/<Originalname>.

const STANDARD_WAV: &[u8] = include_bytes!("../sounds/standard.wav");
const SOUND_EXTS: &[&str] = &["wav", "aif", "aiff", "mp3", "m4a", "caf"];

fn sound_dir(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|d| d.join("startton"))
}

fn custom_sound(app: &AppHandle) -> Option<PathBuf> {
    std::fs::read_dir(sound_dir(app)?)
        .ok()?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .find(|p| p.is_file() && p.extension().and_then(|e| e.to_str()).is_some_and(|e| SOUND_EXTS.contains(&e.to_lowercase().as_str())))
}

fn standard_sound(app: &AppHandle) -> Option<PathBuf> {
    let dir = app.path().app_config_dir().ok()?;
    let path = dir.join("standard.wav");
    if std::fs::metadata(&path).map(|m| m.len() as usize).ok() != Some(STANDARD_WAV.len()) {
        std::fs::create_dir_all(&dir).ok()?;
        std::fs::write(&path, STANDARD_WAV).ok()?;
    }
    Some(path)
}

/// Name der eigenen Datei, None = Standard
#[tauri::command]
fn startup_sound_name(app: AppHandle) -> Option<String> {
    custom_sound(&app).and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
}

#[tauri::command]
async fn play_startup_sound(app: AppHandle, volume: f32) -> Result<(), String> {
    let path = custom_sound(&app).or_else(|| standard_sound(&app)).ok_or("Kein Startton")?;
    #[cfg(target_os = "macos")]
    {
        let path = path.to_string_lossy().into_owned();
        app.run_on_main_thread(move || {
            macos::play_sound(&path, volume);
        })
        .map_err(|e| e.to_string())?;
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (path, volume);
    Ok(())
}

/// Eigene Datei wählen: wird kopiert, die alte ersetzt. Gibt den Namen zurück.
#[tauri::command]
async fn pick_startup_sound(app: AppHandle) -> Result<Option<String>, String> {
    let picked = app
        .dialog()
        .file()
        .add_filter("Audio", SOUND_EXTS)
        .blocking_pick_file()
        .and_then(|p| p.into_path().ok());
    let Some(src) = picked else { return Ok(None) };
    if std::fs::metadata(&src).map(|m| m.len()).unwrap_or(0) > 10 * 1024 * 1024 {
        return Err("Die Datei ist größer als 10 MB.".into());
    }
    let dir = sound_dir(&app).ok_or("Kein Konfigurationsordner")?;
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let name = src.file_name().ok_or("Ungültige Datei")?.to_owned();
    std::fs::copy(&src, dir.join(&name)).map_err(|e| e.to_string())?;
    Ok(Some(name.to_string_lossy().into_owned()))
}

/// Zurück auf den Standard
#[tauri::command]
fn reset_startup_sound(app: AppHandle) -> Result<(), String> {
    if let Some(dir) = sound_dir(&app) {
        let _ = std::fs::remove_dir_all(dir);
    }
    Ok(())
}

/// ⌘C: Bilder in die Zwischenablage (AppKit, auf dem Hauptthread).
#[tauri::command]
async fn copy_images(app: AppHandle, paths: Vec<String>, text: Option<String>) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let (tx, rx) = mpsc::channel();
        app.run_on_main_thread(move || {
            let ok = match (&text, paths.as_slice()) {
                (Some(text), [path]) => macos::copy_text_image(path, text),
                _ => macos::copy_images(&paths),
            };
            let _ = tx.send(ok);
        })
        .map_err(|e| e.to_string())?;
        match rx.recv() {
            Ok(true) => Ok(()),
            _ => Err("Kopieren fehlgeschlagen".into()),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, paths, text);
        Err("Nur auf dem Mac".into())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_drag::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(system::shortcut_plugin())
        .manage(AppState::default())
        .setup(|app| {
            let handle = app.handle().clone();
            // Brücke zur Chrome-Extension anmelden (bei jedem Start, falls die App verschoben wurde).
            std::thread::spawn(bridge::install_manifests);
            // Brücke zur Safari-Extension: HTTP auf 127.0.0.1 (http_bridge.rs).
            // Nur in der laufenden App; der Chrome-Host startet ohne setup.
            #[cfg(target_os = "macos")]
            if http_bridge::ENABLED {
                http_bridge::start();
            }
            // Die Bibliothek öffnet die Oberfläche selbst (open_library), damit das
            // Fenster sofort erscheint, auch wenn das Einlesen dauert.
            if let Some(path) = viewer::from_args() {
                viewer::open(&handle, path);
            }
            system::setup(&handle, &load_settings(&handle).system);
            Ok(())
        })
        .on_window_event(|window, event| {
            // Das Hauptfenster wird beim Schließen nur ausgeblendet: Archiver läuft
            // weiter für Palette und Zonenleiste (system.rs), der Betrachter kann es
            // mit „In Archiver zeigen“ hervorholen, das Dock-Symbol ebenso. ⌘Q beendet.
            match event {
                tauri::WindowEvent::CloseRequested { api, .. }
                    if window.label() == "main" && (cfg!(target_os = "macos") || window.app_handle().get_webview_window(viewer::LABEL).is_some()) =>
                {
                    api.prevent_close();
                    let _ = window.hide();
                }
                // Observer zu: ein dafür weggelegtes Hauptfenster kommt hinten zurück
                #[cfg(target_os = "macos")]
                tauri::WindowEvent::Destroyed if window.label() == viewer::LABEL => {
                    if system::MAIN_TUCKED.swap(false, std::sync::atomic::Ordering::Relaxed) {
                        if let Some(Ok(main)) = window.app_handle().get_webview_window("main").map(|w| w.ns_window()) {
                            unsafe { macos::order_back(main.cast()) };
                        }
                    }
                }
                tauri::WindowEvent::Focused(false) if window.label() == system::PALETTE => {
                    system::palette_blurred(window.app_handle());
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            start_info,
            bridge_browsers,
            extension_status,
            pick_folder,
            pick_export_file,
            open_library,
            libraries,
            set_library_slot,
            create_library,
            naming_prefs,
            set_naming_prefs,
            name_plan,
            rename_items,
            open_demo,
            close_demo,
            demo_collect,
            set_fullscreen,
            snapshot,
            move_items,
            trash_items,
            restore_items,
            create_zone,
            create_folder,
            create_note,
            read_note,
            write_note,
            deleted_items,
            restore_deleted,
            remove_deleted,
            rename_zone,
            move_folder,
            set_zone_order,
            set_default_zone,
            set_expire_zone,
            set_structure,
            set_look,
            trash_zone,
            zone_other_files,
            reveal_zone,
            import_extension_export,
            import_folder,
            import_files,
            drag_icon,
            reveal,
            open_url,
            open_file,
            export_pdf,
            export_moodboard,
            export_cloud_image,
            export_flow,
            export_package,
            set_note,
            set_quote,
            paste_into,
            web_archive,
            external_zones,
            link_folder,
            unlink_folder,
            open_web_archive,
            set_item_order,
            set_sharing,
            copy_images,
            startup_sound_name,
            play_startup_sound,
            pick_startup_sound,
            reset_startup_sound,
            set_appearance,
            update_check,
            update_channel,
            set_update_channel,
            update_install,
            update_versions,
            update_install_version,
            viewer::viewer_state,
            viewer::viewer_siblings,
            viewer::viewer_zones,
            viewer::viewer_file,
            viewer::viewer_drag_icon,
            viewer::viewer_lookup,
            viewer::viewer_save,
            viewer::show_in_app,
            viewer::viewer_defaults,
            viewer::viewer_make_default,
            system::palette_hide,
            system::dropbar_hold,
            system::dropbar_preview,
            system::dropbar_glass,
            system::dropbar_fade,
            log_error,
            system::palette_ghost,
            system::palette_place,
            system_prefs,
            set_system_prefs,
        ])
        .build(tauri::generate_context!())
        .expect("Archery konnte nicht starten")
        .run(|app, event| match event {
            // Bild aus dem Finder („Öffnen mit“ oder Doppelklick)
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            tauri::RunEvent::Opened { urls } => {
                for url in urls {
                    if let Ok(path) = url.to_file_path() {
                        viewer::open(app, path);
                    }
                }
            }
            // Das Hauptfenster startet unsichtbar: wurde die App mit einem Bild
            // geöffnet, erscheint nur der Betrachter, sonst wie gewohnt die App.
            tauri::RunEvent::Ready => {
                // Mit einem Bild gestartet (Finder): nur der Betrachter, nie das Hauptfenster
                #[cfg(target_os = "macos")]
                if macos::launched_with_documents() {
                    return;
                }
                let app = app.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(450));
                    if app.state::<AppState>().viewer.lock().unwrap().is_none() {
                        viewer::show_main(&app);
                    }
                });
            }
            // Klick aufs Dock-Symbol ohne offenes Fenster
            #[cfg(target_os = "macos")]
            // (0.37.1: auch, wenn nur Betrachter oder Tray offen sind und das
            // Hauptfenster fehlt)
            tauri::RunEvent::Reopen { has_visible_windows, .. } => {
                let main_shown = app.get_webview_window("main").and_then(|w| w.is_visible().ok()).unwrap_or(false);
                // 0.40.25: gehört zum Öffnen eines Bildes im Observer, kein Klick aufs Dock
                let opening = library::now_ms() as u64 <= viewer::OPENED_AT.load(std::sync::atomic::Ordering::Relaxed) + 1500;
                if (!has_visible_windows || !main_shown) && !opening {
                    viewer::show_main(app);
                }
            }
            _ => {}
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_from_releases() {
        let json = r#"[
          {"tag_name":"vorab","published_at":"2026-10-16T08:55:00Z","assets":[{"name":"vorab.json"}]},
          {"tag_name":"vorab-0.38.0","published_at":"2026-10-16T08:55:00Z","assets":[{"name":"vorab.json"}]},
          {"tag_name":"vorab-0.37.3","published_at":"2026-10-15T15:44:00Z","assets":[{"name":"vorab.json"}]},
          {"tag_name":"v0.37.3","published_at":"2026-10-16T08:50:00Z","assets":[{"name":"latest.json"}]},
          {"tag_name":"v0.9.0","published_at":"2026-09-01T10:00:00Z","assets":[{"name":"latest.json"}]},
          {"tag_name":"v0.37.0","published_at":"2026-10-14T19:22:00Z","assets":[{"name":"latest.json"}]},
          {"tag_name":"v0.36.0","published_at":"2026-10-10T10:00:00Z","assets":[{"name":"x.dmg"}]},
          {"tag_name":"app-latest","published_at":"2026-10-16T08:55:00Z","assets":[]}
        ]"#;
        let tags = |pre| parse_versions(json, pre).into_iter().map(|v| v.tag).collect::<Vec<_>>();
        assert_eq!(tags(false), ["v0.37.3", "v0.37.0", "v0.9.0"]);
        assert_eq!(tags(true), ["vorab-0.38.0", "v0.37.3", "v0.37.0", "v0.9.0"]);
        assert_eq!(parse_versions(json, true)[0].date, "2026-10-16");
        assert!(parse_versions("{\"message\":\"rate limit\"}", true).is_empty());
    }

    #[test]
    fn library_guard() {
        let home = dirs::home_dir().unwrap();
        assert_eq!(check_library(&home).unwrap_err(), "library-system");
        assert_eq!(check_library(Path::new("/")).unwrap_err(), "library-system");
        let dir = std::env::temp_dir().join(format!("archiver-guard-{}", std::process::id()));
        std::fs::create_dir_all(dir.join(library::META_DIR)).unwrap();
        for i in 0..20 {
            std::fs::write(dir.join(format!("{i}.txt")), "x").unwrap();
        }
        assert!(check_library(&dir).is_ok());
        let big = dir.join("big");
        std::fs::create_dir_all(&big).unwrap();
        for i in 0..=LIBRARY_MAX_FILES {
            std::fs::write(big.join(format!("{i}")), "").unwrap();
        }
        assert_eq!(check_library(&dir).unwrap_err(), "library-large");
        // Schon eine Bibliothek: öffnet sich trotz Menge, ein neuer Ordner nicht
        assert!(check_library_open(&dir).is_ok());
        std::fs::remove_dir_all(dir.join(library::META_DIR)).unwrap();
        assert_eq!(check_library_open(&dir).unwrap_err(), "library-large");
        assert_eq!(check_library_open(&home).unwrap_err(), "library-system");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn cloud_folders_allowed() {
        let home = dirs::home_dir().unwrap().canonicalize().unwrap();
        let drive = home.join("Library/CloudStorage/GoogleDrive-test/Meine Ablage");
        assert_eq!(cloud_depth(&drive), Some(2));
        assert_eq!(cloud_depth(&drive.join("Referenzen")), Some(3));
        assert_eq!(cloud_depth(&home.join("Library/Caches")), None);
    }

    #[test]
    fn slots_from_old_settings() {
        let s = Settings { library: Some("/x/Referenzen".into()), ..Default::default() };
        let slots = library_slots(&s);
        assert_eq!(slots.len(), 4, "vier Plätze seit 0.36.0");
        assert_eq!(slots[0].name, "Referenzen");
        assert!(slots[1].path.is_none());
    }
}

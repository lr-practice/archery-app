// library.rs
// Die Bibliothek: ein normaler Ordner auf der Festplatte. Jeder Unterordner ist
// eine Zone, jede Bilddatei ein Eintrag. Die Datenbank in .archiver/library.db
// ist nur Cache plus Metadaten (Quellen). Fehlt sie, entsteht sie beim nächsten
// Scan neu; die Quellen stehen zusätzlich in .archiver/metadata.json.
//
// Identität per Content-Hash: Quellen hängen am Hash, nicht am Pfad. Wird eine
// Datei im Finder umbenannt oder verschoben, findet der Scan sie über den Hash
// wieder und die Quelle bleibt erhalten.

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

pub const META_DIR: &str = ".archery";
/// Datenordner bis 0.43 (.archiver); wird beim Öffnen umbenannt.
const LEGACY_META_DIR: &str = ".archiver";

/// Benennt den alten Datenordner .archiver in .archery um (0.44.0), wenn es
/// den neuen noch nicht gibt. Quellen, Zonen-Reihenfolge und Vorschauen bleiben so erhalten.
pub fn migrate_meta_dir(root: &Path) {
    let (old, new) = (root.join(LEGACY_META_DIR), root.join(META_DIR));
    if old.is_dir() && !new.exists() {
        let _ = fs::rename(old, new);
    }
}
pub const INBOX: &str = "Unsorted";
/// Ordner für Bildschirmfotos des Macs (0.47.0, Einstellungen › Bibliothek)
pub const SHOTS: &str = "Screenshots";
/// Frühere Standard-Zone (bis 0.23); wird beim Öffnen zu „Unsorted“ umbenannt.
pub const LEGACY_INBOX: &str = "Inbox";
/// Zwischenordner der Extension in den Downloads (wird nach der Übernahme geleert).
pub const TRANSFER_DIR: &str = "Archery-Transfer";
/// Name bis Extension 3.31.1; ältere Extensions legen noch dort ab.
pub const TRANSFER_DIR_OLD: &str = "Archiver-Transfer";

pub fn is_transfer_dir(name: Option<&str>) -> bool {
    matches!(name, Some(TRANSFER_DIR) | Some(TRANSFER_DIR_OLD))
}
const THUMB_SIZE: u32 = 720;
const DRAG_ICON_SIZE: u32 = 96;
const SMALL_THUMB_SIZE: u32 = 400; // Galerie der Extension
const RAW_PREVIEW_MAX_BYTES: u64 = 700 * 1024; // SVG & Co. gehen unverändert durch

const IMAGE_EXTS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "webp", "avif", "bmp", "tif", "tiff", "svg", "heic", "heif",
];
// Formate, aus denen wir selbst Vorschaubilder rechnen können. Für die übrigen
// (SVG, AVIF, HEIC) zeigt die Oberfläche direkt das Original.
const THUMB_EXTS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "bmp", "tif", "tiff"];
/// PDF und Video: Vorschau (Seite 1 bzw. ein Standbild) über Quick Look,
/// Seitenzahl, Dauer und Maße über PDFKit bzw. AVFoundation (macos.rs).
pub const PDF_EXTS: &[&str] = &["pdf"];
pub const VIDEO_EXTS: &[&str] = &["mp4", "m4v", "mov", "webm"];
/// Audio: Vorschau ist das eingebettete Cover oder eine Wellenform,
/// Dauer, Titel und Format kommen aus AVFoundation (macos.rs).
pub const AUDIO_EXTS: &[&str] = &["wav", "mp3", "aif", "aiff", "m4a", "flac"];
/// Textkarten liegen als reine Textdatei in der Zone; die Oberfläche zeigt
/// sie als farbige Karte (Text aus der Quelle oder aus der Datei selbst).
pub const TEXT_EXTS: &[&str] = &["txt", "md"];
/// Notiz-Objekte (0.29): eigene Texte als Markdown in der Zone
pub const NOTE_EXTS: &[&str] = &["md"];
const TEXT_MAX_CHARS: usize = 20_000;

pub type Result<T> = std::result::Result<T, String>;

pub(crate) fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Zone {
    pub path: String,
    pub name: String,
    pub count: usize,
    /// Liegt in einem verknüpften Ordner außerhalb der Bibliothek (0.39.1):
    /// lässt sich nur trennen, nie löschen
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub external: bool,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub path: String,
    pub abs: String,
    pub zone: String,
    /// Ordner innerhalb der Zone (erste Ebene darunter), leer für lose Dateien
    pub folder: String,
    pub filename: String,
    pub hash: String,
    pub bytes: i64,
    pub mtime: i64,
    pub added_at: i64,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub format: String,
    pub thumb: Option<String>,
    pub image_url: Option<String>,
    pub page_url: Option<String>,
    pub page_title: Option<String>,
    pub captured_at: Option<i64>,
    /// Bildinhalt (Apple Vision), Leerzeichen-getrennt; leer, solange nicht analysiert.
    pub labels: Option<String>,
    /// Erkannter Text im Bild.
    pub text: Option<String>,
    /// Farbanteile, z. B. "red:0.42 black:0.30".
    pub colors: Option<String>,
    /// PDF: Anzahl der Seiten
    pub pages: Option<i64>,
    /// Video: Dauer in Sekunden
    pub duration: Option<f64>,
    /// Zitat-Karte: der gesicherte Text
    pub quote: Option<String>,
    /// Audio: Titel, Künstler, Album aus den Metadaten
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    /// Audio: Abtastrate in Hz, Bittiefe (nur unkomprimiert), Kanäle
    pub sample_rate: Option<i64>,
    pub bit_depth: Option<i64>,
    pub channels: Option<i64>,
    /// Notiz zum Objekt (das Warum einer Referenz)
    pub note: Option<String>,
    /// Rechte: Urheber, Lizenz, nur intern verwenden
    pub credit: Option<String>,
    pub license: Option<String>,
    pub internal: bool,
    /// Wer das Objekt abgelegt hat (geteilte Bibliothek)
    pub added_by: Option<String>,
    /// Farbe der Textkarte, falls gewählt
    pub tone: Option<String>,
    /// Bildtext: Credits oder Unterschrift (0.29)
    pub caption: Option<String>,
    /// Notiz-Objekt: verknüpfte Objekte (Hashes, 0.29)
    pub links: Option<Vec<String>>,
    /// Farbmarke in der Übersicht (0.41.0)
    pub mark: Option<String>,
}

/// Seitenzahl, Dauer und Maße eines PDFs oder Videos, bei Audio dazu Titel und Format.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MediaInfo {
    pub pages: Option<i64>,
    pub duration: Option<f64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub sample_rate: Option<i64>,
    pub bit_depth: Option<i64>,
    pub channels: Option<i64>,
}

/// Ergebnis der Bildanalyse für ein Bild.
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Analysis {
    pub hash: String,
    pub labels: String,
    pub text: String,
    pub colors: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub image_url: Option<String>,
    pub page_url: Option<String>,
    pub page_title: Option<String>,
    pub captured_at: Option<i64>,
    /// Zitat-Karten: der markierte Text, damit er sich kopieren lässt
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote: Option<String>,
}

/// Zwischenablage für ⌘V (macos.rs read_pasteboard liest sie aus)
#[derive(Default)]
pub struct Pasted {
    pub files: Vec<PathBuf>,
    pub image: Option<Vec<u8>>,
    pub text: Option<String>,
}

/// Name für Eingefügtes: Text nach seinen ersten Wörtern, sonst „Eingefügt“ mit Datum
fn paste_name(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().take(6).collect();
    let mut name = clean_segment(&words.join(" "));
    if name.chars().count() > 48 {
        name = name.chars().take(48).collect::<String>().trim_end().to_string();
    }
    if !name.is_empty() {
        return name;
    }
    let days = now_ms().div_euclid(86_400_000);
    // Tage seit 1970 → Datum (Howard Hinnant, civil_from_days), UTC
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("Eingefügt {y}-{m:02}-{d:02}")
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NamePlan {
    pub path: String,
    pub hash: String,
    pub old: String,
    pub new: String,
    pub suggestion: Option<String>,
    pub messy: bool,
}

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub imported: usize,
    pub duplicates: usize,
    pub missing: usize,
    pub with_source: usize,
}

/// In den Papierkorb des Macs, ohne Finder: Der Finder fragt bei Ordnern in
/// geteilten iCloud-Ordnern jedes Mal nach („Möchtest du diesen geteilten
/// Ordner wirklich bewegen?“) und blockiert damit den Start.
pub fn to_trash(p: &Path) -> std::result::Result<(), trash::Error> {
    #[allow(unused_mut)]
    let mut ctx = trash::TrashContext::default();
    #[cfg(target_os = "macos")]
    {
        use trash::macos::{DeleteMethod, TrashContextExtMacos};
        ctx.set_delete_method(DeleteMethod::NsFileManager);
    }
    ctx.delete(p)
}


pub struct Library {
    pub root: PathBuf,
    pub(crate) conn: Mutex<Connection>,
    scan_lock: Mutex<()>,
    failed_thumbs: Mutex<HashSet<String>>,
    /// Geteilte Bibliothek (Server, NAS, Dropbox): Datenbank liegt lokal,
    /// Angaben gehen über eine Datei pro Person in .archiver/people/.
    pub shared: bool,
    /// Name oder Kürzel dieser Person; wird bei neuen Objekten als „abgelegt von“ gemerkt.
    pub(crate) user: Mutex<Option<String>>,
    /// Stand der Personen-Dateien beim letzten Abgleich
    pub(crate) people_sig: Mutex<u64>,
}

pub fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

fn time_ms(t: std::io::Result<SystemTime>) -> Option<i64> {
    t.ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_millis() as i64)
}

fn ext_of(path: &Path) -> String {
    path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase()
}

pub fn is_image(path: &Path) -> bool {
    IMAGE_EXTS.contains(&ext_of(path).as_str())
}

pub fn is_pdf(path: &Path) -> bool {
    PDF_EXTS.contains(&ext_of(path).as_str())
}

pub fn is_video(path: &Path) -> bool {
    VIDEO_EXTS.contains(&ext_of(path).as_str())
}

pub fn is_audio(path: &Path) -> bool {
    AUDIO_EXTS.contains(&ext_of(path).as_str())
}

/// PDF, Video oder Audio: Vorschau und Eckdaten über macOS statt image
fn is_av(path: &Path) -> bool {
    is_pdf(path) || is_video(path) || is_audio(path)
}

pub fn is_text(path: &Path) -> bool {
    TEXT_EXTS.contains(&ext_of(path).as_str())
}

pub fn is_note(path: &Path) -> bool {
    NOTE_EXTS.contains(&ext_of(path).as_str())
}

/// Titel einer Notiz: erste Zeile mit Text, ohne Markdown-Raute
pub fn note_title(text: &str) -> String {
    let line = text.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("");
    line.trim_start_matches('#').trim().chars().take(80).collect()
}

/// Alles, was in die Bibliothek gehört: Bilder, PDFs, Videos, Audio, Textkarten.
pub fn is_media(path: &Path) -> bool {
    is_image(path) || is_pdf(path) || is_video(path) || is_audio(path) || is_text(path)
}

/// Endung aus den ersten Bytes, wenn der Name keine passende hat (PDF, Video, Bild).
fn sniff_ext(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"%PDF") {
        return Some("pdf");
    }
    if bytes.len() > 12 && &bytes[4..8] == b"ftyp" {
        if &bytes[8..11] == b"M4A" {
            return Some("m4a");
        }
        return Some(if &bytes[8..10] == b"qt" { "mov" } else { "mp4" });
    }
    if bytes.len() > 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WAVE" {
        return Some("wav");
    }
    if bytes.len() > 12 && bytes.starts_with(b"FORM") && (&bytes[8..12] == b"AIFF" || &bytes[8..12] == b"AIFC") {
        return Some("aif");
    }
    if bytes.starts_with(b"fLaC") {
        return Some("flac");
    }
    if bytes.starts_with(b"ID3") || (bytes.len() > 2 && bytes[0] == 0xff && (bytes[1] & 0xe0) == 0xe0 && (bytes[1] & 0x06) != 0) {
        return Some("mp3");
    }
    if bytes.starts_with(&[0x1a, 0x45, 0xdf, 0xa3]) {
        return Some("webm");
    }
    image::guess_format(bytes).ok().and_then(|f| f.extensions_str().first().copied())
}

// ---- Platzhalter (Audio ohne Cover) ----

/// Ein Bild für alle Tracks ohne Cover: dunkle Fläche, feiner Ring mit
/// Mittelpunkt wie eine Platte (600 × 600). Gleiche Farben wie der
/// Platzhalter in media.js.
fn audio_placeholder() -> image::RgbImage {
    let n = 600u32;
    let bg = [42.0f32, 42.0, 42.0];
    let fg = [140.0f32, 140.0, 140.0];
    let c = n as f32 / 2.0;
    let (ring, stroke, dot) = (n as f32 * 0.3, 1.6f32, n as f32 * 0.022);
    image::RgbImage::from_fn(n, n, |x, y| {
        let d = ((x as f32 + 0.5 - c).powi(2) + (y as f32 + 0.5 - c).powi(2)).sqrt();
        // Kantenglättung über den Abstand zur Linie bzw. zum Punkt
        let a_ring = (stroke / 2.0 + 0.5 - (d - ring).abs()).clamp(0.0, 1.0);
        let a_dot = (dot + 0.5 - d).clamp(0.0, 1.0);
        let a = a_ring.max(a_dot);
        image::Rgb([0, 1, 2].map(|i| (bg[i] + (fg[i] - bg[i]) * a).round() as u8))
    })
}

fn is_hidden(name: &std::ffi::OsStr) -> bool {
    name.to_str().map(|s| s.starts_with('.')).unwrap_or(false)
}

/// Relativer Pfad mit "/" als Trenner, "" für den Bibliotheksordner selbst.
fn rel_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => s.to_str().map(String::from),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn parent_rel(rel: &str) -> String {
    rel.rsplit_once('/').map(|(p, _)| p.to_string()).unwrap_or_default()
}

/// Zone eines Ordners: nur die ersten `depth` Ebenen. Tiefere Unterordner
/// gehören zu ihrer Zone (bestehende Projektordner werden nicht zerfasert).
/// Zone, die sich nach 7 Tagen leert (0.36.0): jeder Ordner namens Unsorted,
/// auch selbst angelegt, klein geschrieben, mit Nummer davor (00_Unsorted) oder
/// als Unterzone (Projekt/Unsorted), dazu alles darin.
pub fn is_unsorted(zone: &str) -> bool {
    zone.split('/').any(|seg| {
        seg.trim()
            .trim_start_matches(|c: char| c.is_ascii_digit() || matches!(c, '_' | '-' | '.' | ' '))
            .eq_ignore_ascii_case(INBOX)
    })
}

fn zone_at(dir: &str, depth: usize) -> String {
    if dir.is_empty() {
        return String::new();
    }
    dir.split('/').take(depth.max(1)).collect::<Vec<_>>().join("/")
}

/// Kleines Bild über Quick Look (`qlmanage -t`), höchstens 5 s
pub(crate) fn ql_image(src: &Path, size: u32, tmp: &Path) -> Option<image::DynamicImage> {
    let _ = fs::remove_dir_all(tmp);
    fs::create_dir_all(tmp).ok()?;
    let img = (|| {
        let mut child = std::process::Command::new("/usr/bin/qlmanage")
            .args(["-t", "-s", &size.to_string(), "-o"])
            .arg(tmp)
            .arg(src)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .ok()?;
        let start = std::time::Instant::now();
        while child.try_wait().ok()?.is_none() {
            if start.elapsed() > std::time::Duration::from_secs(5) {
                let _ = child.kill();
                return None;
            }
            std::thread::sleep(std::time::Duration::from_millis(30));
        }
        let png = fs::read_dir(tmp).ok()?.filter_map(|e| e.ok()).map(|e| e.path()).find(|p| ext_of(p) == "png")?;
        image::open(png).ok()
    })();
    let _ = fs::remove_dir_all(tmp);
    img
}

/// Erster Ordner unterhalb der Zone, in dem eine Datei liegt ("" = direkt in der Zone)
fn folder_in(dir: &str, zone: &str) -> String {
    let rest = if zone.is_empty() { dir } else { dir.strip_prefix(zone).and_then(|r| r.strip_prefix('/')).unwrap_or("") };
    rest.split('/').next().unwrap_or("").to_string()
}

/// Ordner, die Archiver in bestehenden Projekten überspringt (Layout-Links,
/// Schriften, Backups, Ableton-Technik). Einträge mit "/" passen auf das Pfadende.
pub const DEFAULT_SKIP: &[&str] = &["Links", "Document fonts", "Backup", "Ableton Project Info", "Presets", "Samples/Imported"];
pub const DEFAULT_DEPTH: usize = 2;

/// Pakete, die im Finder wie Dateien aussehen (Programme, Mediatheken, Projekte)
const PACKAGE_EXTS: &[&str] = &["app", "photoslibrary", "logicx", "fcpbundle", "band", "rtfd", "pages", "key", "numbers", "imovielibrary", "lrdata", "bundle", "framework", "plugin"];

fn is_package_dir(e: &walkdir::DirEntry) -> bool {
    e.file_type().is_dir()
        && e.path().extension().and_then(|x| x.to_str()).is_some_and(|x| PACKAGE_EXTS.contains(&x.to_lowercase().as_str()))
}

/// Ordnerregeln einer Bibliothek: Zonentiefe und übersprungene Ordner
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Structure {
    pub depth: usize,
    pub skip: Vec<String>,
    /// Unterordner als eigene Zonen (bis `depth`). Aus: jeder Ordner der
    /// obersten Ebene ist eine Zone, Unterordner erscheinen darin als Ordner.
    #[serde(default)]
    pub subzones: bool,
}

impl Structure {
    /// Ebenen, die eine Zone bilden
    pub fn zone_depth(&self) -> usize {
        if self.subzones {
            self.depth.clamp(2, 3)
        } else {
            1
        }
    }

    /// Wird dieser Ordner (relativer Pfad) übersprungen?
    pub fn skips(&self, rel: &str) -> bool {
        let name = rel.rsplit('/').next().unwrap_or(rel);
        if let Some((_, ext)) = name.rsplit_once('.') {
            if PACKAGE_EXTS.contains(&ext.to_lowercase().as_str()) {
                return true;
            }
        }
        let lower = rel.to_lowercase();
        self.skip.iter().any(|s| {
            let s = s.trim().trim_matches('/').to_lowercase();
            !s.is_empty() && (lower == s || lower.ends_with(&format!("/{s}")))
        })
    }
}

/// Tage unter „Zuletzt gelöscht“, bevor eine Datei in den Papierkorb des Macs geht
pub const DELETED_DAYS: i64 = 7;
/// .archiver/unsorted-seen.json: seit wann ein Objekt in Unsorted liegt (expire_inbox)
const INBOX_SEEN: &str = "unsorted-seen.json";
/// .archiver/screenshots-seen.json: dasselbe für Screenshots (expire_shots, 0.47.0)
const SHOTS_SEEN: &str = "screenshots-seen.json";
const DELETED_HASHES: &str = ".hashes.json";

/// Eintrag unter „Zuletzt gelöscht“: alter Pfad in der Bibliothek, Ort unter
/// .archiver/deleted, Zeitpunkt des Löschens
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeletedItem {
    pub path: String,
    pub abs: String,
    pub zone: String,
    pub filename: String,
    pub hash: String,
    pub format: String,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub thumb: Option<String>,
    pub quote: Option<String>,
    pub tone: Option<String>,
    pub deleted_at: i64,
    /// Keine Mediendatei (PSD, AI, INDD, ZIP …), kam mit einer gelöschten Zone
    pub other: bool,
}

/// Schreibt über eine eigene Temp-Datei und benennt dann um. App und Brücke
/// (eigener Prozess) können gleichzeitig schreiben, darum ist der Name je
/// Prozess und Aufruf eindeutig (0.31.0).
pub(crate) fn write_atomic(target: &Path, body: &[u8]) -> Result<()> {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let name = target.file_name().and_then(|n| n.to_str()).unwrap_or("datei");
    let tmp = target.with_file_name(format!(".{name}.{}-{n}.tmp", std::process::id()));
    if let Err(e) = fs::write(&tmp, body).and_then(|_| fs::rename(&tmp, target)) {
        let _ = fs::remove_file(&tmp);
        return Err(format!("{}: {e}", target.display()));
    }
    Ok(())
}

/// Dieselbe Datei auf der Platte (gleiches Gerät und gleicher Inode)
fn same_file(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

/// Verknüpfung (Symlink), ohne ihr zu folgen
fn is_link(p: &Path) -> bool {
    fs::symlink_metadata(p).map(|m| m.file_type().is_symlink()).unwrap_or(false)
}

/// Verschieben, notfalls über Volumegrenzen (kopieren, dann löschen)
/// Verschiebt eine Datei oder einen Ordner nach `dst` (das es noch nicht gibt).
/// Nur über Laufwerksgrenzen (EXDEV, etwa externe Zonen) wird kopiert und erst
/// danach gelöscht. Dabei gilt: Was nicht vollständig kopiert ist, bleibt am
/// alten Ort, und es wird nur gelöscht, was kopiert wurde (0.31.0). Andere
/// Fehler (Rechte, schreibgeschützt) werden gemeldet, nicht umgangen.
pub(crate) fn move_file(src: &Path, dst: &Path) -> Result<()> {
    match fs::rename(src, dst) {
        Ok(()) => return Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices => {}
        Err(e) => return Err(format!("{}: {e}", src.display())),
    }
    if fs::symlink_metadata(src).map_err(err)?.is_dir() {
        copy_dir_then_remove(src, dst)
    } else {
        copy_file(src, dst)?;
        if let Err(e) = fs::remove_file(src) {
            // Original lässt sich nicht entfernen (schreibgeschützt): Kopie zurücknehmen
            let _ = fs::remove_file(dst);
            return Err(format!("{}: {e}", src.display()));
        }
        Ok(())
    }
}

/// Kopiert über eine versteckte Teildatei und benennt erst um, wenn alles da ist.
fn copy_file(src: &Path, dst: &Path) -> Result<()> {
    let name = dst.file_name().and_then(|n| n.to_str()).unwrap_or("datei");
    let part = dst.with_file_name(format!(".{name}.archery-part"));
    let done = fs::copy(src, &part).map_err(err).and_then(|n| {
        let want = fs::metadata(src).map_err(err)?.len();
        if n == want {
            Ok(())
        } else {
            Err(format!("{}: unvollständig kopiert ({n} von {want} Byte)", src.display()))
        }
    });
    if let Err(e) = done.and_then(|_| fs::rename(&part, dst).map_err(err)) {
        let _ = fs::remove_file(&part);
        return Err(e);
    }
    Ok(())
}

fn copy_dir_then_remove(src: &Path, dst: &Path) -> Result<()> {
    let mut copied = Vec::new();
    let mut dirs = Vec::new();
    let result = (|| -> Result<()> {
        fs::create_dir_all(dst).map_err(err)?;
        for entry in WalkDir::new(src).min_depth(1) {
            let entry = entry.map_err(err)?;
            let to = dst.join(entry.path().strip_prefix(src).map_err(err)?);
            let kind = entry.file_type();
            if kind.is_dir() {
                fs::create_dir_all(&to).map_err(err)?;
                dirs.push(entry.path().to_path_buf());
            } else if kind.is_symlink() {
                #[cfg(unix)]
                std::os::unix::fs::symlink(fs::read_link(entry.path()).map_err(err)?, &to).map_err(err)?;
                copied.push(entry.path().to_path_buf());
            } else {
                copy_file(entry.path(), &to)?;
                copied.push(entry.path().to_path_buf());
            }
        }
        Ok(())
    })();
    if let Err(e) = result {
        // Nichts halb verschoben: die neue Kopie verwerfen, das Original bleibt
        let _ = fs::remove_dir_all(dst);
        return Err(e);
    }
    // Nur löschen, was kopiert wurde. Was währenddessen neu hinzukam, bleibt liegen.
    for f in &copied {
        let _ = fs::remove_file(f);
    }
    dirs.sort_by_key(|d| std::cmp::Reverse(d.components().count()));
    for d in dirs {
        let _ = fs::remove_dir(d);
    }
    let _ = fs::remove_dir(src);
    Ok(())
}

fn write_deleted_hashes(stamp: &Path, hashes: &HashMap<String, String>) -> Result<()> {
    if hashes.is_empty() {
        return Ok(());
    }
    let mut all = read_deleted_hashes(stamp);
    all.extend(hashes.iter().map(|(k, v)| (k.clone(), v.clone())));
    fs::write(stamp.join(DELETED_HASHES), serde_json::to_vec(&all).map_err(err)?).map_err(err)
}

fn read_deleted_hashes(stamp: &Path) -> HashMap<String, String> {
    fs::read(stamp.join(DELETED_HASHES)).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

/// Leere Ordner von unten nach oben entfernen, samt `dir` selbst. Eine Liste
/// der Hashes allein hält einen Ordner nicht am Leben.
fn prune_empty(dir: &Path) {
    let mut dirs: Vec<PathBuf> = WalkDir::new(dir).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_dir()).map(|e| e.into_path()).collect();
    dirs.sort_by_key(|d| std::cmp::Reverse(d.components().count()));
    for d in dirs {
        let only_meta = fs::read_dir(&d).map(|rd| rd.filter_map(|e| e.ok()).all(|e| e.file_name() == DELETED_HASHES)).unwrap_or(false);
        if only_meta {
            let _ = fs::remove_file(d.join(DELETED_HASHES));
            let _ = fs::remove_dir(&d);
        }
    }
}

/// Ordner ohne sichtbaren Inhalt
fn dir_is_empty(dir: &Path) -> bool {
    fs::read_dir(dir).map(|rd| rd.filter_map(|e| e.ok()).all(|e| is_hidden(&e.file_name()))).unwrap_or(true)
}

fn file_name(rel: &str) -> String {
    rel.rsplit_once('/').map(|(_, f)| f.to_string()).unwrap_or_else(|| rel.to_string())
}

/// Prüft einen relativen Pfad aus der Oberfläche: keine "..", keine absoluten
/// Pfade, nichts Verstecktes. Gibt den absoluten Pfad in der Bibliothek zurück.
/// Relativer Pfad in der Bibliothek ohne `..`, versteckte oder absolute Teile.
pub fn check_rel(rel: &str) -> Result<()> {
    if rel.starts_with('/') {
        return Err(format!("Ungültiger Pfad: {rel}"));
    }
    safe_join(Path::new(""), rel).map(|_| ())
}

fn safe_join(root: &Path, rel: &str) -> Result<PathBuf> {
    let mut out = root.to_path_buf();
    for seg in rel.split('/').filter(|s| !s.is_empty()) {
        if seg == "." || seg == ".." || seg.starts_with('.') || seg.contains('\\') {
            return Err(format!("Ungültiger Pfad: {rel}"));
        }
        out.push(seg);
    }
    Ok(out)
}

/// Zonen- bzw. Ordnernamen bereinigen (wie in der Extension).
pub fn clean_segment(name: &str) -> String {
    let cleaned: String = name
        .trim()
        .chars()
        .map(|c| if matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') { '_' } else { c })
        .collect();
    let cleaned = cleaned.trim_start_matches('.').trim().to_string();
    cleaned.chars().take(120).collect()
}

/// Freier Dateiname im Zielordner: "bild.jpg", "bild 2.jpg", "bild 3.jpg" …
pub(crate) fn unique_path(dir: &Path, filename: &str) -> PathBuf {
    let candidate = dir.join(filename);
    if !candidate.exists() {
        return candidate;
    }
    let p = Path::new(filename);
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
    let ext = p.extension().and_then(|s| s.to_str());
    for n in 2.. {
        let name = match ext {
            Some(e) => format!("{stem} {n}.{e}"),
            None => format!("{stem} {n}"),
        };
        let c = dir.join(name);
        if !c.exists() {
            return c;
        }
    }
    unreachable!()
}

/// Dateiname für ein übernommenes Bild: Wunschname bzw. Name der Datei,
/// bereinigt und bei Bedarf mit passender Endung (z. B. "bild" → "bild.webp").
fn image_file_name(src: &Path, wanted: Option<&str>) -> String {
    let base = wanted
        .map(clean_segment)
        .filter(|n| !n.is_empty())
        .or_else(|| src.file_name().and_then(|n| n.to_str()).map(clean_segment))
        .unwrap_or_else(|| "image".into());
    let mut head = vec![0u8; 64];
    let n = fs::File::open(src).and_then(|mut f| std::io::Read::read(&mut f, &mut head)).unwrap_or(0);
    let sniffed = sniff_ext(&head[..n]);
    // Passende Endung behalten; ein PDF oder Video mit Bild-Endung (z. B. aus
    // einer Download-Adresse ohne Endung) bekommt die richtige.
    let named = Path::new(&base);
    let fits = match sniffed {
        Some("pdf") => is_pdf(named),
        Some("mp4" | "mov" | "webm") => is_video(named),
        Some("wav" | "aif" | "flac" | "mp3" | "m4a") => is_audio(named),
        _ => is_media(named),
    };
    if fits {
        return base;
    }
    let ext = sniffed.unwrap_or("jpg");
    if is_media(named) {
        return named.with_extension(ext).to_string_lossy().into_owned();
    }
    format!("{base}.{ext}")
}

/// Farbanteile eines Bildes in groben, benannten Farben (Anteil ≥ 8 %),
/// größter zuerst, z. B. "blue:0.46 white:0.31".
pub fn color_shares(path: &Path) -> Option<String> {
    let img = image::open(path).ok()?.thumbnail(64, 64).to_rgba8();
    let mut counts: HashMap<&'static str, u32> = HashMap::new();
    let mut total = 0u32;
    for p in img.pixels() {
        if p[3] < 128 {
            continue;
        }
        *counts.entry(color_name(p[0], p[1], p[2])).or_default() += 1;
        total += 1;
    }
    if total == 0 {
        return Some(String::new());
    }
    let mut list: Vec<_> = counts.into_iter().map(|(n, c)| (n, c as f32 / total as f32)).filter(|(_, f)| *f >= 0.08).collect();
    list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    Some(list.iter().map(|(n, f)| format!("{n}:{f:.2}")).collect::<Vec<_>>().join(" "))
}

/// Grobe Farbklasse eines Pixels über Helligkeit, Sättigung und Farbton.
pub fn color_name(r: u8, g: u8, b: u8) -> &'static str {
    let (r, g, b) = (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let d = max - min;
    let s = if d == 0.0 { 0.0 } else { d / (1.0 - (2.0 * l - 1.0).abs()) };
    if l < 0.13 {
        return "black";
    }
    if l > 0.9 && s < 0.5 {
        return "white";
    }
    if s < 0.16 || d < 0.07 {
        return if l > 0.75 { "white" } else { "gray" };
    }
    let mut h = if max == r {
        60.0 * (((g - b) / d) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    if h < 0.0 {
        h += 360.0;
    }
    match h {
        h if h < 15.0 || h >= 340.0 => {
            if l < 0.4 && s < 0.6 { "brown" } else { "red" }
        }
        h if h < 45.0 => {
            if l < 0.45 { "brown" } else { "orange" }
        }
        h if h < 70.0 => {
            if l < 0.35 { "brown" } else { "yellow" }
        }
        h if h < 165.0 => "green",
        h if h < 195.0 => "teal",
        h if h < 255.0 => "blue",
        h if h < 290.0 => "purple",
        _ => "pink",
    }
}

pub fn hash_file(path: &Path) -> Result<String> {
    let mut hasher = blake3::Hasher::new();
    let mut f = fs::File::open(path).map_err(err)?;
    std::io::copy(&mut f, &mut hasher).map_err(err)?;
    Ok(hasher.finalize().to_hex()[..32].to_string())
}

pub(crate) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS item (
  path     TEXT PRIMARY KEY,
  hash     TEXT NOT NULL,
  bytes    INTEGER NOT NULL,
  mtime    INTEGER NOT NULL,
  added_at INTEGER NOT NULL,
  width    INTEGER,
  height   INTEGER,
  format   TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS item_hash ON item(hash);
CREATE TABLE IF NOT EXISTS source (
  hash        TEXT PRIMARY KEY,
  image_url   TEXT,
  page_url    TEXT,
  page_title  TEXT,
  captured_at INTEGER
);
CREATE TABLE IF NOT EXISTS thumb (
  hash TEXT PRIMARY KEY,
  file TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS media (
  hash        TEXT PRIMARY KEY,
  pages       INTEGER,
  duration    REAL,
  width       INTEGER,
  height      INTEGER,
  title       TEXT,
  artist      TEXT,
  album       TEXT,
  sample_rate INTEGER,
  bit_depth   INTEGER,
  channels    INTEGER
);
CREATE TABLE IF NOT EXISTS analysis (
  hash   TEXT PRIMARY KEY,
  labels TEXT NOT NULL,
  text   TEXT NOT NULL,
  colors TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS note (
  hash       TEXT PRIMARY KEY,
  note       TEXT,
  credit     TEXT,
  license    TEXT,
  internal   INTEGER,
  added_by   TEXT,
  updated_at INTEGER NOT NULL DEFAULT 0,
  tone       TEXT,
  caption    TEXT,
  links      TEXT,
  mark       TEXT
);
";

/// Verbindung öffnen, einstellen, Schema anlegen und kurz auf Schäden prüfen
fn open_db(db: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(db)?;
    // Die Brücke zur Extension öffnet dieselbe Datenbank parallel zur App.
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
    conn.execute_batch(SCHEMA)?;
    let check: String = conn.query_row("PRAGMA quick_check(1)", [], |r| r.get(0))?;
    if check != "ok" {
        return Err(rusqlite::Error::SqliteFailure(rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CORRUPT), Some(check)));
    }
    Ok(conn)
}

fn is_corrupt(e: &rusqlite::Error) -> bool {
    matches!(e.sqlite_error_code(), Some(rusqlite::ErrorCode::DatabaseCorrupt) | Some(rusqlite::ErrorCode::NotADatabase))
}

impl Library {
    /// Öffnet (und legt bei Bedarf an) eine Bibliothek in `root`.
    pub fn open(root: &Path) -> Result<Library> {
        migrate_meta_dir(root);
        fs::create_dir_all(root.join(META_DIR).join("thumbs")).map_err(err)?;
        let shared = crate::studio::is_shared(root);
        // Geteilt: SQLite gehört nicht auf ein Netzlaufwerk, jede Person hat ihren eigenen Cache.
        let db = if shared { crate::studio::local_db(root)? } else { root.join(META_DIR).join("library.db") };
        // Die Datenbank ist nur ein Zwischenspeicher: Ordner und .archiver/metadata.json
        // sind die Wahrheit. Ist sie beschädigt, wird sie beiseitegelegt und neu
        // aufgebaut, statt die Bibliothek gar nicht mehr zu öffnen (0.31.0).
        let conn = match open_db(&db) {
            Err(e) if is_corrupt(&e) => {
                let aside = db.with_extension(format!("db.broken-{}", now_ms()));
                crate::log::error("db", format!("{} beschädigt ({e}), neu aufgebaut, alte Datei: {}", db.display(), aside.display()));
                fs::rename(&db, &aside).map_err(err)?;
                for ext in ["db-wal", "db-shm"] {
                    let _ = fs::remove_file(db.with_extension(ext));
                }
                open_db(&db).map_err(err)?
            }
            r => r.map_err(err)?,
        };
        // ab 0.9: Text der Zitat-Karten (ältere Datenbanken bekommen die Spalte nachgereicht)
        let _ = conn.execute("ALTER TABLE source ADD COLUMN quote TEXT", []);
        // ab 0.20.1: Farbe der Textkarten
        let _ = conn.execute("ALTER TABLE note ADD COLUMN tone TEXT", []);
        // ab 0.29: Bildtext
        let _ = conn.execute("ALTER TABLE note ADD COLUMN caption TEXT", []);
        // ab 0.30: Das Feld „Notiz“ geht im Bildtext auf (hängt sich an einen vorhandenen
        // Bildtext an). Läuft bei jedem Öffnen, betrifft nur noch nicht übernommene Zeilen.
        let _ = conn.execute(
            "UPDATE note SET caption = CASE WHEN caption IS NULL OR trim(caption) = '' THEN note \
             ELSE caption || char(10) || char(10) || note END, note = NULL \
             WHERE note IS NOT NULL AND trim(note) <> ''",
            [],
        );
        // ab 0.29: Verknüpfungen einer Notiz
        let _ = conn.execute("ALTER TABLE note ADD COLUMN links TEXT", []);
        // ab 0.41.0: Farbmarke in der Übersicht
        let _ = conn.execute("ALTER TABLE note ADD COLUMN mark TEXT", []);
        // ab 0.12: Audio-Eckdaten (ältere Datenbanken bekommen die Spalten nachgereicht)
        for col in ["title TEXT", "artist TEXT", "album TEXT", "sample_rate INTEGER", "bit_depth INTEGER", "channels INTEGER"] {
            let _ = conn.execute(&format!("ALTER TABLE media ADD COLUMN {col}"), []);
        }
        // ab 0.10: PDF-Vorschauen ohne hellen Rand neu rechnen (einmalig, user_version 1)
        let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap_or(0);
        if version < 1 {
            let _ = conn.execute("DELETE FROM thumb WHERE hash IN (SELECT hash FROM media WHERE pages IS NOT NULL)", []);
            let _ = conn.execute_batch("PRAGMA user_version = 1;");
        }
        // ab 0.13.1: Audio ohne Cover bekommt den Platzhalter statt der Wellenform
        if version < 2 {
            let _ = conn.execute("DELETE FROM thumb WHERE hash IN (SELECT hash FROM media WHERE sample_rate IS NOT NULL OR title IS NOT NULL)", []);
            let _ = conn.execute_batch("PRAGMA user_version = 2;");
        }
        // ab 0.31.1: Motive mit Sicherheit („poster:0.92“). Ältere Analysen ohne
        // Sicherheit laufen einmal neu, damit die Suche unsichere Motive auslässt.
        if version < 3 {
            let _ = conn.execute("DELETE FROM analysis WHERE labels <> '' AND labels NOT LIKE '%:%'", []);
            let _ = conn.execute_batch("PRAGMA user_version = 3;");
        }
        // ab 0.31.2: Gesichter und Personen eigens erkannt. Alle Bilder laufen
        // einmal neu durch die Analyse (Hintergrund, Farben gleich mit).
        if version < 4 {
            let _ = conn.execute("DELETE FROM analysis", []);
            let _ = conn.execute_batch("PRAGMA user_version = 4;");
        }
        // ab 0.36.3: Analyse am Original statt an der Vorschau, dazu Tiere, Gruppen und
        // mehr Sprachen im Bildtext. Alles läuft einmal neu (Hintergrund).
        if version < 5 {
            let _ = conn.execute("DELETE FROM analysis", []);
            let _ = conn.execute_batch("PRAGMA user_version = 5;");
        }
        let lib = Library {
            root: root.to_path_buf(),
            conn: Mutex::new(conn),
            scan_lock: Mutex::new(()),
            failed_thumbs: Mutex::new(HashSet::new()),
            shared,
            user: Mutex::new(None),
            people_sig: Mutex::new(0),
        };
        // Unsorted (Standard-Zone) gibt es in jeder eigenen Bibliothek von Anfang
        // an, auch in einem bestehenden Ordnersystem (0.33.1). Geteilte
        // Bibliotheken bekommen sie nur, wenn sie noch leer sind.
        let has_folders = fs::read_dir(root)
            .map(|rd| rd.filter_map(|e| e.ok()).any(|e| e.path().is_dir() && !is_hidden(&e.file_name())))
            .unwrap_or(false);
        lib.migrate_inbox()?;
        if !shared || !has_folders {
            fs::create_dir_all(root.join(lib.default_zone())).map_err(err)?;
        }
        lib.restore_metadata()?;
        lib.merge_people()?;
        Ok(lib)
    }

    pub(crate) fn metadata_path(&self) -> PathBuf {
        self.root.join(META_DIR).join("metadata.json")
    }

    fn zone_order_path(&self) -> PathBuf {
        self.root.join(META_DIR).join("zones.json")
    }

    /// Reihenfolge der Zonen und die vier Hot Zones aus der Übersicht der App.
    /// Liegt in der Bibliothek, damit die Extension (über die Bridge) dieselben
    /// Zonen zeigt. Fehlt die Datei, sind beide Listen leer.
    pub fn zone_order(&self) -> (Vec<String>, Vec<String>) {
        let v: serde_json::Value = fs::read(self.zone_order_path())
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        let list = |key: &str| -> Vec<String> {
            v.get(key)
                .and_then(|a| a.as_array())
                .map(|a| a.iter().filter_map(|s| s.as_str().map(String::from)).collect())
                .unwrap_or_default()
        };
        (list("order"), list("hot"))
    }

    /// Speichert Reihenfolge und Hot Zones; schreibt nur bei Änderung.
    /// Weitere Schlüssel in zones.json (z. B. "alpha") bleiben erhalten.
    pub fn set_zone_order(&self, order: &[String], hot: &[String]) -> Result<()> {
        if self.zone_order() == (order.to_vec(), hot.to_vec()) {
            return Ok(());
        }
        let mut v = self.zone_file();
        v["order"] = serde_json::json!(order);
        v["hot"] = serde_json::json!(hot);
        self.write_zone_file(&v)
    }

    /// Standard-Zone: hierhin kommt alles, was ohne Ziel gesammelt wird
    /// (Extension ohne Zone, lose Bilder beim Import). Ohne Wahl die Inbox.
    pub fn default_zone(&self) -> String {
        self.zone_file()
            .get("default")
            .and_then(|z| z.as_str())
            .filter(|z| !z.is_empty() && self.root.join(z).is_dir())
            .map(String::from)
            .unwrap_or_else(|| {
                if !self.root.join(INBOX).is_dir() && self.root.join(LEGACY_INBOX).is_dir() {
                    LEGACY_INBOX.to_string()
                } else {
                    INBOX.to_string()
                }
            })
    }

    /// Bis 0.23 hieß die Standard-Zone „Inbox“. In eigenen Bibliotheken wird
    /// sie zu „Unsorted“, sofern sie Standard ist und es „Unsorted“ noch nicht gibt.
    fn migrate_inbox(&self) -> Result<()> {
        if self.shared || self.root.join(INBOX).exists() || !self.root.join(LEGACY_INBOX).is_dir() {
            return Ok(());
        }
        let stored = self.zone_file().get("default").and_then(|z| z.as_str()).map(String::from);
        if stored.as_deref().is_some_and(|z| z != LEGACY_INBOX && self.root.join(z).is_dir()) {
            return Ok(());
        }
        let new_zone = self.rename_zone(LEGACY_INBOX, INBOX)?;
        let mut v = self.zone_file();
        v["default"] = serde_json::json!(new_zone);
        for key in ["order", "hot"] {
            if let Some(list) = v.get_mut(key).and_then(|a| a.as_array_mut()) {
                for z in list.iter_mut() {
                    if z.as_str() == Some(LEGACY_INBOX) {
                        *z = serde_json::json!(new_zone);
                    }
                }
            }
        }
        self.write_zone_file(&v)
    }

    /// Zone, die sich nach 7 Tagen selbst leert: immer Unsorted (0.35, vorher
    /// wählbar und an die Standard-Zone gebunden). "" heißt aus (Einstellungen ›
    /// Bibliothek); jede frühere Wahl einer anderen Zone gilt als Unsorted.
    /// Welche Ordner als Unsorted zählen, sagt is_unsorted (0.36.0).
    pub fn expire_zone(&self) -> Option<String> {
        let set = self.zone_file().get("expire").and_then(|z| z.as_str()).map(str::to_string);
        if set.as_deref() == Some("") {
            return None;
        }
        // Geteilte Bibliotheken räumen nur, wenn das dort ausdrücklich angeschaltet wurde
        if self.shared && set.as_deref() != Some(INBOX) {
            return None;
        }
        Some(INBOX.to_string())
    }

    pub fn set_expire_zone(&self, zone: &str) -> Result<()> {
        if !zone.is_empty() && zone != INBOX {
            return Err(format!("Nur Unsorted leert sich nach 7 Tagen: {zone}"));
        }
        // Wieder an: Fristen beginnen neu. Sonst wanderte alles, was schon vor
        // dem Ausschalten in Unsorted lag, beim nächsten Lauf sofort nach
        // „Zuletzt gelöscht“.
        if !zone.is_empty() && self.expire_zone().is_none() {
            let _ = fs::remove_file(self.root.join(META_DIR).join(INBOX_SEEN));
        }
        let mut v = self.zone_file();
        v["expire"] = serde_json::json!(zone);
        self.write_zone_file(&v)
    }

    /// Screenshots räumt nach 7 Tagen auf (0.47.0). Erst an, wenn die
    /// Bildschirmfotos einmal hierher gelenkt wurden (set_shots_expire), damit
    /// ein eigener Ordner gleichen Namens nicht plötzlich geleert wird.
    pub fn shots_expire(&self) -> bool {
        self.zone_file().get("shotsExpire").and_then(|v| v.as_bool()).unwrap_or(false)
    }

    pub fn set_shots_expire(&self, on: bool) -> Result<()> {
        // Wieder an: Fristen beginnen neu (wie set_expire_zone)
        if on && !self.shots_expire() {
            let _ = fs::remove_file(self.root.join(META_DIR).join(SHOTS_SEEN));
        }
        let mut v = self.zone_file();
        v["shotsExpire"] = serde_json::json!(on);
        self.write_zone_file(&v)
    }

    /// Wurde Aufräumen hier noch nie gewählt, gilt es ab dem ersten Einschalten
    pub fn shots_expire_default(&self) -> Result<()> {
        if self.zone_file().get("shotsExpire").is_none() {
            self.set_shots_expire(true)?;
        }
        Ok(())
    }

    /// Ordner für Bildschirmfotos, bei Bedarf angelegt
    pub fn shots_dir(&self) -> Result<PathBuf> {
        let dir = self.root.join(SHOTS);
        fs::create_dir_all(&dir).map_err(err)?;
        Ok(dir)
    }

    pub fn set_default_zone(&self, zone: &str) -> Result<()> {
        let dir = safe_join(&self.root, zone)?;
        if zone.is_empty() || !dir.is_dir() {
            return Err(format!("Zone nicht gefunden: {zone}"));
        }
        let mut v = self.zone_file();
        v["default"] = serde_json::json!(zone);
        self.write_zone_file(&v)
    }

    pub fn structure(&self) -> Structure {
        let v = self.zone_file();
        let depth = v.get("depth").and_then(|d| d.as_u64()).map(|d| d as usize).filter(|d| (2..=3).contains(d)).unwrap_or(DEFAULT_DEPTH);
        let skip = v
            .get("skip")
            .and_then(|a| a.as_array())
            .map(|a| a.iter().filter_map(|s| s.as_str().map(String::from)).collect())
            .unwrap_or_else(|| DEFAULT_SKIP.iter().map(|s| s.to_string()).collect());
        let subzones = v.get("subzones").and_then(|b| b.as_bool()).unwrap_or(false);
        Structure { depth, skip, subzones }
    }

    pub fn set_structure(&self, st: &Structure) -> Result<()> {
        let mut v = self.zone_file();
        v["depth"] = serde_json::json!(st.depth.clamp(2, 3));
        v["subzones"] = serde_json::json!(st.subzones);
        v["skip"] = serde_json::json!(st.skip.iter().map(|s| s.trim()).filter(|s| !s.is_empty()).collect::<Vec<_>>());
        self.write_zone_file(&v)?;
        self.scan()?;
        Ok(())
    }

    /// Ecken (square, soft, round, xl) und Deckkraft der Zonenkarten (Glas-Regler,
    /// None = Standard) aus Einstellungen › Aussehen. Die App trägt sie hier ein,
    /// damit die Extension dieselben Werte zeigt.
    pub fn look_radius(&self) -> Option<String> {
        self.zone_file().get("radius").and_then(|r| r.as_str()).map(String::from)
    }

    pub fn look_alpha(&self) -> Option<f64> {
        self.zone_file().get("alpha").and_then(|a| a.as_f64())
    }

    pub fn set_look(&self, radius: &str, alpha: Option<f64>) -> Result<()> {
        let alpha = alpha.map(|a| a.clamp(0.3, 1.0));
        if self.look_radius().as_deref() == Some(radius) && self.look_alpha() == alpha {
            return Ok(());
        }
        let mut v = self.zone_file();
        v["radius"] = serde_json::json!(radius);
        v["alpha"] = serde_json::json!(alpha);
        self.write_zone_file(&v)
    }

    pub(crate) fn zone_file(&self) -> serde_json::Value {
        let v: serde_json::Value = fs::read(self.zone_order_path())
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        if v.is_object() {
            v
        } else {
            serde_json::json!({})
        }
    }

    pub(crate) fn write_zone_file(&self, v: &serde_json::Value) -> Result<()> {
        let body = serde_json::to_vec_pretty(v).map_err(err)?;
        write_atomic(&self.zone_order_path(), &body)
    }

    fn thumbs_dir(&self) -> PathBuf {
        self.root.join(META_DIR).join("thumbs")
    }

    /// Neue oder geleerte Datenbank: Quellen aus metadata.json zurückholen.
    /// Notizen und Rechte werden immer abgeglichen (neuere gewinnt).
    fn restore_metadata(&self) -> Result<()> {
        let Ok(text) = fs::read_to_string(self.metadata_path()) else { return Ok(()) };
        let Ok(meta) = serde_json::from_str::<crate::studio::MetaDoc>(&text) else { return Ok(()) };
        self.merge_notes(&meta.notes)?;
        // Fehlende Quellen ergänzen (z. B. nach dem Wechsel zurück von einer geteilten Bibliothek)
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction().map_err(err)?;
        for (hash, s) in meta.sources {
            tx.execute(
                "INSERT INTO source (hash, image_url, page_url, page_title, captured_at, quote) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(hash) DO UPDATE SET
                   image_url = COALESCE(source.image_url, ?2),
                   page_url = COALESCE(source.page_url, ?3),
                   page_title = COALESCE(source.page_title, ?4),
                   captured_at = COALESCE(source.captured_at, ?5),
                   quote = COALESCE(source.quote, ?6)",
                params![hash, s.image_url, s.page_url, s.page_title, s.captured_at, s.quote],
            )
            .map_err(err)?;
        }
        tx.commit().map_err(err)
    }

    /// Schreibt alle Quellen nach .archiver/metadata.json, damit nichts an der DB hängt.
    /// Geteilt: in die eigene Datei unter .archiver/people/.
    pub fn write_metadata(&self) -> Result<()> {
        let target = if self.shared { self.person_file() } else { self.metadata_path() };
        self.write_metadata_to(&target)
    }

    /// Wie write_metadata, aber in eine bestimmte Datei (Wechsel geteilt ↔ allein).
    pub fn write_metadata_to(&self, target: &Path) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT hash, image_url, page_url, page_title, captured_at, quote FROM source ORDER BY hash")
            .map_err(err)?;
        let mut sources = serde_json::Map::new();
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    Source {
                        image_url: r.get(1)?,
                        page_url: r.get(2)?,
                        page_title: r.get(3)?,
                        captured_at: r.get(4)?,
                        quote: r.get(5)?,
                    },
                ))
            })
            .map_err(err)?;
        for row in rows {
            let (hash, s) = row.map_err(err)?;
            sources.insert(hash, serde_json::to_value(s).map_err(err)?);
        }
        drop(stmt);
        drop(conn);
        let notes = self.all_notes()?;
        let doc = serde_json::json!({ "version": 1, "sources": sources, "notes": notes });
        // Geteilt: jede Person schreibt nur ihre eigene Datei, so überschreibt niemand den anderen.
        if let Some(dir) = target.parent() {
            fs::create_dir_all(dir).map_err(err)?;
        }
        write_atomic(target, &serde_json::to_vec_pretty(&doc).map_err(err)?)?;
        if self.shared {
            *self.people_sig.lock().unwrap() = self.people_signature();
        }
        Ok(())
    }

    /// Gleicht die Datenbank mit dem Ordner ab. Gibt true zurück, wenn sich etwas geändert hat.
    /// Gehasht werden nur neue oder veränderte Dateien (Größe oder Änderungsdatum anders).
    pub fn scan(&self) -> Result<bool> {
        let _guard = self.scan_lock.lock().unwrap();
        let known: HashMap<String, (i64, i64)> = {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn.prepare("SELECT path, bytes, mtime FROM item").map_err(err)?;
            let rows = stmt
                .query_map([], |r| Ok((r.get::<_, String>(0)?, (r.get(1)?, r.get(2)?))))
                .map_err(err)?;
            rows.collect::<std::result::Result<_, _>>().map_err(err)?
        };

        // Fehlt der Bibliotheksordner selbst (Netz weg, Laufwerk ab), wäre alles
        // „verschwunden“. Dann lieber gar nichts ändern.
        if !self.root.is_dir() {
            return Err("library-missing".into());
        }
        let mut seen = HashSet::new();
        let mut changed = false;
        let st = self.structure();
        let root = self.root.clone();
        // Bibliothek, dazu jede externe Zone (Verknüpfung auf einen Ordner
        // außerhalb, 0.30.1); deren Pfade bleiben „Zone/…“ unter der Bibliothek.
        // Was sich nicht lesen ließ (offline, keine Rechte), gilt nicht als gelöscht.
        let mut blind: Vec<String> = Vec::new();
        let mut starts: Vec<PathBuf> = vec![self.root.clone()];
        for (zone, _, online) in self.external_zones() {
            if online {
                starts.push(self.root.join(zone));
            } else {
                blind.push(zone);
            }
        }
        for start in starts {
            let filter_root = root.clone();
            let filter_st = st.clone();
            let walker = WalkDir::new(&start)
                .min_depth(1)
                .into_iter()
                .filter_entry(move |e| !is_hidden(e.file_name()) && !(e.file_type().is_dir() && filter_st.skips(&rel_path(&filter_root, e.path()))));
            for entry in walker {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(e) => {
                        blind.push(rel_path(&self.root, e.path().unwrap_or(&start)));
                        continue;
                    }
                };
                if !entry.file_type().is_file() || !is_media(entry.path()) {
                    continue;
                }
                let rel = rel_path(&self.root, entry.path());
                let Ok(meta) = entry.metadata() else {
                    blind.push(rel);
                    continue;
                };
                let bytes = meta.len() as i64;
                let mtime = time_ms(meta.modified()).unwrap_or(0);
                seen.insert(rel.clone());
                if known.get(&rel) == Some(&(bytes, mtime)) {
                    continue;
                }
                let Ok(hash) = hash_file(entry.path()) else { continue };
                let added = time_ms(meta.created()).unwrap_or(mtime);
                self.upsert_item(&rel, &hash, bytes, mtime, added, entry.path())?;
                changed = true;
            }
        }

        let unseen = |p: &String| blind.iter().any(|b| b.is_empty() || p == b || p.starts_with(&format!("{b}/")));
        let gone: Vec<&String> = known.keys().filter(|p| !seen.contains(*p) && !unseen(p)).collect();
        if !gone.is_empty() {
            let conn = self.conn.lock().unwrap();
            for p in gone {
                conn.execute("DELETE FROM item WHERE path = ?1", params![p]).map_err(err)?;
            }
            changed = true;
        }
        // Geteilt: Angaben der anderen übernehmen, sobald sich ihre Dateien ändern
        if self.merge_people()? {
            changed = true;
        }
        Ok(changed)
    }

    fn upsert_item(&self, rel: &str, hash: &str, bytes: i64, mtime: i64, added: i64, abs: &Path) -> Result<()> {
        let (w, h) = match image::image_dimensions(abs) {
            Ok((w, h)) => (Some(w as i64), Some(h as i64)),
            Err(_) => (None, None),
        };
        let format = ext_of(abs);
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO item (path, hash, bytes, mtime, added_at, width, height, format)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(path) DO UPDATE SET hash=?2, bytes=?3, mtime=?4, width=?6, height=?7, format=?8",
            params![rel, hash, bytes, mtime, added, w, h, format],
        )
        .map_err(err)?;
        Ok(())
    }

    /// Kurzer Fingerabdruck des Bestands (Anzahl, Pfade, Quellen), um Änderungen
    /// von außen zu bemerken, die schon in der Datenbank stehen.
    pub fn signature(&self) -> (i64, i64, i64, i64) {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT (SELECT COUNT(*) FROM item), (SELECT TOTAL(LENGTH(path)) FROM item),
                    (SELECT TOTAL(mtime % 1000003) FROM item),
                    (SELECT COUNT(*) FROM source) + (SELECT TOTAL(updated_at % 1000003) FROM note)",
            [],
            |r| Ok((r.get(0)?, r.get::<_, f64>(1)? as i64, r.get::<_, f64>(2)? as i64, r.get::<_, f64>(3)? as i64)),
        )
        .unwrap_or_default()
    }

    /// Alle Zonen: jeder sichtbare Unterordner, Inbox zuerst, dann alphabetisch.
    /// Liegen Bilder direkt im Bibliotheksordner, gibt es zusätzlich die Zone "".
    pub fn zones(&self) -> Result<Vec<Zone>> {
        let default = self.default_zone();
        let st = self.structure();
        let mut counts: HashMap<String, usize> = HashMap::new();
        {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn.prepare("SELECT path FROM item").map_err(err)?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0)).map_err(err)?;
            for p in rows {
                *counts.entry(zone_at(&parent_rel(&p.map_err(err)?), st.zone_depth())).or_default() += 1;
            }
        }
        let root = self.root.clone();
        let links = self.external_zones();
        let offline: HashSet<String> = links.iter().filter(|(_, _, on)| !on).map(|(z, _, _)| z.clone()).collect();
        let mut dirs: Vec<PathBuf> = WalkDir::new(&self.root)
            .min_depth(1)
            .max_depth(st.zone_depth())
            .into_iter()
            .filter_entry(|e| !is_hidden(e.file_name()) && !(e.file_type().is_dir() && st.skips(&rel_path(&root, e.path()))))
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir() || (e.depth() == 1 && links.iter().any(|(z, _, _)| e.file_name().to_str() == Some(z))))
            .map(|e| e.into_path())
            .collect();
        // Unterzonen in externen Zonen: WalkDir folgt der Verknüpfung nicht,
        // darum jede erreichbare externe Zone eigens durchgehen (wie im Scan)
        if st.zone_depth() > 1 {
            for (zone, _, _) in links.iter().filter(|(_, _, on)| *on) {
                dirs.extend(
                    WalkDir::new(self.root.join(zone))
                        .min_depth(1)
                        .max_depth(st.zone_depth() - 1)
                        .into_iter()
                        .filter_entry(|e| !is_hidden(e.file_name()) && !(e.file_type().is_dir() && st.skips(&rel_path(&root, e.path()))))
                        .filter_map(|e| e.ok())
                        .filter(|e| e.file_type().is_dir())
                        .map(|e| e.into_path()),
                );
            }
        }
        let mut zones: Vec<Zone> = dirs
            .into_iter()
            .filter_map(|dir| {
                let path = rel_path(&self.root, &dir);
                let count = counts.get(&path).copied().unwrap_or(0);
                // Ordner ohne Bilder & Co., in denen aber anderes liegt (Projekt-
                // ordner, Technik): keine Zone. Ganz leere Ordner (neue Zonen) bleiben.
                // Externe Zonen offline bleiben stehen, ihre Vorschauen liegen hier.
                if count == 0 && path != default && !offline.contains(&path) && !dir_is_empty(&dir) {
                    return None;
                }
                let external = links.iter().any(|(z, _, _)| path == *z || path.starts_with(&format!("{z}/")));
                Some(Zone { count, name: path.clone(), path, external })
            })
            .collect();
        zones.sort_by(|a, b| {
            let rank = |z: &Zone| if z.path == default { 0 } else { 1 };
            rank(a).cmp(&rank(b)).then_with(|| a.path.to_lowercase().cmp(&b.path.to_lowercase()))
        });
        if let Some(&count) = counts.get("") {
            let name = self.root.file_name().and_then(|n| n.to_str()).unwrap_or("Archery").to_string();
            zones.insert(1.min(zones.len()), Zone { path: String::new(), name, count, external: false });
        }
        Ok(zones)
    }

    /// Alle Einträge mit Quelle und Vorschaubild, neueste zuerst.
    pub fn items(&self) -> Result<Vec<Item>> {
        self.items_under(None)
    }

    /// Nur Objekte unter diesem Ordner (Zone), für die Brücke: die Galerie der
    /// Extension braucht nicht die ganze Bibliothek (0.31.0)
    pub fn items_under(&self, prefix: Option<&str>) -> Result<Vec<Item>> {
        let depth = self.structure().zone_depth();
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT i.path, i.hash, i.bytes, i.mtime, i.added_at,
                        COALESCE(i.width, m.width), COALESCE(i.height, m.height), i.format,
                        s.image_url, s.page_url, s.page_title, s.captured_at, t.file,
                        a.labels, a.text, a.colors, m.pages, m.duration, s.quote,
                        m.title, m.artist, m.album, m.sample_rate, m.bit_depth, m.channels,
                        n.note, n.credit, n.license, n.internal, n.added_by, n.tone, n.caption, n.links, n.mark
                 FROM item i
                 LEFT JOIN note n ON n.hash = i.hash
                 LEFT JOIN source s ON s.hash = i.hash
                 LEFT JOIN thumb t ON t.hash = i.hash
                 LEFT JOIN analysis a ON a.hash = i.hash
                 LEFT JOIN media m ON m.hash = i.hash
                 WHERE ?1 IS NULL OR i.path = ?1 OR substr(i.path, 1, length(?1) + 1) = ?1 || '/'
                 ORDER BY COALESCE(s.captured_at, i.added_at) DESC, i.path",
            )
            .map_err(err)?;
        let thumbs = self.thumbs_dir();
        let rows = stmt
            .query_map([prefix], |r| {
                let path: String = r.get(0)?;
                let thumb: Option<String> = r.get(12)?;
                let zone = zone_at(&parent_rel(&path), depth);
                Ok(Item {
                    folder: folder_in(&parent_rel(&path), &zone),
                    abs: self.root.join(&path).to_string_lossy().into_owned(),
                    zone,
                    filename: file_name(&path),
                    hash: r.get(1)?,
                    bytes: r.get(2)?,
                    mtime: r.get(3)?,
                    added_at: r.get(4)?,
                    width: r.get(5)?,
                    height: r.get(6)?,
                    format: r.get(7)?,
                    image_url: r.get(8)?,
                    page_url: r.get(9)?,
                    page_title: r.get(10)?,
                    captured_at: r.get(11)?,
                    thumb: thumb.map(|f| thumbs.join(f).to_string_lossy().into_owned()),
                    labels: r.get(13)?,
                    text: r.get(14)?,
                    colors: r.get(15)?,
                    pages: r.get(16)?,
                    duration: r.get(17)?,
                    quote: r.get(18)?,
                    title: r.get(19)?,
                    artist: r.get(20)?,
                    album: r.get(21)?,
                    sample_rate: r.get(22)?,
                    bit_depth: r.get(23)?,
                    channels: r.get(24)?,
                    note: r.get(25)?,
                    credit: r.get(26)?,
                    license: r.get(27)?,
                    internal: r.get::<_, Option<i64>>(28)?.unwrap_or(0) != 0,
                    added_by: r.get(29)?,
                    tone: r.get(30)?,
                    caption: r.get(31)?,
                    links: crate::studio::links_of(r.get(32)?),
                    mark: r.get(33)?,
                    path,
                })
            })
            .map_err(err)?;
        let mut items: Vec<Item> = rows.collect::<std::result::Result<_, _>>().map_err(err)?;
        // Textdatei ohne gespeicherten Text (z. B. im Finder angelegt): Inhalt lesen
        for item in items.iter_mut().filter(|i| i.quote.is_none() && TEXT_EXTS.contains(&i.format.as_str())) {
            if let Ok(text) = fs::read_to_string(&item.abs) {
                let text: String = text.trim().chars().take(TEXT_MAX_CHARS).collect();
                if !text.is_empty() {
                    item.quote = Some(text);
                }
            }
        }
        Ok(items)
    }

    // ---- Vorschaubilder ----

    /// Einträge, für die noch ein Vorschaubild fehlt (je Hash einmal).
    pub fn missing_thumbs(&self) -> Result<Vec<(String, PathBuf)>> {
        let failed = self.failed_thumbs.lock().unwrap().clone();
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT i.hash, MIN(i.path), i.format FROM item i
                 LEFT JOIN thumb t ON t.hash = i.hash
                 WHERE t.hash IS NULL GROUP BY i.hash",
            )
            .map_err(err)?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?)))
            .map_err(err)?;
        let mut out = Vec::new();
        for row in rows {
            let (hash, path, format) = row.map_err(err)?;
            let media = PDF_EXTS.contains(&format.as_str())
                || VIDEO_EXTS.contains(&format.as_str())
                || AUDIO_EXTS.contains(&format.as_str());
            if (THUMB_EXTS.contains(&format.as_str()) || media) && !failed.contains(&hash) {
                out.push((hash, self.root.join(path)));
            }
        }
        Ok(out)
    }

    /// Rechnet ein Vorschaubild (max. 720 px, JPEG bzw. PNG bei Transparenz).
    pub fn make_thumb(&self, hash: &str, src: &Path) -> Option<String> {
        let result = (|| -> Result<String> {
            let img = if is_av(src) {
                let (img, info) = if is_audio(src) { self.audio_preview(src)? } else { self.media_preview(hash, src)? };
                let conn = self.conn.lock().unwrap();
                conn.execute(
                    "INSERT OR REPLACE INTO media (hash, pages, duration, width, height, title, artist, album, sample_rate, bit_depth, channels)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                    params![
                        hash,
                        info.pages,
                        info.duration,
                        info.width,
                        info.height,
                        info.title,
                        info.artist,
                        info.album,
                        info.sample_rate,
                        info.bit_depth,
                        info.channels
                    ],
                )
                .map_err(err)?;
                img
            } else {
                image::open(src).map_err(err)?
            };
            let small = if img.width() > THUMB_SIZE || img.height() > THUMB_SIZE {
                img.thumbnail(THUMB_SIZE, THUMB_SIZE)
            } else {
                img
            };
            let file = if small.color().has_alpha() {
                let name = format!("{hash}.png");
                small.to_rgba8().save(self.thumbs_dir().join(&name)).map_err(err)?;
                name
            } else {
                let name = format!("{hash}.jpg");
                let out = fs::File::create(self.thumbs_dir().join(&name)).map_err(err)?;
                let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(std::io::BufWriter::new(out), 82);
                enc.encode_image(&small.to_rgb8()).map_err(err)?;
                name
            };
            let conn = self.conn.lock().unwrap();
            conn.execute("INSERT OR REPLACE INTO thumb (hash, file) VALUES (?1, ?2)", params![hash, file])
                .map_err(err)?;
            Ok(self.thumbs_dir().join(file).to_string_lossy().into_owned())
        })();
        match result {
            Ok(p) => Some(p),
            Err(_) => {
                self.mark_thumb_failed(hash);
                None
            }
        }
    }

    /// Diese Datei nicht noch einmal versuchen (bis zum nächsten Start)
    pub fn mark_thumb_failed(&self, hash: &str) {
        self.failed_thumbs.lock().unwrap_or_else(|e| e.into_inner()).insert(hash.to_string());
    }

    /// Audio: eingebettetes Cover, sonst der Standard-Platzhalter. Eckdaten aus AVFoundation.
    fn audio_preview(&self, src: &Path) -> Result<(image::DynamicImage, MediaInfo)> {
        #[cfg(target_os = "macos")]
        let (info, cover) = (crate::macos::media_info(src), crate::macos::audio_artwork(src));
        #[cfg(not(target_os = "macos"))]
        let (info, cover): (MediaInfo, Option<Vec<u8>>) = (MediaInfo::default(), None);
        if let Some(img) = cover.and_then(|b| image::load_from_memory(&b).ok()) {
            return Ok((img, info));
        }
        Ok((image::DynamicImage::ImageRgb8(audio_placeholder()), info))
    }

    /// Vorschau und Eckdaten eines PDFs oder Videos. Das Bild kommt von Quick Look
    /// (`qlmanage -t`, Seite 1 bzw. ein Standbild), die Eckdaten von macos.rs.
    fn media_preview(&self, hash: &str, src: &Path) -> Result<(image::DynamicImage, MediaInfo)> {
        let tmp = self.thumbs_dir().join(format!("ql-{hash}"));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).map_err(err)?;
        let result = (|| -> Result<(image::DynamicImage, MediaInfo)> {
            let mut child = std::process::Command::new("/usr/bin/qlmanage")
                .args(["-t", "-s", &THUMB_SIZE.to_string(), "-o"])
                .arg(&tmp)
                .arg(src)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .map_err(err)?;
            // Quick Look hängt selten, dann nach 30 s abbrechen
            let start = std::time::Instant::now();
            loop {
                if child.try_wait().map_err(err)?.is_some() {
                    break;
                }
                if start.elapsed() > std::time::Duration::from_secs(30) {
                    let _ = child.kill();
                    return Err("Quick Look antwortet nicht".into());
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            let png = fs::read_dir(&tmp)
                .map_err(err)?
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .find(|p| ext_of(p) == "png")
                .ok_or("Keine Vorschau")?;
            let mut img = image::open(&png).map_err(err)?;
            // Quick Look zeichnet PDF-Seiten mit hellem Randpixel: 2 px rundum abschneiden
            if is_pdf(src) && img.width() > 40 && img.height() > 40 {
                img = img.crop_imm(2, 2, img.width() - 4, img.height() - 4);
            }
            #[cfg(target_os = "macos")]
            let mut info = crate::macos::media_info(src);
            #[cfg(not(target_os = "macos"))]
            let mut info = MediaInfo::default();
            // Hochkant gedrehte Videos: Maße an der Vorschau ausrichten
            if let (Some(w), Some(h)) = (info.width, info.height) {
                if (w > h) != (img.width() > img.height()) && img.width() != img.height() {
                    info.width = Some(h);
                    info.height = Some(w);
                }
            } else {
                info.width = Some(img.width() as i64);
                info.height = Some(img.height() as i64);
            }
            Ok((img, info))
        })();
        let _ = fs::remove_dir_all(&tmp);
        result
    }

    // ---- Bildanalyse (Farben hier, Inhalt und Text über Apple Vision) ----

    /// Bilder mit Vorschau, aber noch ohne Analyse: (Hash, Vorschaudatei).
    pub fn missing_analysis(&self) -> Result<Vec<(String, PathBuf)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT t.hash, t.file FROM thumb t
                 LEFT JOIN analysis a ON a.hash = t.hash
                 WHERE a.hash IS NULL",
            )
            .map_err(err)?;
        let thumbs = self.thumbs_dir();
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, thumbs.join(r.get::<_, String>(1)?))))
            .map_err(err)?;
        rows.collect::<std::result::Result<_, _>>().map_err(err)
    }

    /// Die Bilddatei selbst, wenn sie ein Bild ist und in der Bibliothek liegt: Vision
    /// liest daran kleine Schrift und Details, die in der Vorschau fehlen (0.36.3).
    pub fn original_for_hash(&self, hash: &str) -> Option<PathBuf> {
        let rel: String = {
            let conn = self.conn.lock().unwrap();
            conn.query_row("SELECT path FROM item WHERE hash = ?1 ORDER BY path LIMIT 1", params![hash], |r| r.get(0)).optional().ok()??
        };
        let abs = safe_join(&self.root, &rel).ok()?;
        let small_enough = fs::metadata(&abs).map(|m| m.len() <= 60 * 1024 * 1024).unwrap_or(false);
        (is_image(&abs) && THUMB_EXTS.contains(&ext_of(&abs).as_str()) && small_enough).then_some(abs)
    }

    pub fn save_analysis(&self, a: &Analysis) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO analysis (hash, labels, text, colors) VALUES (?1, ?2, ?3, ?4)",
            params![a.hash, a.labels, a.text, a.colors],
        )
        .map_err(err)?;
        Ok(())
    }

    /// Vorschaubild eines Hashs, falls schon gerechnet.
    pub fn thumb_file(&self, hash: &str) -> Option<PathBuf> {
        let conn = self.conn.lock().unwrap();
        let file: Option<String> = conn
            .query_row("SELECT file FROM thumb WHERE hash = ?1", params![hash], |r| r.get(0))
            .optional()
            .ok()?;
        file.map(|f| self.thumbs_dir().join(f)).filter(|p| p.is_file())
    }

    /// Kleines Bild für den Drag-Cursor (max. 96 px), damit beim Ziehen die Zonen sichtbar bleiben.
    pub fn drag_icon(&self, hash: &str, src: &Path) -> Option<String> {
        if is_text(src) {
            return None;
        }
        let out = self.thumbs_dir().join(format!("{hash}-drag.png"));
        if !out.is_file() {
            // Was das image-Crate nicht liest (AVIF, HEIC, manche WebP): Quick Look
            let img = image::open(src).ok().or_else(|| ql_image(src, DRAG_ICON_SIZE, &self.thumbs_dir().join(format!("ql-drag-{hash}"))))?;
            img.thumbnail(DRAG_ICON_SIZE, DRAG_ICON_SIZE).to_rgba8().save(&out).ok()?;
        }
        Some(out.to_string_lossy().into_owned())
    }

    /// Ziehbild einer Textkarte: Fläche in der Kartenfarbe (#rrggbb)
    pub fn color_icon(&self, color: &str) -> Option<String> {
        let hex = color.trim_start_matches('#');
        if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let out = self.thumbs_dir().join(format!("tone-{}.png", hex.to_lowercase()));
        if !out.is_file() {
            let c = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0);
            let (w, h) = (DRAG_ICON_SIZE, DRAG_ICON_SIZE * 3 / 4);
            image::RgbaImage::from_pixel(w, h, image::Rgba([c(0), c(2), c(4), 255])).save(&out).ok()?;
        }
        Some(out.to_string_lossy().into_owned())
    }

    /// Kleines Vorschaubild für die Extension (max. 400 px) als (MIME-Typ, Bytes).
    /// Formate, die wir nicht selbst verkleinern können, gehen bis 700 KB unverändert durch.
    pub fn small_preview(&self, hash: &str, src: &Path) -> Option<(String, Vec<u8>)> {
        // PDF und Video: aus dem Vorschaubild (Seite 1 bzw. Standbild)
        let thumb;
        let src = if is_av(src) {
            thumb = self.thumb_file(hash)?;
            thumb.as_path()
        } else {
            src
        };
        let jpg = self.thumbs_dir().join(format!("{hash}-s.jpg"));
        let png = self.thumbs_dir().join(format!("{hash}-s.png"));
        for (path, mime) in [(&jpg, "image/jpeg"), (&png, "image/png")] {
            if let Ok(bytes) = fs::read(path) {
                return Some((mime.into(), bytes));
            }
        }
        if THUMB_EXTS.contains(&ext_of(src).as_str()) {
            if let Ok(img) = image::open(src) {
                let small = if img.width() > SMALL_THUMB_SIZE || img.height() > SMALL_THUMB_SIZE {
                    img.thumbnail(SMALL_THUMB_SIZE, SMALL_THUMB_SIZE)
                } else {
                    img
                };
                let saved = if small.color().has_alpha() {
                    small.to_rgba8().save(&png).ok().map(|_| (&png, "image/png"))
                } else {
                    fs::File::create(&jpg).ok().and_then(|f| {
                        let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(std::io::BufWriter::new(f), 78);
                        enc.encode_image(&small.to_rgb8()).ok().map(|_| (&jpg, "image/jpeg"))
                    })
                };
                if let Some((path, mime)) = saved {
                    return fs::read(path).ok().map(|b| (mime.into(), b));
                }
            }
        }
        let size = fs::metadata(src).ok()?.len();
        if size > RAW_PREVIEW_MAX_BYTES {
            return None;
        }
        let mime = match ext_of(src).as_str() {
            "svg" => "image/svg+xml",
            "avif" => "image/avif",
            "png" => "image/png",
            "gif" => "image/gif",
            "webp" => "image/webp",
            _ => "image/jpeg",
        };
        fs::read(src).ok().map(|b| (mime.into(), b))
    }

    /// Absoluter Pfad und Hash eines Eintrags (für Vorschau und Finder).
    pub fn locate(&self, rel: &str) -> Result<(PathBuf, String)> {
        let abs = safe_join(&self.root, rel)?;
        let conn = self.conn.lock().unwrap();
        let hash: Option<String> = conn
            .query_row("SELECT hash FROM item WHERE path = ?1", params![rel], |r| r.get(0))
            .optional()
            .map_err(err)?;
        drop(conn);
        let hash = match hash {
            Some(h) => h,
            None => hash_file(&abs)?,
        };
        Ok((abs, hash))
    }

    // ---- Sortieren ----

    /// Verschiebt Dateien in eine Zone. Gibt die neuen relativen Pfade zurück.
    /// `exact`: nur überspringen, was schon genau in diesem Ordner liegt (⌘Z,
    /// Ordnerkarten). Sonst gilt: was schon in der Zone liegt, bleibt, wo es ist.
    pub fn move_items(&self, paths: &[String], zone: &str) -> Result<Vec<String>> {
        self.move_items_to(paths, zone, false)
    }

    pub fn move_items_to(&self, paths: &[String], zone: &str, exact: bool) -> Result<Vec<String>> {
        let target = safe_join(&self.root, zone)?;
        if !target.is_dir() {
            return Err(format!("Zone nicht gefunden: {zone}"));
        }
        let depth = self.structure().zone_depth();
        let mut moved = Vec::new();
        for rel in paths {
            let src = safe_join(&self.root, rel)?;
            let dir = parent_rel(rel);
            let in_zone = !exact && zone_at(zone, depth) == zone && zone_at(&dir, depth) == zone;
            if dir == zone || in_zone || !src.is_file() {
                continue;
            }
            let dst = unique_path(&target, &file_name(rel));
            move_file(&src, &dst)?; // externe Zonen liegen evtl. auf einem anderen Volume
            let new_rel = rel_path(&self.root, &dst);
            let mtime = fs::metadata(&dst).ok().and_then(|m| time_ms(m.modified()));
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "UPDATE item SET path = ?2, mtime = COALESCE(?3, mtime) WHERE path = ?1",
                params![rel, new_rel, mtime],
            )
            .map_err(err)?;
            moved.push(new_rel);
        }
        Ok(moved)
    }

    // ---- Dateinamen (F10, F11, 0.36.0) ----

    /// Plan fürs Umbenennen-Blatt: je Objekt alter Name, neuer Name nach Schema
    /// (leer bei „Original behalten“), Vorschlag aus dem Inhalt und ob der alte
    /// Name uneinheitlich ist. Die Nummer zählt pro Zone nach Ablagedatum.
    pub fn name_plan(&self, paths: &[String], naming: &crate::naming::Naming, library: &str) -> Result<Vec<NamePlan>> {
        let items = self.items()?;
        let mut by_zone: HashMap<&str, Vec<&Item>> = HashMap::new();
        for it in &items {
            by_zone.entry(it.zone.as_str()).or_default().push(it);
        }
        let mut nr: HashMap<&str, usize> = HashMap::new();
        for list in by_zone.values_mut() {
            list.sort_by(|a, b| (a.captured_at.unwrap_or(a.added_at), &a.path).cmp(&(b.captured_at.unwrap_or(b.added_at), &b.path)));
            for (i, it) in list.iter().enumerate() {
                nr.insert(it.path.as_str(), i + 1);
            }
        }
        let wanted: HashSet<&str> = paths.iter().map(|p| p.as_str()).collect();
        let mut out = Vec::new();
        let mut taken: HashSet<String> = HashSet::new();
        for it in items.iter().filter(|i| wanted.contains(i.path.as_str())) {
            let ctx = crate::naming::Ctx { item: it, library, nr: nr.get(it.path.as_str()).copied().unwrap_or(1) };
            let ext = it.filename.rsplit_once('.').map(|(_, e)| e.to_string());
            let mut new = crate::naming::render(naming, &ctx).unwrap_or_default();
            // zwei Objekte im selben Ordner mit gleichem Namen: hinten zählen
            if !new.is_empty() {
                let dir = parent_rel(&it.path);
                let base = new.clone();
                let mut k = 2;
                while !taken.insert(format!("{dir}/{new}")) {
                    new = format!("{base}-{k}");
                    k += 1;
                }
            }
            out.push(NamePlan {
                path: it.path.clone(),
                hash: it.hash.clone(),
                old: it.filename.clone(),
                new: if new.is_empty() { String::new() } else { ext.as_ref().map(|e| format!("{new}.{e}")).unwrap_or(new) },
                suggestion: crate::naming::suggest(it, &naming.lang),
                messy: crate::naming::messy(&it.filename),
            });
        }
        out.sort_by_key(|p| paths.iter().position(|q| q == &p.path).unwrap_or(usize::MAX));
        Ok(out)
    }

    /// Objekte umbenennen: (Pfad, neuer Name ohne oder mit alter Endung). Bleibt
    /// im selben Ordner, die Endung bleibt immer die alte. Gibt (alt, neu) zurück.
    pub fn rename_items(&self, list: &[(String, String)]) -> Result<Vec<(String, String)>> {
        let mut done = Vec::new();
        for (rel, wanted) in list {
            let src = safe_join(&self.root, rel)?;
            if !src.is_file() {
                continue;
            }
            let old = file_name(rel);
            let ext = old.rsplit_once('.').map(|(_, e)| e.to_string());
            let mut stem = crate::naming::clean_stem(wanted);
            if let Some(e) = &ext {
                if let Some(s) = stem.strip_suffix(&format!(".{e}")) {
                    stem = s.trim().to_string();
                }
            }
            if stem.is_empty() {
                continue;
            }
            let name = ext.as_ref().map(|e| format!("{stem}.{e}")).unwrap_or(stem);
            if name == old {
                continue;
            }
            let dir = src.parent().ok_or("Kein Ordner")?.to_path_buf();
            // Nur Groß/klein geändert: auf dem Mac ist das dieselbe Datei
            let dst = if name.to_lowercase() == old.to_lowercase() { dir.join(&name) } else { unique_path(&dir, &name) };
            fs::rename(&src, &dst).map_err(err)?;
            let new_rel = rel_path(&self.root, &dst);
            {
                let conn = self.conn.lock().unwrap();
                conn.execute("UPDATE item SET path = ?2 WHERE path = ?1", params![rel, new_rel]).map_err(err)?;
            }
            done.push((rel.clone(), new_rel));
        }
        if !done.is_empty() {
            self.write_metadata()?;
        }
        Ok(done)
    }

    /// Legt Dateien unter „Zuletzt gelöscht“ ab (.archiver/deleted/<Zeit>/<Pfad>).
    /// Nach 7 Tagen wandern sie in den Papierkorb des Macs (purge_deleted).
    /// Gibt je Datei (Pfad in der Bibliothek, neuer Ort) zurück, für ⌘Z.
    pub fn trash_items(&self, paths: &[String]) -> Result<Vec<(String, String)>> {
        let stamp = self.deleted_stamp()?;
        let mut done = Vec::new();
        let mut hashes = HashMap::new();
        let mut failed: Option<String> = None;
        for rel in paths {
            let abs = safe_join(&self.root, rel)?;
            if !abs.is_file() {
                continue;
            }
            let dst = stamp.join(rel);
            // Scheitert eine Datei (schreibgeschützt), gehen die übrigen trotzdem,
            // und für alle Erledigten werden Hashes und ⌘Z-Liste geschrieben
            let moved = dst.parent().map_or(Ok(()), |d| fs::create_dir_all(d).map_err(err)).and_then(|_| move_file(&abs, &dst));
            if let Err(e) = moved {
                failed.get_or_insert(e);
                continue;
            }
            if let Some(h) = self.hash_of(rel) {
                hashes.insert(rel.clone(), h);
            }
            done.push((rel.clone(), dst.to_string_lossy().into_owned()));
        }
        if done.is_empty() {
            prune_empty(&stamp);
            return match failed {
                Some(e) => Err(e),
                None => Ok(done),
            };
        }
        write_deleted_hashes(&stamp, &hashes)?;
        let conn = self.conn.lock().unwrap();
        for (p, _) in &done {
            conn.execute("DELETE FROM item WHERE path = ?1", params![p]).map_err(err)?;
        }
        drop(conn);
        prune_empty(&stamp);
        Ok(done)
    }

    /// ⌘Z nach dem Löschen: Dateien aus dem Papierkorb zurück an ihren Platz.
    pub fn restore_items(&self, items: &[(String, String)]) -> Result<usize> {
        let mut n = 0;
        for (rel, from) in items {
            // Nur aus „Zuletzt gelöscht“ zurückholen, nie einen beliebigen Pfad
            let Some(src) = self.inside_deleted(from) else { continue };
            let dst = safe_join(&self.root, rel)?;
            if !src.is_file() {
                continue;
            }
            if let Some(dir) = dst.parent() {
                fs::create_dir_all(dir).map_err(err)?;
            }
            let dst = if dst.exists() { unique_path(dst.parent().unwrap_or(&self.root), &file_name(rel)) } else { dst };
            move_file(&src, &dst)?;
            n += 1;
        }
        if n > 0 {
            self.prune_deleted();
            self.scan()?;
        }
        Ok(n)
    }

    pub fn create_zone(&self, name: &str) -> Result<String> {
        let name = clean_segment(name);
        if name.is_empty() {
            return Err("Leerer Name".into());
        }
        let dir = unique_path(&self.root, &name);
        fs::create_dir(&dir).map_err(err)?;
        Ok(rel_path(&self.root, &dir))
    }

    pub fn rename_zone(&self, zone: &str, new_name: &str) -> Result<String> {
        if zone.is_empty() {
            return Err("Der Bibliotheksordner selbst lässt sich hier nicht umbenennen".into());
        }
        let name = clean_segment(new_name);
        if name.is_empty() {
            return Err("Leerer Name".into());
        }
        let src = safe_join(&self.root, zone)?;
        let parent = src.parent().unwrap_or(&self.root).to_path_buf();
        let current = src.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        if current == name {
            return Ok(zone.to_string());
        }
        // Nur Groß-/Kleinschreibung anders: macOS sieht denselben Ordner, also
        // direkt umbenennen statt „Fotos 2“ (0.31.0)
        let dst = if current.to_lowercase() == name.to_lowercase() { parent.join(&name) } else { unique_path(&parent, &name) };
        fs::rename(&src, &dst).map_err(err)?;
        let new_zone = rel_path(&self.root, &dst);
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE item SET path = ?2 || substr(path, length(?1) + 1) WHERE substr(path, 1, length(?1) + 1) = ?1 || '/'",
            params![zone, new_zone],
        )
        .map_err(err)?;
        drop(conn);
        self.rename_item_order(zone, &new_zone)?;
        Ok(new_zone)
    }

    /// Verschiebt einen Unterordner samt Inhalt in einen anderen Ordner (0.28).
    /// Gibt den neuen Pfad zurück; gleichnamige Ziele bekommen eine Nummer.
    pub fn move_folder(&self, from: &str, into: &str) -> Result<String> {
        if from.is_empty() || !from.contains('/') {
            return Err("Nur Unterordner lassen sich verschieben".into());
        }
        if into == from || into.starts_with(&format!("{from}/")) {
            return Err("Ein Ordner kann nicht in sich selbst liegen".into());
        }
        let src = safe_join(&self.root, from)?;
        let parent = safe_join(&self.root, into)?;
        if !src.is_dir() || !parent.is_dir() {
            return Err(format!("Ordner nicht gefunden: {from}"));
        }
        if src.parent() == Some(parent.as_path()) {
            return Ok(from.to_string());
        }
        let name = src.file_name().and_then(|n| n.to_str()).unwrap_or("Ordner").to_string();
        let dst = unique_path(&parent, &name);
        move_file(&src, &dst)?;
        let new_path = rel_path(&self.root, &dst);
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE item SET path = ?2 || substr(path, length(?1) + 1) WHERE substr(path, 1, length(?1) + 1) = ?1 || '/'",
            params![from, new_path],
        )
        .map_err(err)?;
        drop(conn);
        self.rename_item_order(from, &new_path)?;
        Ok(new_path)
    }

    // ---- Externe Zonen (0.30.1) ----
    // Ein Ordner außerhalb der Bibliothek wird Zone über eine Verknüpfung
    // (Symlink) im Bibliotheksordner. Dateien bleiben, wo sie sind; Archiver
    // liest und sichert direkt dort. Lösen entfernt nur die Verknüpfung.

    /// Externe Zonen: (Zone, Ordner draußen, erreichbar). Eine Verknüpfung, deren
    /// Ziel gerade fehlt (Laufwerk ausgeworfen, Netz weg), bleibt Zone und gilt als
    /// offline; ihre Objekte bleiben in der Datenbank, bis das Ziel zurück ist (0.31.0).
    pub fn external_zones(&self) -> Vec<(String, String, bool)> {
        let Ok(rd) = fs::read_dir(&self.root) else { return Vec::new() };
        let mut out: Vec<(String, String, bool)> = rd
            .filter_map(|e| e.ok())
            .filter(|e| is_link(&e.path()) && !is_hidden(&e.file_name()))
            .filter_map(|e| {
                let path = e.path();
                // Verknüpfung auf eine Datei ist keine Zone; fehlendes Ziel schon
                let online = match fs::metadata(&path) {
                    Ok(m) if m.is_dir() => fs::read_dir(&path).is_ok(),
                    Ok(_) => return None,
                    Err(_) => false,
                };
                let target = fs::read_link(&path).ok()?;
                Some((e.file_name().to_string_lossy().into_owned(), target.to_string_lossy().into_owned(), online))
            })
            .collect();
        out.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        out
    }

    /// Ziele der erreichbaren externen Zonen, aufgelöst (für Asset-Freigabe und Watcher)
    pub fn external_targets(&self) -> Vec<PathBuf> {
        self.external_zones()
            .into_iter()
            .filter(|(_, _, online)| *online)
            .filter_map(|(zone, _, _)| self.root.join(zone).canonicalize().ok())
            .collect()
    }

    /// Ordner außerhalb als Zone verknüpfen. Gibt den Zonennamen zurück.
    pub fn link_folder(&self, folder: &Path) -> Result<String> {
        let folder = folder.canonicalize().map_err(err)?;
        let root = self.root.canonicalize().map_err(err)?;
        if !folder.is_dir() {
            return Err("Kein Ordner".into());
        }
        if folder.starts_with(&root) {
            return Err("Der Ordner liegt schon in der Bibliothek".into());
        }
        if root.starts_with(&folder) {
            return Err("Die Bibliothek liegt in diesem Ordner".into());
        }
        let existing = self.external_targets();
        if let Some((zone, _, _)) = self
            .external_zones()
            .into_iter()
            .find(|(z, _, _)| self.root.join(z).canonicalize().ok().as_deref() == Some(folder.as_path()))
        {
            return Ok(zone);
        }
        // Ineinander verschachtelte Ziele führten jedes Objekt doppelt
        if existing.iter().any(|t| folder.starts_with(t) || t.starts_with(&folder)) {
            return Err("zone-overlap".into());
        }
        let name = clean_segment(folder.file_name().and_then(|n| n.to_str()).unwrap_or("Ordner"));
        let name = if name.is_empty() { "Ordner".to_string() } else { name };
        let dst = unique_path(&self.root, &name);
        #[cfg(unix)]
        std::os::unix::fs::symlink(&folder, &dst).map_err(err)?;
        #[cfg(not(unix))]
        return Err("Externe Zonen gibt es nur auf dem Mac".into());
        #[allow(unreachable_code)]
        {
            self.scan()?;
            Ok(rel_path(&self.root, &dst))
        }
    }

    /// Liegt `rel` in einer externen Zone (die Verknüpfung selbst ausgenommen)?
    pub fn in_external(&self, rel: &str) -> bool {
        let mut parts = rel.split('/');
        let (Some(top), Some(_)) = (parts.next(), parts.next()) else { return false };
        is_link(&self.root.join(top))
    }

    /// Verknüpfung lösen: die Zone verschwindet, der Ordner draußen bleibt.
    pub fn unlink_folder(&self, zone: &str) -> Result<()> {
        let link = safe_join(&self.root, zone)?;
        if zone.contains('/') || !is_link(&link) {
            return Err(format!("Keine externe Zone: {zone}"));
        }
        fs::remove_file(&link).map_err(err)?;
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM item WHERE substr(path, 1, length(?1) + 1) = ?1 || '/'", params![zone]).map_err(err)?;
        Ok(())
    }

    /// Dateien in einer Zone, die Archiver nicht zeigt (PSD, AI, ZIP …). Die
    /// Rückfrage beim Löschen nennt sie, denn sie wandern mit.
    pub fn zone_other_files(&self, zone: &str) -> usize {
        let Ok(dir) = safe_join(&self.root, zone) else { return 0 };
        if is_link(&dir) {
            return 0;
        }
        WalkDir::new(&dir)
            .min_depth(1)
            .into_iter()
            .filter_entry(|e| !is_hidden(e.file_name()) && !is_package_dir(e))
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file() && !is_media(e.path()))
            .count()
    }

    /// Legt eine Zone samt Inhalt in den Papierkorb. Inbox und Bibliotheksordner bleiben.
    pub fn trash_zone(&self, zone: &str) -> Result<()> {
        if zone.is_empty() || zone == self.default_zone() {
            return Err("Diese Zone lässt sich nicht löschen".into());
        }
        let dir = safe_join(&self.root, zone)?;
        if !dir.is_dir() {
            return Err(format!("Zone nicht gefunden: {zone}"));
        }
        // Externe Zone: nur die Verknüpfung lösen, der Ordner draußen bleibt unberührt
        if is_link(&dir) {
            return self.unlink_folder(zone);
        }
        // Ordner in einer externen Zone gehören nicht der Bibliothek: nie löschen (0.39.1)
        if self.in_external(zone) {
            return Err("external-folder".into());
        }
        let stamp = self.deleted_stamp()?;
        let hashes: HashMap<String, String> = {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn.prepare("SELECT path, hash FROM item WHERE substr(path, 1, length(?1) + 1) = ?1 || '/'").map_err(err)?;
            let rows = stmt.query_map(params![zone], |r| Ok((r.get(0)?, r.get(1)?))).map_err(err)?;
            rows.filter_map(|r| r.ok()).collect()
        };
        let dst = stamp.join(zone);
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(err)?;
        }
        move_file(&dir, &dst)?;
        write_deleted_hashes(&stamp, &hashes)?;
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM item WHERE substr(path, 1, length(?1) + 1) = ?1 || '/'", params![zone]).map_err(err)?;
        Ok(())
    }

    // ---- Zuletzt gelöscht (0.27) ----

    fn deleted_dir(&self) -> PathBuf {
        self.root.join(META_DIR).join("deleted")
    }

    /// Neuer Ordner für einen Löschvorgang, benannt nach der Zeit in ms
    fn deleted_stamp(&self) -> Result<PathBuf> {
        let base = self.deleted_dir();
        let mut t = now_ms();
        loop {
            let dir = base.join(t.to_string());
            if !dir.exists() {
                fs::create_dir_all(&dir).map_err(err)?;
                return Ok(dir);
            }
            t += 1;
        }
    }

    fn hash_of(&self, rel: &str) -> Option<String> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT hash FROM item WHERE path = ?1", params![rel], |r| r.get(0)).optional().ok().flatten()
    }

    /// Alles unter „Zuletzt gelöscht“, zuletzt Gelöschtes zuerst
    pub fn deleted_items(&self) -> Result<Vec<DeletedItem>> {
        self.purge_deleted(DELETED_DAYS);
        let thumbs = self.thumbs_dir();
        let mut out = Vec::new();
        let Ok(rd) = fs::read_dir(self.deleted_dir()) else { return Ok(out) };
        for stamp in rd.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.is_dir()) {
            let Some(at) = stamp.file_name().and_then(|n| n.to_str()).and_then(|n| n.parse::<i64>().ok()) else { continue };
            let hashes = read_deleted_hashes(&stamp);
            // Auch Nicht-Medien (PSD, AI, ZIP …): Sie kommen mit einer gelöschten
            // Zone hierher und müssen sich ebenso zurückholen lassen.
            for entry in WalkDir::new(&stamp).min_depth(1).into_iter().filter_entry(|e| !is_package_dir(e)).filter_map(|e| e.ok()) {
                if !entry.file_type().is_file() || is_hidden(entry.file_name()) {
                    continue;
                }
                let rel = rel_path(&stamp, entry.path());
                let hash = hashes.get(&rel).cloned().unwrap_or_default();
                let (thumb, w, h, tone) = {
                    let conn = self.conn.lock().unwrap();
                    let thumb: Option<String> = conn.query_row("SELECT file FROM thumb WHERE hash = ?1", params![hash], |r| r.get(0)).optional().ok().flatten();
                    let (w, h): (Option<i64>, Option<i64>) = conn
                        .query_row("SELECT width, height FROM media WHERE hash = ?1", params![hash], |r| Ok((r.get(0)?, r.get(1)?)))
                        .optional()
                        .ok()
                        .flatten()
                        .unwrap_or((None, None));
                    let tone: Option<String> = conn.query_row("SELECT tone FROM note WHERE hash = ?1", params![hash], |r| r.get(0)).optional().ok().flatten();
                    (thumb, w, h, tone)
                };
                let (w, h) = match (w, h) {
                    (Some(w), Some(h)) => (Some(w), Some(h)),
                    _ => match image::image_dimensions(entry.path()) {
                        Ok((w, h)) => (Some(w as i64), Some(h as i64)),
                        Err(_) => (None, None),
                    },
                };
                let format = ext_of(entry.path());
                let quote = if TEXT_EXTS.contains(&format.as_str()) {
                    fs::read_to_string(entry.path()).ok().map(|t| t.trim().chars().take(TEXT_MAX_CHARS).collect())
                } else {
                    None
                };
                out.push(DeletedItem {
                    abs: entry.path().to_string_lossy().into_owned(),
                    zone: zone_at(&parent_rel(&rel), self.structure().zone_depth()),
                    filename: file_name(&rel),
                    path: rel,
                    hash,
                    format,
                    width: w,
                    height: h,
                    thumb: thumb.map(|f| thumbs.join(f).to_string_lossy().into_owned()),
                    quote,
                    tone,
                    deleted_at: at,
                    other: !is_media(entry.path()),
                });
            }
        }
        out.sort_by(|a, b| b.deleted_at.cmp(&a.deleted_at).then_with(|| a.path.cmp(&b.path)));
        Ok(out)
    }

    pub fn deleted_count(&self) -> usize {
        WalkDir::new(self.deleted_dir())
            .min_depth(2)
            .into_iter()
            .filter_entry(|e| !is_package_dir(e))
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file() && !is_hidden(e.file_name()))
            .count()
    }

    /// Gelöschte Dateien (absolute Pfade unter .archiver/deleted) zurück an ihren Platz
    pub fn restore_deleted(&self, files: &[String]) -> Result<usize> {
        let base = self.deleted_dir();
        let mut n = 0;
        for f in files {
            let Some(src) = self.inside_deleted(f) else { continue };
            let Ok(inside) = src.strip_prefix(&base) else { continue };
            let rel = rel_path(Path::new(""), inside);
            let Some((_, rel)) = rel.split_once('/') else { continue };
            if !src.is_file() {
                continue;
            }
            let dst = safe_join(&self.root, rel)?;
            if let Some(dir) = dst.parent() {
                fs::create_dir_all(dir).map_err(err)?;
            }
            let dst = if dst.exists() { unique_path(dst.parent().unwrap_or(&self.root), &file_name(rel)) } else { dst };
            move_file(&src, &dst)?;
            n += 1;
        }
        self.prune_deleted();
        if n > 0 {
            self.scan()?;
        }
        Ok(n)
    }

    /// Pfad aus der Oberfläche, der wirklich unter .archiver/deleted liegt.
    /// `Path::starts_with` allein ließe `deleted/../../x` durch.
    fn inside_deleted(&self, f: &str) -> Option<PathBuf> {
        let p = PathBuf::from(f);
        if p.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return None;
        }
        let base = self.deleted_dir();
        let real = p.canonicalize().ok()?;
        let real_base = base.canonicalize().ok()?;
        if !real.starts_with(&real_base) {
            return None;
        }
        // mit dem Pfad weiterarbeiten, wie ihn die Bibliothek kennt
        Some(base.join(real.strip_prefix(&real_base).ok()?))
    }

    /// Gelöschte Dateien endgültig entfernen: sie gehen in den Papierkorb des Macs
    pub fn remove_deleted(&self, files: &[String]) -> Result<usize> {
        let base = self.deleted_dir();
        let mut n = 0;
        for f in files {
            let Some(p) = self.inside_deleted(f) else { continue };
            if p != base && p.exists() {
                to_trash(&p).map_err(err)?;
                n += 1;
            }
        }
        self.prune_deleted();
        Ok(n)
    }

    /// Löschvorgänge, die älter als `days` Tage sind, wandern in den Papierkorb des Macs
    pub fn purge_deleted(&self, days: i64) -> usize {
        let limit = now_ms() - days * 24 * 3600 * 1000;
        let mut n = 0;
        let Ok(rd) = fs::read_dir(self.deleted_dir()) else { return 0 };
        for stamp in rd.filter_map(|e| e.ok()).map(|e| e.path()) {
            let at = stamp.file_name().and_then(|n| n.to_str()).and_then(|n| n.parse::<i64>().ok());
            if at.map(|t| t < limit).unwrap_or(false) && to_trash(&stamp).is_ok() {
                n += 1;
            }
        }
        n
    }

    /// Unsorted räumt sich selbst auf (0.32): Was länger als `days` Tage in der
    /// Standardzone liegt, kommt unter „Zuletzt gelöscht“ (dort weitere 7 Tage
    /// wiederherstellbar). Gezählt wird ab dem Tag, an dem Archiver das Objekt
    /// dort zuerst gesehen hat (.archiver/unsorted-seen.json), nicht ab Datei-
    /// oder Importdatum: Wer etwas Altes nach Unsorted zieht, hat 7 Tage Zeit,
    /// und beim ersten Start mit dieser Version beginnt die Frist für alles neu.
    /// Geteilte Bibliotheken bleiben unberührt, solange dort nicht ausdrücklich
    /// „An“ gewählt wurde (0.36.1): sonst räumt niemand für andere auf.
    /// Seit wann jedes Objekt in Unsorted liegt (Hash → ms), für „noch n T.“ im Raster (0.33)
    pub fn inbox_seen(&self) -> HashMap<String, i64> {
        self.seen(INBOX_SEEN)
    }

    pub fn shots_seen(&self) -> HashMap<String, i64> {
        self.seen(SHOTS_SEEN)
    }

    fn seen(&self, file: &str) -> HashMap<String, i64> {
        fs::read(self.root.join(META_DIR).join(file)).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
    }

    pub fn expire_inbox(&self, days: i64) -> Result<usize> {
        if self.expire_zone().is_none() {
            return Ok(0);
        }
        self.expire_where(INBOX_SEEN, is_unsorted, days)
    }

    /// Screenshots nach 7 Tagen nach „Zuletzt gelöscht“ (0.47.0), wie Unsorted
    pub fn expire_shots(&self, days: i64) -> Result<usize> {
        if !self.shots_expire() {
            return Ok(0);
        }
        self.expire_where(SHOTS_SEEN, |z| z.split('/').next() == Some(SHOTS), days)
    }

    fn expire_where(&self, file: &str, in_zone: fn(&str) -> bool, days: i64) -> Result<usize> {
        let seen_path = self.root.join(META_DIR).join(file);
        let seen: HashMap<String, i64> = fs::read(&seen_path).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
        let now = now_ms();
        let limit = days * 24 * 3600 * 1000;
        let mut keep = HashMap::new();
        let mut old = Vec::new();
        for item in self.items()?.into_iter().filter(|i| in_zone(&i.zone)) {
            let since = seen.get(&item.hash).copied().unwrap_or(now);
            if now - since > limit {
                old.push(item.path);
            } else {
                keep.insert(item.hash, since);
            }
        }
        let n = if old.is_empty() { 0 } else { self.trash_items(&old)?.len() };
        fs::write(&seen_path, serde_json::to_vec(&keep).map_err(err)?).map_err(err)?;
        Ok(n)
    }

    /// Leere Löschordner aufräumen
    fn prune_deleted(&self) {
        let Ok(rd) = fs::read_dir(self.deleted_dir()) else { return };
        for stamp in rd.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.is_dir()) {
            prune_empty(&stamp);
        }
    }

    // ---- Import ----

    /// Relativer Pfad einer Datei in der Bibliothek: direkt, wenn sie im
    /// Bibliotheksordner liegt, sonst über den Inhalt (gleiches Bild woanders).
    pub fn find_file(&self, src: &Path) -> Option<String> {
        if let Ok(rest) = src.strip_prefix(&self.root) {
            if !rest.as_os_str().is_empty() {
                return Some(rel_path(&self.root, src));
            }
        }
        let hash = hash_file(src).ok()?;
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT path FROM item WHERE hash = ?1 LIMIT 1", params![hash], |r| r.get::<_, String>(0))
            .optional()
            .ok()
            .flatten()
    }

    fn hash_known(&self, hash: &str) -> bool {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT 1 FROM item WHERE hash = ?1 LIMIT 1", params![hash], |_| Ok(()))
            .optional()
            .ok()
            .flatten()
            .is_some()
    }

    /// Quelle eintragen. Vorhandene Angaben werden nur ergänzt, nicht überschrieben.
    pub fn set_source(&self, hash: &str, s: &Source) -> Result<bool> {
        if s.page_url.is_none() && s.image_url.is_none() {
            return Ok(false);
        }
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO source (hash, image_url, page_url, page_title, captured_at, quote) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(hash) DO UPDATE SET
               image_url = COALESCE(source.image_url, ?2),
               page_url = COALESCE(source.page_url, ?3),
               page_title = COALESCE(source.page_title, ?4),
               captured_at = COALESCE(source.captured_at, ?5),
               quote = COALESCE(source.quote, ?6)",
            params![hash, s.image_url, s.page_url, s.page_title, s.captured_at, s.quote],
        )
        .map_err(err)?;
        Ok(true)
    }

    /// Gesicherte Webseite zu einer Linkkarte (0.30.1): .archery/web/<hash>.mhtml
    pub fn web_archive(&self, hash: &str) -> Option<PathBuf> {
        if hash.is_empty() || !hash.chars().all(|c| c.is_ascii_alphanumeric()) {
            return None;
        }
        let p = self.root.join(META_DIR).join("web").join(format!("{hash}.mhtml"));
        p.is_file().then_some(p)
    }

    /// Legt die von der Extension gesicherte Seite (MHTML) zur Karte mit `hash`.
    pub fn attach_web_archive(&self, hash: &str, src: &Path) -> Result<()> {
        if !src.is_file() || hash.is_empty() || !hash.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Ok(());
        }
        let dir = self.root.join(META_DIR).join("web");
        fs::create_dir_all(&dir).map_err(err)?;
        let dst = dir.join(format!("{hash}.mhtml"));
        let _ = fs::remove_file(&dst); // ältere Fassung derselben Seite
        move_file(src, &dst)
    }

    /// Text einer Textkarte ändern (0.30.1, in der Lightbox editierbar).
    /// Das PNG bleibt, wie es gesichert wurde; die App zeigt den Text.
    pub fn set_quote(&self, hash: &str, quote: &str) -> Result<()> {
        let quote: String = quote.trim().chars().take(20_000).collect();
        if quote.is_empty() {
            return Ok(());
        }
        {
            // Auch ohne Quelle (Textkarte aus dem Finder, Sicherung ohne URL):
            // sonst verpuffte die Änderung still (0.31.0)
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO source (hash, quote) VALUES (?1, ?2) ON CONFLICT(hash) DO UPDATE SET quote = ?2",
                params![hash, quote],
            )
            .map_err(err)?;
        }
        self.write_metadata()
    }

    /// Kopiert eine Datei in eine Zone. Bereits vorhandene Bilder (gleicher Hash)
    /// werden nicht doppelt angelegt; ihre Quelle wird trotzdem ergänzt.
    fn import_one(&self, src: &Path, zone: &str, source: Option<&Source>, report: &mut ImportReport) -> Result<()> {
        if !src.is_file() {
            report.missing += 1;
            return Ok(());
        }
        let hash = hash_file(src)?;
        if let Some(s) = source {
            if self.set_source(&hash, s)? {
                report.with_source += 1;
            }
        }
        if self.hash_known(&hash) {
            report.duplicates += 1;
            return Ok(());
        }
        let dir = safe_join(&self.root, zone)?;
        fs::create_dir_all(&dir).map_err(err)?;
        let name = src.file_name().and_then(|n| n.to_str()).unwrap_or("image");
        let dst = unique_path(&dir, name);
        fs::copy(src, &dst).map_err(err)?;
        let meta = fs::metadata(&dst).map_err(err)?;
        let mtime = time_ms(meta.modified()).unwrap_or(0);
        let added = source.and_then(|s| s.captured_at).unwrap_or_else(now_ms);
        let rel = rel_path(&self.root, &dst);
        self.upsert_item(&rel, &hash, meta.len() as i64, mtime, added, &dst)?;
        self.mark_added_by(&hash)?;
        report.imported += 1;
        Ok(())
    }

    /// Übernimmt eine fertig geladene Datei aus der Extension: verschiebt sie in
    /// die Zone und merkt sich die Quelle. Ist das Bild schon in der Bibliothek,
    /// wird die neue Datei verworfen und der vorhandene Pfad zurückgegeben.
    pub fn adopt_file(&self, src: &Path, name: Option<&str>, zone: &str, source: &Source) -> Result<(String, bool)> {
        if !src.is_file() {
            return Err(format!("Datei nicht gefunden: {}", src.display()));
        }
        let hash = hash_file(src)?;
        self.set_source(&hash, source)?;
        let existing: Option<String> = {
            let conn = self.conn.lock().unwrap();
            conn.query_row("SELECT path FROM item WHERE hash = ?1 ORDER BY path LIMIT 1", params![hash], |r| r.get(0))
                .optional()
                .map_err(err)?
        };
        if let Some(path) = existing {
            let known = self.root.join(&path);
            // Ist `src` selbst die bekannte Datei (Transfer-Ordner liegt in einer
            // Zone), darf sie nicht als Duplikat gelöscht werden (0.31.0)
            if known.is_file() && !same_file(&known, src) {
                let _ = fs::remove_file(src);
                self.write_metadata()?;
                return Ok((path, true));
            }
        }
        let dir = safe_join(&self.root, zone)?;
        fs::create_dir_all(&dir).map_err(err)?;
        let name = image_file_name(src, name);
        let dst = unique_path(&dir, &name);
        move_file(src, &dst)?; // anderes Volume: kopieren, dann Original entfernen
        let meta = fs::metadata(&dst).map_err(err)?;
        let mtime = time_ms(meta.modified()).unwrap_or(0);
        let added = source.captured_at.unwrap_or_else(now_ms);
        let rel = rel_path(&self.root, &dst);
        self.upsert_item(&rel, &hash, meta.len() as i64, mtime, added, &dst)?;
        self.mark_added_by(&hash)?;
        self.write_metadata()?;
        Ok((rel, false))
    }

    /// Einzelne Dateien (z. B. aus dem Finder hereingezogen) in eine Zone kopieren.
    /// Dateien und ganze Ordner (0.27) übernehmen. Ein Ordner wird mit seinen
    /// Unterordnern zum Unterordner der Zone, ohne Zone (`""`) zur neuen Zone.
    /// Lose Dateien ohne Zone landen in der Standard-Zone.
    pub fn import_files(&self, files: &[PathBuf], zone: &str) -> Result<ImportReport> {
        let mut report = ImportReport::default();
        let loose = if zone.is_empty() { self.default_zone() } else { zone.to_string() };
        for f in files {
            if f.is_dir() {
                self.import_tree(f, zone, &mut report)?;
            } else if is_media(f) {
                self.import_one(f, &loose, None, &mut report)?;
            }
        }
        if report.imported > 0 && self.user.lock().unwrap().is_some() {
            self.write_metadata()?;
        }
        Ok(report)
    }

    /// ⌘V (0.32): Inhalt der Zwischenablage in eine Zone. Dateien aus dem
    /// Finder gehen wie beim Hineinziehen; sonst Bilddaten (PNG oder TIFF, als
    /// PNG gesichert), sonst Text als Textkarte. Bild und Text bekommen einen
    /// Namen (Text: seine ersten Wörter) und laufen über einen Zwischenordner.
    pub fn paste_into(&self, zone: &str, pasted: Pasted) -> Result<ImportReport> {
        if !pasted.files.is_empty() {
            return self.import_files(&pasted.files, zone);
        }
        let tmp = self.root.join(META_DIR).join("paste");
        fs::create_dir_all(&tmp).map_err(err)?;
        let file = if let Some(bytes) = pasted.image {
            let png = if bytes.starts_with(b"\x89PNG") {
                bytes
            } else {
                let img = image::load_from_memory(&bytes).map_err(err)?;
                let mut out = std::io::Cursor::new(Vec::new());
                img.write_to(&mut out, image::ImageFormat::Png).map_err(err)?;
                out.into_inner()
            };
            let f = tmp.join(format!("{}.png", paste_name("")));
            fs::write(&f, png).map_err(err)?;
            f
        } else if let Some(text) = pasted.text.filter(|t| !t.trim().is_empty()) {
            let text: String = text.trim().chars().take(TEXT_MAX_CHARS).collect();
            // 0.42.5: eingefügter Text wird eine Notiz (.md) statt .txt
            let f = tmp.join(format!("{}.md", paste_name(&text)));
            fs::write(&f, text).map_err(err)?;
            f
        } else {
            return Ok(ImportReport::default());
        };
        let report = self.import_files(&[file.clone()], zone);
        let _ = fs::remove_file(&file);
        report
    }

    /// Ordner aus dem Finder: Struktur bleibt erhalten, Name wird bereinigt.
    fn import_tree(&self, dir: &Path, zone: &str, report: &mut ImportReport) -> Result<()> {
        let name = clean_segment(dir.file_name().and_then(|n| n.to_str()).unwrap_or("Ordner"));
        if name.is_empty() {
            return Ok(());
        }
        let dest = if zone.is_empty() { name } else { format!("{zone}/{name}") };
        fs::create_dir_all(safe_join(&self.root, &dest)?).map_err(err)?;
        let st = self.structure();
        let walker = WalkDir::new(dir)
            .min_depth(1)
            .into_iter()
            .filter_entry(|e| !is_hidden(e.file_name()) && !(e.file_type().is_dir() && st.skips(&e.file_name().to_string_lossy())));
        for entry in walker.filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() || !is_media(entry.path()) {
                continue;
            }
            let sub = parent_rel(&rel_path(dir, entry.path()));
            let target = if sub.is_empty() { dest.clone() } else { format!("{dest}/{}", sub.split('/').map(clean_segment).collect::<Vec<_>>().join("/")) };
            self.import_one(entry.path(), &target, None, report)?;
        }
        Ok(())
    }

    /// Neuer Unterordner in einer Zone (bzw. in einem ihrer Ordner)
    pub fn create_folder(&self, parent: &str, name: &str) -> Result<String> {
        let name = clean_segment(name);
        if parent.is_empty() || name.is_empty() {
            return Err("Leerer Name".into());
        }
        let base = safe_join(&self.root, parent)?;
        if !base.is_dir() {
            return Err(format!("Zone nicht gefunden: {parent}"));
        }
        let dir = unique_path(&base, &name);
        fs::create_dir(&dir).map_err(err)?;
        Ok(rel_path(&self.root, &dir))
    }

    // ---- Notiz-Objekte (0.29): eigene Texte als .md in der Zone ----

    /// Neue Notiz in einer Zone (bzw. einem ihrer Ordner). Gibt den relativen Pfad zurück.
    pub fn create_note(&self, dir: &str, title: &str) -> Result<String> {
        let base = safe_join(&self.root, dir)?;
        if !base.is_dir() {
            return Err(format!("Zone nicht gefunden: {dir}"));
        }
        let title = title.trim();
        let name = match clean_segment(title) {
            n if n.is_empty() => "Notiz".to_string(),
            n => n,
        };
        let path = unique_path(&base, &format!("{name}.md"));
        fs::write(&path, format!("# {}\n", if title.is_empty() { &name } else { title })).map_err(err)?;
        let rel = rel_path(&self.root, &path);
        let hash = self.record_note(&rel, &path)?;
        self.mark_added_by(&hash)?;
        Ok(rel)
    }

    /// Ganzer Text einer Notiz (ohne die Kürzung der Kartenvorschau)
    pub fn read_note(&self, rel: &str) -> Result<String> {
        let abs = safe_join(&self.root, rel)?;
        if !is_note(&abs) {
            return Err("Keine Notiz".into());
        }
        fs::read_to_string(abs).map_err(err)
    }

    /// Text einer Notiz speichern. Mit `rename` bekommt die Datei den Titel
    /// (erste Zeile) als Namen. Der Inhalt ändert den Hash: Farbe, Verknüpfungen
    /// und Reihenfolge wandern zum neuen Hash mit. Gibt (Pfad, Hash) zurück.
    pub fn write_note(&self, rel: &str, text: &str, rename: bool) -> Result<(String, String)> {
        let abs = safe_join(&self.root, rel)?;
        if !is_note(&abs) || !abs.is_file() {
            return Err("Keine Notiz".into());
        }
        let old: Option<String> = {
            let conn = self.conn.lock().unwrap();
            conn.query_row("SELECT hash FROM item WHERE path = ?1", params![rel], |r| r.get(0)).optional().map_err(err)?
        };
        let body = text.trim_end();
        fs::write(&abs, if body.trim().is_empty() { String::new() } else { format!("{body}\n") }).map_err(err)?;
        let mut target = abs.clone();
        let name = clean_segment(&note_title(text));
        let stem = abs.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
        // „Titel 2“ bleibt, wenn „Titel“ schon vergeben war
        let same = stem == name || stem.strip_prefix(&format!("{name} ")).map(|n| n.parse::<u32>().is_ok()).unwrap_or(false);
        if rename && !name.is_empty() && !same {
            let dir = abs.parent().unwrap_or(&self.root);
            let file = format!("{name}.md");
            // Nur Groß-/Kleinschreibung anders: direkt umbenennen (macOS sieht dieselbe Datei)
            target = if stem.to_lowercase() == name.to_lowercase() { dir.join(&file) } else { unique_path(dir, &file) };
            fs::rename(&abs, &target).map_err(err)?;
        }
        let new_rel = rel_path(&self.root, &target);
        let hash = self.record_note_at(rel, &new_rel, &target)?;
        if let Some(old) = old.filter(|o| *o != hash) {
            self.move_note_hash(&old, &hash)?;
        }
        Ok((new_rel, hash))
    }

    fn record_note(&self, rel: &str, abs: &Path) -> Result<String> {
        self.record_note_at(rel, rel, abs)
    }

    /// Eintrag einer Notiz nachführen (wie der Scan, aber ohne auf ihn zu warten)
    fn record_note_at(&self, old_rel: &str, rel: &str, abs: &Path) -> Result<String> {
        let meta = fs::metadata(abs).map_err(err)?;
        let hash = hash_file(abs)?;
        let mtime = time_ms(meta.modified()).unwrap_or(0);
        let updated = {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "UPDATE item SET path = ?1, hash = ?2, bytes = ?3, mtime = ?4 WHERE path = ?5",
                params![rel, hash, meta.len() as i64, mtime, old_rel],
            )
            .map_err(err)?
        };
        if updated == 0 {
            self.upsert_item(rel, &hash, meta.len() as i64, mtime, now_ms(), abs)?;
        }
        Ok(hash)
    }

    /// Angaben einer Notiz vom alten zum neuen Hash: Notizzeile, Verweise aus
    /// anderen Notizen und die eigene Reihenfolge. Teilt sich ein anderes
    /// Objekt den alten Hash noch, wird nur kopiert.
    fn move_note_hash(&self, old: &str, new: &str) -> Result<()> {
        let still_used = {
            let conn = self.conn.lock().unwrap();
            let n: i64 = conn.query_row("SELECT COUNT(*) FROM item WHERE hash = ?1", params![old], |r| r.get(0)).map_err(err)?;
            conn.execute(
                "INSERT OR IGNORE INTO note (hash, note, credit, license, internal, added_by, updated_at, tone, caption, links, mark)
                 SELECT ?2, note, credit, license, internal, added_by, ?3, tone, caption, links, mark FROM note WHERE hash = ?1",
                params![old, new, now_ms()],
            )
            .map_err(err)?;
            // Herkunft (Seite, Titel) wandert mit; der Text steht jetzt in der Datei
            conn.execute(
                "INSERT OR IGNORE INTO source (hash, image_url, page_url, page_title, captured_at, quote)
                 SELECT ?2, image_url, page_url, page_title, captured_at, NULL FROM source WHERE hash = ?1",
                params![old, new],
            )
            .map_err(err)?;
            if n == 0 {
                conn.execute("DELETE FROM note WHERE hash = ?1", params![old]).map_err(err)?;
                conn.execute("DELETE FROM source WHERE hash = ?1", params![old]).map_err(err)?;
                // Verweise anderer Notizen umschreiben
                let mut stmt = conn.prepare("SELECT hash, links FROM note WHERE links LIKE ?1").map_err(err)?;
                let rows: Vec<(String, Option<String>)> = stmt
                    .query_map(params![format!("%{old}%")], |r| Ok((r.get(0)?, r.get(1)?)))
                    .map_err(err)?
                    .collect::<std::result::Result<_, _>>()
                    .map_err(err)?;
                for (h, links) in rows {
                    let list: Vec<String> = crate::studio::links_of(links)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|l| if l == old { new.to_string() } else { l })
                        .collect();
                    conn.execute(
                        "UPDATE note SET links = ?2, updated_at = MAX(updated_at + 1, ?3) WHERE hash = ?1",
                        params![h, crate::studio::links_json(&Some(list)), now_ms()],
                    )
                    .map_err(err)?;
                }
            }
            n > 0
        };
        if !still_used {
            self.replace_in_order(old, new)?;
        }
        self.write_metadata()
    }

    /// Alle Ordner unterhalb der Zonen (auch leere), für Ordnerkarten und Chips
    pub fn folders(&self) -> Vec<String> {
        let st = self.structure();
        let root = self.root.clone();
        // Bibliothek ab Tiefe zone_depth + 1; externe Zonen beginnen eine Ebene tiefer
        let starts = std::iter::once((self.root.clone(), 0)).chain(
            self.external_zones().into_iter().filter(|(_, _, on)| *on).map(|(zone, _, _)| (self.root.join(zone), 1)),
        );
        let mut out = Vec::new();
        for (start, offset) in starts {
            out.extend(
                WalkDir::new(&start)
                    .min_depth(st.zone_depth() + 1 - offset)
                    .max_depth(st.zone_depth() + 4 - offset)
                    .into_iter()
                    // Unterhalb von min_depth prüft WalkDir den Filter nicht: .archiver/thumbs
                    // kam sonst als Ordner mit, deshalb den ganzen Pfad prüfen (0.31.0)
                    .filter_entry(|e| {
                        let rel = rel_path(&root, e.path());
                        !rel.split('/').any(|s| s.starts_with('.')) && !(e.file_type().is_dir() && st.skips(&rel))
                    })
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_dir())
                    .map(|e| rel_path(&self.root, e.path())),
            );
        }
        out
    }

    /// Einen ganzen Ordner übernehmen: Unterordner werden zu Zonen, lose Bilder landen in der Inbox.
    pub fn import_folder(&self, dir: &Path) -> Result<ImportReport> {
        let mut report = ImportReport::default();
        let walker = WalkDir::new(dir).min_depth(1).into_iter().filter_entry(|e| !is_hidden(e.file_name()));
        for entry in walker.filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() || !is_media(entry.path()) {
                continue;
            }
            let rel = rel_path(dir, entry.path());
            let zone = match parent_rel(&rel) {
                p if p.is_empty() => self.default_zone(),
                p => p.split('/').map(clean_segment).collect::<Vec<_>>().join("/"),
            };
            self.import_one(entry.path(), &zone, None, &mut report)?;
        }
        self.write_metadata()?;
        Ok(report)
    }

    /// Import aus der Extension: die Datei archery-export.json (Einstellungen →
    /// „Für die App exportieren“) kennt Zone, Speicherort und Quelle jedes Bildes.
    pub fn import_extension_export(&self, json_path: &Path) -> Result<ImportReport> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ExportItem {
            zone: String,
            file: String,
            url: Option<String>,
            page_url: Option<String>,
            page_title: Option<String>,
            ts: Option<i64>,
        }
        #[derive(Deserialize)]
        struct Export {
            #[serde(default)]
            zones: Vec<String>,
            items: Vec<ExportItem>,
        }
        let text = fs::read_to_string(json_path).map_err(err)?;
        let export: Export = serde_json::from_str(&text).map_err(|e| format!("Keine gültige Export-Datei: {e}"))?;
        let default = self.default_zone();
        let zone_for = |z: &str| -> String {
            let cleaned = z.split('/').map(clean_segment).filter(|s| !s.is_empty()).collect::<Vec<_>>().join("/");
            if cleaned.is_empty() { default.clone() } else { cleaned }
        };
        for z in &export.zones {
            let _ = fs::create_dir_all(safe_join(&self.root, &zone_for(z))?);
        }
        let mut report = ImportReport::default();
        for it in &export.items {
            let source = Source {
                image_url: it.url.clone(),
                page_url: it.page_url.clone(),
                page_title: it.page_title.clone(),
                captured_at: it.ts,
                quote: None,
            };
            self.import_one(Path::new(&it.file), &zone_for(&it.zone), Some(&source), &mut report)?;
        }
        self.write_metadata()?;
        Ok(report)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn audio_sniff_and_placeholder() {
        let mut wav = b"RIFF\0\0\0\0WAVEfmt ".to_vec();
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&[0; 16]);
        wav.extend_from_slice(b"data");
        let samples: Vec<i16> = vec![0, 1000, -2000, 500, 16000, -100, 0, 8000];
        wav.extend_from_slice(&((samples.len() * 2) as u32).to_le_bytes());
        for v in &samples {
            wav.extend_from_slice(&v.to_le_bytes());
        }
        assert_eq!(sniff_ext(&wav), Some("wav"));
        assert_eq!(sniff_ext(b"FORM\0\0\0\0AIFFCOMM"), Some("aif"));
        assert_eq!(sniff_ext(b"ID3\x04\0\0\0\0\0\0\0\0"), Some("mp3"));
        assert_eq!(sniff_ext(b"fLaC\0\0\0\x22"), Some("flac"));
        assert_eq!(sniff_ext(b"\0\0\0\x20ftypM4A \0\0"), Some("m4a"));
        assert_eq!(sniff_ext(b"\0\0\0\x20ftypisom\0\0"), Some("mp4"));
        let img = audio_placeholder();
        assert_eq!((img.width(), img.height()), (600, 600));
        assert_eq!(img.get_pixel(0, 0).0, [42, 42, 42]);
        assert_eq!(img.get_pixel(300, 300).0, [140, 140, 140]);
        assert!(is_media(Path::new("a/Song.AIFF")) && is_audio(Path::new("b.flac")));
    }

    #[test]
    fn open_renames_legacy_meta_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join(".archiver")).unwrap();
        fs::write(root.join(".archiver/metadata.json"), "{}").unwrap();
        Library::open(root).unwrap();
        assert!(!root.join(".archiver").exists());
        assert_eq!(fs::read_to_string(root.join(META_DIR).join("metadata.json")).unwrap(), "{}");
    }

    #[test]
    fn color_classes() {
        assert_eq!(color_name(220, 30, 40), "red");
        assert_eq!(color_name(30, 80, 220), "blue");
        assert_eq!(color_name(10, 10, 10), "black");
        assert_eq!(color_name(245, 245, 245), "white");
        assert_eq!(color_name(128, 128, 128), "gray");
        assert_eq!(color_name(120, 70, 30), "brown");
        assert_eq!(color_name(240, 200, 60), "yellow");
    }

    use super::*;

    fn write_png(path: &Path, w: u32, h: u32, shade: u8) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        image::RgbImage::from_pixel(w, h, image::Rgb([shade, 0, 0])).save(path).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn external_zones_offline_subzones_and_overlap() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        let outside = tmp.path().join("Projekt");
        write_png(&root.join("Typo/a.png"), 4, 4, 10);
        write_png(&outside.join("x.png"), 4, 4, 30);
        write_png(&outside.join("Entwurf/y.png"), 4, 4, 40);
        let lib = Library::open(&root).unwrap();
        lib.scan().unwrap();
        let zone = lib.link_folder(&outside).unwrap();
        assert_eq!(zone, "Projekt");
        let paths = |lib: &Library| {
            let mut p: Vec<String> = lib.items().unwrap().into_iter().map(|i| i.path).collect();
            p.sort();
            p
        };
        assert_eq!(paths(&lib), ["Projekt/Entwurf/y.png", "Projekt/x.png", "Typo/a.png"]);
        assert!(lib.folders().contains(&"Projekt/Entwurf".to_string()), "Unterordner der externen Zone");
        assert_eq!(lib.external_targets(), vec![outside.canonicalize().unwrap()]);

        // Verschachtelte Ziele führten Objekte doppelt
        assert_eq!(lib.link_folder(&outside.join("Entwurf")).unwrap_err(), "zone-overlap");
        // Derselbe Ordner noch einmal: dieselbe Zone, keine zweite
        assert_eq!(lib.link_folder(&outside).unwrap(), "Projekt");

        // Unterzonen: „Projekt/Entwurf“ ist eine Zone, „Projekt“ bleibt sichtbar
        lib.set_structure(&Structure { depth: 2, skip: vec![], subzones: true }).unwrap();
        let zones: Vec<String> = lib.zones().unwrap().into_iter().map(|z| z.path).collect();
        assert!(zones.contains(&"Projekt/Entwurf".to_string()), "{zones:?}");
        lib.set_structure(&Structure { depth: 2, skip: vec![], subzones: false }).unwrap();

        // Laufwerk weg: die Zone bleibt (offline), ihre Objekte bleiben in der Datenbank
        // Gespeichert wird das echte Ziel (auf dem Mac liegt /var unter /private/var)
        let target = outside.canonicalize().unwrap().to_string_lossy().into_owned();
        let parked = tmp.path().join("Projekt-weg");
        fs::rename(&outside, &parked).unwrap();
        assert_eq!(lib.external_zones(), vec![("Projekt".to_string(), target, false)]);
        lib.scan().unwrap();
        assert_eq!(paths(&lib).len(), 3, "offline heißt nicht gelöscht");
        let zones: Vec<String> = lib.zones().unwrap().into_iter().map(|z| z.path).collect();
        assert!(zones.contains(&"Projekt".to_string()), "{zones:?}");
        // zurück: alles wieder da, ohne neu zu hashen
        fs::rename(&parked, &outside).unwrap();
        assert!(!lib.scan().unwrap(), "nichts geändert");
        assert_eq!(paths(&lib).len(), 3);

        // Ordner in der externen Zone löscht Archiver nie, die Zone selbst wird nur getrennt (0.39.1)
        assert!(lib.zones().unwrap().iter().any(|z| z.path == "Projekt" && z.external));
        assert!(lib.in_external("Projekt/Entwurf"));
        assert!(!lib.in_external("Projekt") && !lib.in_external("Typo/x"));
        assert_eq!(lib.trash_zone("Projekt/Entwurf").unwrap_err(), "external-folder");
        assert!(outside.join("Entwurf/y.png").exists());

        // Lösen entfernt nur die Verknüpfung
        lib.unlink_folder("Projekt").unwrap();
        assert!(outside.join("x.png").exists());
        assert_eq!(paths(&lib), ["Typo/a.png"]);
    }

    #[cfg(unix)]
    #[test]
    fn unreadable_folder_is_not_deleted() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        write_png(&root.join("Typo/a.png"), 4, 4, 10);
        write_png(&root.join("Geheim/b.png"), 4, 4, 20);
        let lib = Library::open(&root).unwrap();
        lib.scan().unwrap();
        assert_eq!(lib.items().unwrap().len(), 2);
        let secret = root.join("Geheim");
        fs::set_permissions(&secret, fs::Permissions::from_mode(0o000)).unwrap();
        let readable = fs::read_dir(&secret).is_ok(); // als root lässt sich alles lesen
        lib.scan().unwrap();
        fs::set_permissions(&secret, fs::Permissions::from_mode(0o755)).unwrap();
        if !readable {
            assert_eq!(lib.items().unwrap().len(), 2, "ohne Leserecht heißt nicht gelöscht");
        }
        // Bibliotheksordner fehlt ganz: Fehler statt leerer Bibliothek
        let parked = tmp.path().join("Bib-weg");
        fs::rename(&root, &parked).unwrap();
        assert!(lib.scan().is_err());
        fs::rename(&parked, &root).unwrap();
        assert_eq!(lib.items().unwrap().len(), 2);
    }

    #[test]
    fn copy_keeps_original_until_complete() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("Zone");
        write_png(&src.join("a.png"), 4, 4, 10);
        write_png(&src.join("Sub/b.png"), 4, 4, 20);
        #[cfg(unix)]
        std::os::unix::fs::symlink("/nirgends", src.join("link")).unwrap();
        let dst = tmp.path().join("Ziel");
        copy_dir_then_remove(&src, &dst).unwrap();
        assert!(dst.join("a.png").is_file() && dst.join("Sub/b.png").is_file());
        #[cfg(unix)]
        assert!(is_link(&dst.join("link")), "Verknüpfung bleibt Verknüpfung");
        assert!(!src.exists());
        // Ziel nicht beschreibbar (existiert als Datei): nichts gelöscht, keine Reste
        let src2 = tmp.path().join("Zwei");
        write_png(&src2.join("c.png"), 4, 4, 30);
        let blocked = tmp.path().join("Blockiert");
        fs::write(&blocked, b"x").unwrap();
        assert!(copy_dir_then_remove(&src2, &blocked.join("Ziel")).is_err());
        assert!(src2.join("c.png").is_file(), "Original bleibt");
        // Einzeldatei: keine Teildatei bleibt liegen
        let f = tmp.path().join("d.png");
        write_png(&f, 4, 4, 40);
        copy_file(&f, &tmp.path().join("Ziel/d.png")).unwrap();
        assert!(fs::read_dir(tmp.path().join("Ziel")).unwrap().all(|e| !e.unwrap().file_name().to_string_lossy().ends_with(".archery-part")));
    }

    #[test]
    fn prefix_match_is_literal() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        write_png(&root.join("A_1/x.png"), 4, 4, 10);
        write_png(&root.join("A 1/y.png"), 4, 4, 20);
        write_png(&root.join("a_1b/z.png"), 4, 4, 30);
        let lib = Library::open(&root).unwrap();
        lib.scan().unwrap();
        let new = lib.rename_zone("A_1", "Neu").unwrap();
        let mut paths: Vec<String> = lib.items().unwrap().into_iter().map(|i| i.path).collect();
        paths.sort();
        assert_eq!(paths, ["A 1/y.png", "Neu/x.png", "a_1b/z.png"], "nur die umbenannte Zone ändert sich");
        lib.trash_zone(&new).unwrap();
        assert_eq!(lib.items().unwrap().len(), 2);
    }

    #[test]
    fn quote_without_source_and_adopt_of_known_file() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        write_png(&root.join("Typo/a.png"), 4, 4, 10);
        let lib = Library::open(&root).unwrap();
        lib.scan().unwrap();
        let (_, hash) = lib.locate("Typo/a.png").unwrap();
        lib.set_quote(&hash, "Neuer Text").unwrap();
        let item = lib.items().unwrap().into_iter().next().unwrap();
        assert_eq!(item.quote.as_deref(), Some("Neuer Text"), "ohne Quelle nicht verpufft");

        // Transfer-Ordner liegt in einer Zone und ist schon indiziert: nicht löschen
        let transfer = root.join("Typo").join(TRANSFER_DIR);
        write_png(&transfer.join("b.png"), 4, 4, 50);
        lib.scan().unwrap();
        let (path, duplicate) = lib.adopt_file(&transfer.join("b.png"), None, "Typo", &Source::default()).unwrap();
        assert!(root.join(&path).is_file(), "einzige Kopie bleibt erhalten: {path} {duplicate}");
    }

    #[test]
    fn broken_database_is_rebuilt() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        write_png(&root.join("Typo/a.png"), 4, 4, 10);
        let hash = {
            let lib = Library::open(&root).unwrap();
            lib.scan().unwrap();
            let hash = lib.items().unwrap()[0].hash.clone();
            lib.set_quote(&hash, "bleibt").unwrap();
            hash
        };
        let db = root.join(META_DIR).join("library.db");
        let _ = fs::remove_file(db.with_extension("db-wal"));
        let _ = fs::remove_file(db.with_extension("db-shm"));
        fs::write(&db, b"kein SQLite, sondern Datenmuell".repeat(200)).unwrap();
        let lib = Library::open(&root).expect("öffnet trotz kaputter Datenbank");
        lib.scan().unwrap();
        let items = lib.items().unwrap();
        assert_eq!(items.len(), 1, "Bestand neu eingelesen");
        assert_eq!(items[0].hash, hash);
        let aside = fs::read_dir(root.join(META_DIR)).unwrap().filter_map(|e| e.ok()).any(|e| e.file_name().to_string_lossy().starts_with("library.db.broken-"));
        assert!(aside, "alte Datei liegt daneben");
    }

    #[test]
    fn deleted_paths_stay_inside() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        write_png(&root.join("Typo/a.png"), 4, 4, 10);
        let outside = tmp.path().join("privat.png");
        write_png(&outside, 4, 4, 99);
        let lib = Library::open(&root).unwrap();
        lib.scan().unwrap();
        lib.trash_items(&["Typo/a.png".into()]).unwrap();
        let sneaky = format!("{}/../../../privat.png", lib.deleted_dir().display());
        assert_eq!(lib.remove_deleted(&[sneaky.clone()]).unwrap(), 0);
        assert_eq!(lib.restore_deleted(&[sneaky.clone()]).unwrap(), 0);
        assert_eq!(lib.restore_items(&[("Typo/p.png".into(), outside.to_string_lossy().into_owned())]).unwrap(), 0);
        assert!(outside.is_file(), "Datei außerhalb unberührt");
    }

    #[test]
    fn deleted_restore_purge_and_folders() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        write_png(&root.join("Typo/a.png"), 4, 4, 10);
        write_png(&root.join("Typo/Serif/b.png"), 4, 4, 20);
        let lib = Library::open(&root).unwrap();
        lib.scan().unwrap();
        // Ordner, auch leere
        lib.create_folder("Typo", "Grotesk").unwrap();
        let folders = lib.folders();
        assert!(folders.contains(&"Typo/Serif".to_string()) && folders.contains(&"Typo/Grotesk".to_string()));
        // in einen Unterordner verschieben und genau zurück (⌘Z)
        assert_eq!(lib.move_items_to(&["Typo/a.png".into()], "Typo/Grotesk", true).unwrap(), vec!["Typo/Grotesk/a.png".to_string()]);
        assert!(lib.move_items(&["Typo/Grotesk/a.png".into()], "Typo").unwrap().is_empty(), "in der Zone bleibt es");
        assert_eq!(lib.move_items_to(&["Typo/Grotesk/a.png".into()], "Typo", true).unwrap(), vec!["Typo/a.png".to_string()]);
        // Löschen: unter „Zuletzt gelöscht“, nicht mehr in der Bibliothek
        let done = lib.trash_items(&["Typo/Serif/b.png".into()]).unwrap();
        assert_eq!(done.len(), 1);
        assert!(!root.join("Typo/Serif/b.png").exists());
        assert_eq!(lib.items().unwrap().len(), 1);
        let del = lib.deleted_items().unwrap();
        assert_eq!(del.len(), 1);
        assert_eq!(del[0].path, "Typo/Serif/b.png");
        assert!(!del[0].hash.is_empty());
        assert_eq!(lib.deleted_count(), 1);
        // Wiederherstellen
        assert_eq!(lib.restore_deleted(&[del[0].abs.clone()]).unwrap(), 1);
        assert!(root.join("Typo/Serif/b.png").is_file());
        assert_eq!(lib.deleted_count(), 0);
        assert_eq!(fs::read_dir(lib.deleted_dir()).unwrap().count(), 0, "leere Löschordner sind weg");
        // Zone löschen und ⌘Z über restore_items
        let done = lib.trash_items(&["Typo/a.png".into()]).unwrap();
        lib.restore_items(&done).unwrap();
        assert!(root.join("Typo/a.png").is_file());
        fs::write(root.join("Typo/layout.psd"), b"8BPS").unwrap();
        assert_eq!(lib.zone_other_files("Typo"), 1);
        lib.trash_zone("Typo").unwrap();
        assert!(!root.join("Typo").exists());
        // Die PSD ist sichtbar und lässt sich zurückholen (BC2)
        let del = lib.deleted_items().unwrap();
        assert_eq!(del.len(), 3);
        assert_eq!(lib.deleted_count(), 3);
        let psd = del.iter().find(|d| d.other).unwrap();
        assert_eq!(psd.filename, "layout.psd");
        assert_eq!(lib.restore_deleted(&[psd.abs.clone()]).unwrap(), 1);
        assert!(root.join("Typo/layout.psd").is_file());
        assert_eq!(lib.deleted_items().unwrap().len(), 2);
        // Alt genug: wandert aus „Zuletzt gelöscht“ (in den Papierkorb des Macs)
        let stamp = fs::read_dir(lib.deleted_dir()).unwrap().next().unwrap().unwrap().path();
        fs::rename(&stamp, lib.deleted_dir().join("1000")).unwrap();
        let _ = lib.purge_deleted(DELETED_DAYS);
        // Ordner aus dem Finder: wird zur Zone bzw. zum Unterordner
        let ext = tmp.path().join("Extern/Projekt X");
        write_png(&ext.join("c.png"), 4, 4, 30);
        write_png(&ext.join("Sub/d.png"), 4, 4, 40);
        let r = lib.import_files(&[ext.clone()], "").unwrap();
        assert_eq!(r.imported, 2);
        assert!(root.join("Projekt X/Sub/d.png").is_file());
        write_png(&ext.join("e.png"), 4, 4, 50);
        lib.import_files(&[ext], "Unsorted").unwrap();
        assert!(root.join("Unsorted/Projekt X/e.png").is_file());
        // Unterordner in einen anderen Ordner ziehen (0.28)
        assert_eq!(lib.move_folder("Projekt X/Sub", "Unsorted/Projekt X").unwrap(), "Unsorted/Projekt X/Sub");
        assert!(root.join("Unsorted/Projekt X/Sub/d.png").is_file());
        assert!(lib.items().unwrap().iter().any(|i| i.path == "Unsorted/Projekt X/Sub/d.png"));
        assert!(lib.move_folder("Unsorted/Projekt X", "Unsorted/Projekt X/Sub").is_err(), "nicht in sich selbst");
    }

    #[test]
    fn older_analyses_run_again_once() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        {
            let lib = Library::open(&root).unwrap();
            for (hash, labels) in [("alt", "poster, art"), ("neu", "poster:0.92, art:0.41"), ("leer", "")] {
                let a = Analysis { hash: hash.into(), labels: labels.into(), text: String::new(), colors: String::new() };
                lib.save_analysis(&a).unwrap();
            }
            lib.conn.lock().unwrap().execute_batch("PRAGMA user_version = 2;").unwrap();
        }
        let lib = Library::open(&root).unwrap();
        let conn = lib.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT hash FROM analysis ORDER BY hash").unwrap();
        let left: Vec<String> = stmt.query_map([], |r| r.get(0)).unwrap().map(|r| r.unwrap()).collect();
        assert!(left.is_empty(), "vor 0.31.2 ohne Gesichter und Personen: alles neu, {left:?}");
        drop(stmt);
        drop(conn);
        // danach bleibt eine neue Analyse beim Öffnen stehen
        let a = Analysis { hash: "x".into(), labels: "people:0.91".into(), text: String::new(), colors: String::new() };
        lib.save_analysis(&a).unwrap();
        drop(lib);
        let lib = Library::open(&root).unwrap();
        let n: i64 = lib.conn.lock().unwrap().query_row("SELECT COUNT(*) FROM analysis", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn expire_zone_is_a_setting() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        let lib = Library::open(&root).unwrap();
        assert_eq!(lib.expire_zone().as_deref(), Some(INBOX), "ab Werk Unsorted");
        std::fs::create_dir_all(root.join("Downloads")).unwrap();
        assert!(lib.set_expire_zone("Downloads").is_err(), "nur Unsorted");
        lib.set_default_zone("Downloads").unwrap();
        assert_eq!(lib.expire_zone().as_deref(), Some(INBOX), "auch mit anderer Standard-Zone");
        let mut v = lib.zone_file();
        v["expire"] = serde_json::json!("Downloads");
        lib.write_zone_file(&v).unwrap();
        assert_eq!(lib.expire_zone().as_deref(), Some(INBOX), "alte Wahl gilt als Unsorted");
        lib.set_expire_zone(INBOX).unwrap();
        assert_eq!(lib.expire_zone().as_deref(), Some(INBOX));
        lib.set_expire_zone("").unwrap();
        assert_eq!(lib.expire_zone(), None);
        assert_eq!(lib.expire_inbox(DELETED_DAYS).unwrap(), 0);
        assert!(lib.set_expire_zone("Gibtsnicht").is_err());
    }

    #[test]
    fn screenshots_empty_after_seven_days_only_once_chosen() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        let lib = Library::open(&root).unwrap();
        let src = tmp.path().join("src");
        write_png(&src.join("shot.png"), 4, 4, 10);
        write_png(&src.join("typo.png"), 4, 4, 20);
        lib.import_files(&[src.join("shot.png")], SHOTS).unwrap();
        lib.import_files(&[src.join("typo.png")], "Typo").unwrap();
        // eigener Ordner gleichen Namens: ab Werk unberührt
        assert!(!lib.shots_expire());
        assert_eq!(lib.expire_shots(DELETED_DAYS).unwrap(), 0);
        assert!(!root.join(META_DIR).join(SHOTS_SEEN).exists());
        // erstes Einschalten der Bildschirmfotos: Aufräumen an, Frist beginnt
        lib.shots_expire_default().unwrap();
        assert!(lib.shots_expire());
        assert_eq!(lib.expire_shots(DELETED_DAYS).unwrap(), 0);
        let seen_path = root.join(META_DIR).join(SHOTS_SEEN);
        let mut seen = lib.shots_seen();
        assert_eq!(seen.len(), 1, "nur Screenshots zählt");
        let h = lib.hash_of(&format!("{SHOTS}/shot.png")).unwrap();
        seen.insert(h, now_ms() - 8 * 24 * 3600 * 1000);
        fs::write(&seen_path, serde_json::to_vec(&seen).unwrap()).unwrap();
        assert_eq!(lib.expire_shots(DELETED_DAYS).unwrap(), 1);
        assert!(!root.join(SHOTS).join("shot.png").exists());
        assert!(root.join("Typo/typo.png").is_file());
        // aus bleibt aus, auch beim nächsten Einschalten der Bildschirmfotos
        lib.set_shots_expire(false).unwrap();
        lib.shots_expire_default().unwrap();
        assert!(!lib.shots_expire());
        assert!(lib.shots_dir().unwrap().is_dir());
    }

    #[test]
    fn unsorted_empties_itself_after_seven_days() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        let lib = Library::open(&root).unwrap();
        let src = tmp.path().join("src");
        write_png(&src.join("alt.png"), 4, 4, 10);
        write_png(&src.join("neu.png"), 4, 4, 20);
        write_png(&src.join("zone.png"), 4, 4, 30);
        lib.import_files(&[src.join("alt.png"), src.join("neu.png")], "Unsorted").unwrap();
        lib.import_files(&[src.join("zone.png")], "Typo").unwrap();
        // erster Lauf: nichts geht, die Frist beginnt
        assert_eq!(lib.expire_inbox(DELETED_DAYS).unwrap(), 0);
        let seen_path = root.join(META_DIR).join(INBOX_SEEN);
        let mut seen: HashMap<String, i64> = serde_json::from_slice(&fs::read(&seen_path).unwrap()).unwrap();
        assert_eq!(seen.len(), 2, "nur Unsorted zählt");
        assert_eq!(lib.inbox_seen(), seen, "Snapshot liefert dieselben Zeiten");
        // „alt.png“ liegt seit acht Tagen dort
        let alt = lib.hash_of("Unsorted/alt.png").unwrap();
        seen.insert(alt, now_ms() - 8 * 24 * 3600 * 1000);
        fs::write(&seen_path, serde_json::to_vec(&seen).unwrap()).unwrap();
        assert_eq!(lib.expire_inbox(DELETED_DAYS).unwrap(), 1);
        assert!(!root.join("Unsorted/alt.png").exists());
        assert!(root.join("Unsorted/neu.png").is_file());
        assert!(root.join("Typo/zone.png").is_file());
        // unter „Zuletzt gelöscht“, also wiederherstellbar
        assert_eq!(lib.deleted_items().unwrap().len(), 1);
    }

    #[test]
    fn rename_only_on_request() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        let lib = Library::open(&root).unwrap();
        let src = tmp.path().join("src");
        write_png(&src.join("IMG_1.png"), 4, 4, 41);
        write_png(&src.join("IMG_2.png"), 4, 4, 42);
        // Ab Werk bleibt der Name
        lib.import_files(&[src.join("IMG_1.png")], "Typo").unwrap();
        assert!(root.join("Typo/IMG_1.png").is_file());
        // Auch mit gewähltem Schema benennt der Import nie um (nur das Blatt, 0.36.4)
        let mut n = crate::naming::Naming::default();
        n.preset = "zone-nr".into();
        lib.import_files(&[src.join("IMG_2.png")], "Typo").unwrap();
        assert!(root.join("Typo/IMG_2.png").is_file(), "Import behält den Namen");
        assert!(root.join("Typo/IMG_1.png").is_file(), "Vorhandenes bleibt");
        // Aufräumen: Plan, dann umbenennen, Datenbank folgt
        let plan = lib.name_plan(&["Typo/IMG_1.png".into()], &n, "Bib").unwrap();
        assert_eq!(plan.len(), 1);
        assert!(plan[0].messy);
        assert!(plan[0].new.starts_with("typo_") && plan[0].new.ends_with(".png"));
        let done = lib.rename_items(&[("Typo/IMG_1.png".into(), "Mein Plakat".into())]).unwrap();
        assert_eq!(done, vec![("Typo/IMG_1.png".to_string(), "Typo/Mein Plakat.png".to_string())]);
        assert!(root.join("Typo/Mein Plakat.png").is_file());
        assert!(!root.join("Typo/IMG_1.png").exists());
        assert!(lib.hash_of("Typo/Mein Plakat.png").is_some(), "Eintrag wanderte mit");
        // nur Groß/klein
        lib.rename_items(&[("Typo/Mein Plakat.png".into(), "mein plakat.png".into())]).unwrap();
        assert!(root.join("Typo/mein plakat.png").is_file());
        // Pfadzeichen im Namen werden entfernt, nichts verlässt den Ordner
        let done = lib.rename_items(&[("Typo/mein plakat.png".into(), "../x/y".into())]).unwrap();
        assert_eq!(done[0].1, "Typo/xy.png");
    }

    #[test]
    fn unsorted_by_name() {
        for yes in ["Unsorted", "unsorted", "UNSORTED", "00_Unsorted", "01 unsorted", "Projekt/Unsorted", "Unsorted/Sub"] {
            assert!(is_unsorted(yes), "{yes}");
        }
        for no in ["", "Typo", "Unsortiert", "Unsorted Projekte", "Projekt/Typo"] {
            assert!(!is_unsorted(no), "{no}");
        }
    }

    #[test]
    fn shared_library_empties_only_when_switched_on() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        drop(Library::open(&root).unwrap());
        crate::studio::set_shared(&root, true).unwrap();
        let lib = Library::open(&root).unwrap();
        assert!(lib.shared);
        assert_eq!(lib.expire_zone(), None, "geteilt: ab Werk aus");
        let src = tmp.path().join("src");
        write_png(&src.join("a.png"), 4, 4, 41);
        lib.import_files(&[src.join("a.png")], "Unsorted").unwrap();
        assert_eq!(lib.expire_inbox(DELETED_DAYS).unwrap(), 0);
        assert!(lib.inbox_seen().is_empty());
        lib.set_expire_zone(INBOX).unwrap();
        assert_eq!(lib.expire_zone().as_deref(), Some(INBOX));
        assert_eq!(lib.expire_inbox(DELETED_DAYS).unwrap(), 0, "Frist beginnt");
        assert_eq!(lib.inbox_seen().len(), 1);
    }

    #[test]
    fn self_made_unsorted_empties_too() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        let lib = Library::open(&root).unwrap();
        let src = tmp.path().join("src");
        write_png(&src.join("a.png"), 4, 4, 11);
        write_png(&src.join("b.png"), 4, 4, 12);
        lib.import_files(&[src.join("a.png")], "unsorted").unwrap();
        lib.import_files(&[src.join("b.png")], "Typo").unwrap();
        assert_eq!(lib.expire_inbox(DELETED_DAYS).unwrap(), 0);
        let seen_path = root.join(META_DIR).join(INBOX_SEEN);
        let mut seen: HashMap<String, i64> = serde_json::from_slice(&fs::read(&seen_path).unwrap()).unwrap();
        assert_eq!(seen.len(), 1, "selbst angelegtes unsorted zählt");
        let a = lib.hash_of("unsorted/a.png").unwrap();
        seen.insert(a, now_ms() - 8 * 24 * 3600 * 1000);
        fs::write(&seen_path, serde_json::to_vec(&seen).unwrap()).unwrap();
        assert_eq!(lib.expire_inbox(DELETED_DAYS).unwrap(), 1);
        assert!(root.join("Typo/b.png").is_file());
        assert_eq!(lib.deleted_items().unwrap().len(), 1);
    }

    #[test]
    fn switching_unsorted_back_on_restarts_the_clock() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        let lib = Library::open(&root).unwrap();
        let src = tmp.path().join("src");
        write_png(&src.join("a.png"), 4, 4, 51);
        lib.import_files(&[src.join("a.png")], INBOX).unwrap();
        assert_eq!(lib.expire_inbox(DELETED_DAYS).unwrap(), 0);
        let seen_path = root.join(META_DIR).join(INBOX_SEEN);
        let mut seen = lib.inbox_seen();
        for v in seen.values_mut() {
            *v = now_ms() - 30 * 24 * 3600 * 1000;
        }
        fs::write(&seen_path, serde_json::to_vec(&seen).unwrap()).unwrap();
        lib.set_expire_zone("").unwrap();
        assert_eq!(lib.expire_inbox(DELETED_DAYS).unwrap(), 0, "aus");
        lib.set_expire_zone(INBOX).unwrap();
        assert_eq!(lib.expire_inbox(DELETED_DAYS).unwrap(), 0, "Frist beginnt neu");
        assert!(root.join("Unsorted/a.png").is_file());
    }

    #[test]
    fn paste_files_image_and_text() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        let lib = Library::open(&root).unwrap();
        // Bilddaten (PNG)
        let src = tmp.path().join("src");
        write_png(&src.join("a.png"), 4, 4, 10);
        let png = fs::read(src.join("a.png")).unwrap();
        let r = lib.paste_into("Typo", Pasted { image: Some(png), ..Default::default() }).unwrap();
        assert_eq!(r.imported, 1);
        // Text wird zur Notiz, benannt nach den ersten Wörtern
        let r = lib.paste_into("Typo", Pasted { text: Some("  Form folgt Funktion, sagt man  ".into()), ..Default::default() }).unwrap();
        assert_eq!(r.imported, 1);
        assert!(root.join("Typo/Form folgt Funktion, sagt man.md").is_file());
        // Dateien aus dem Finder gehen vor Bild und Text
        write_png(&src.join("b.png"), 4, 4, 20);
        let r = lib
            .paste_into("Typo", Pasted { files: vec![src.join("b.png")], image: Some(vec![1, 2, 3]), text: Some("b.png".into()) })
            .unwrap();
        assert_eq!(r.imported, 1);
        assert!(root.join("Typo/b.png").is_file());
        assert_eq!(lib.items().unwrap().iter().filter(|i| i.zone == "Typo").count(), 3);
        assert!(fs::read_dir(root.join(META_DIR).join("paste")).unwrap().next().is_none(), "Zwischenordner leer");
        assert!(paste_name("").starts_with("Eingefügt 20"));
    }

    #[test]
    fn note_keeps_source_when_edited() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        let lib = Library::open(&root).unwrap();
        fs::create_dir_all(root.join("Typo")).unwrap();
        let rel = lib.create_note("Typo", "Zitat").unwrap();
        let hash = lib.items().unwrap().into_iter().find(|i| i.path == rel).unwrap().hash;
        let src = Source { page_url: Some("https://a.b/x".into()), page_title: Some("Seite".into()), quote: Some("alt".into()), ..Default::default() };
        lib.set_source(&hash, &src).unwrap();
        let (rel, _) = lib.write_note(&rel, "# Zitat\n\nneu", false).unwrap();
        let item = lib.items().unwrap().into_iter().find(|i| i.path == rel).unwrap();
        assert_eq!(item.page_url.as_deref(), Some("https://a.b/x"));
        assert_eq!(item.page_title.as_deref(), Some("Seite"));
        assert_eq!(item.quote.as_deref(), Some("# Zitat\n\nneu"), "Text aus der Datei, nicht der alte");
    }

    #[test]
    fn zone_order_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let lib = Library::open(&tmp.path().join("Bib")).unwrap();
        assert_eq!(lib.zone_order(), (vec![], vec![]));
        let order: Vec<String> = ["Typo", "Unsorted", "Farbe"].map(String::from).to_vec();
        let hot: Vec<String> = order[..2].to_vec();
        lib.set_zone_order(&order, &hot).unwrap();
        assert_eq!(lib.zone_order(), (order, hot));
    }

    #[test]
    fn scan_zones_move_and_sources() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        let lib = Library::open(&root).unwrap();
        write_png(&root.join("Unsorted/a.png"), 40, 20, 10);
        write_png(&root.join("Projekt 1/b.png"), 10, 30, 20);
        write_png(&root.join(META_DIR).join("thumbs/ignore.png"), 2, 2, 1);
        assert!(lib.scan().unwrap());
        assert!(!lib.scan().unwrap(), "zweiter Scan ohne Änderung");

        let zones = lib.zones().unwrap();
        assert_eq!(zones.iter().map(|z| z.path.as_str()).collect::<Vec<_>>(), ["Unsorted", "Projekt 1"]);
        assert_eq!(zones[0].count, 1);

        let items = lib.items().unwrap();
        assert_eq!(items.len(), 2);
        let a = items.iter().find(|i| i.filename == "a.png").unwrap();
        assert_eq!((a.width, a.height), (Some(40), Some(20)));

        // Quelle hängt am Hash und überlebt das Verschieben
        lib.set_source(&a.hash, &Source { page_url: Some("https://example.com/x".into()), ..Default::default() }).unwrap();
        let moved = lib.move_items(&["Unsorted/a.png".into()], "Projekt 1").unwrap();
        assert_eq!(moved, ["Projekt 1/a.png"]);
        assert!(!lib.scan().unwrap(), "Verschieben hält die DB aktuell");
        let a2 = lib.items().unwrap().into_iter().find(|i| i.filename == "a.png").unwrap();
        assert_eq!(a2.zone, "Projekt 1");
        assert_eq!(a2.page_url.as_deref(), Some("https://example.com/x"));

        // Namenskonflikt beim Verschieben
        write_png(&root.join("Unsorted/b.png"), 5, 5, 99);
        lib.scan().unwrap();
        let moved = lib.move_items(&["Unsorted/b.png".into()], "Projekt 1").unwrap();
        assert_eq!(moved, ["Projekt 1/b 2.png"]);

        // Im Finder umbenannt: Scan findet die Quelle über den Hash wieder
        fs::rename(root.join("Projekt 1/a.png"), root.join("Projekt 1/umbenannt.png")).unwrap();
        assert!(lib.scan().unwrap());
        let r = lib.items().unwrap().into_iter().find(|i| i.filename == "umbenannt.png").unwrap();
        assert_eq!(r.page_url.as_deref(), Some("https://example.com/x"));

        // Zone umbenennen
        let z = lib.rename_zone("Projekt 1", "Kunde: A").unwrap();
        assert_eq!(z, "Kunde_ A");
        assert!(!lib.scan().unwrap());
        assert!(lib.items().unwrap().iter().all(|i| i.zone != "Projekt 1"));

        assert!(lib.move_items(&["../x.png".into()], "Unsorted").is_err());
        assert!(lib.trash_zone("Unsorted").is_err());
        assert!(lib.trash_zone("").is_err());
    }

    #[test]
    fn thumbs_and_metadata_restore() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        let lib = Library::open(&root).unwrap();
        write_png(&root.join("Unsorted/gross.png"), 2000, 1000, 50);
        fs::write(root.join("Unsorted/kaputt.jpg"), b"nope").unwrap();
        lib.scan().unwrap();
        let missing = lib.missing_thumbs().unwrap();
        assert_eq!(missing.len(), 2);
        let mut made = 0;
        for (hash, path) in missing {
            if let Some(t) = lib.make_thumb(&hash, &path) {
                let (w, h) = image::image_dimensions(&t).unwrap();
                assert_eq!((w, h), (720, 360));
                made += 1;
            }
        }
        assert_eq!(made, 1);
        let item = lib.items().unwrap().into_iter().find(|i| i.filename == "gross.png").unwrap();
        let icon = lib.drag_icon(&item.hash, Path::new(item.thumb.as_ref().unwrap())).unwrap();
        assert_eq!(image::image_dimensions(&icon).unwrap(), (96, 48));
        assert!(lib.missing_thumbs().unwrap().is_empty(), "kaputte Dateien nicht endlos neu versuchen");

        let hash = lib.items().unwrap()[0].hash.clone();
        lib.set_source(&hash, &Source { page_url: Some("https://a.b/".into()), ..Default::default() }).unwrap();
        lib.write_metadata().unwrap();
        drop(lib);
        fs::remove_file(root.join(META_DIR).join("library.db")).unwrap();
        let _ = fs::remove_file(root.join(META_DIR).join("library.db-wal"));
        let _ = fs::remove_file(root.join(META_DIR).join("library.db-shm"));
        let lib = Library::open(&root).unwrap();
        lib.scan().unwrap();
        let item = lib.items().unwrap().into_iter().find(|i| i.hash == hash).unwrap();
        assert_eq!(item.page_url.as_deref(), Some("https://a.b/"));
    }

    #[test]
    fn import_export_and_folder() {
        let tmp = tempfile::tempdir().unwrap();
        let downloads = tmp.path().join("Downloads");
        write_png(&downloads.join("Projekt 1/x.png"), 8, 8, 1);
        write_png(&downloads.join("lose.png"), 8, 8, 2);
        write_png(&downloads.join("Projekt 1/y.png"), 8, 8, 1); // Duplikat von x
        let export = serde_json::json!({
            "version": 1,
            "zones": ["Projekt 1", "Projekt 2", "Downloads"],
            "items": [
                { "zone": "Projekt 1", "file": downloads.join("Projekt 1/x.png"), "url": "https://cdn/x.png",
                  "pageUrl": "https://site/x", "pageTitle": "Seite X", "ts": 1700000000000i64 },
                { "zone": "", "file": downloads.join("lose.png"), "pageUrl": "https://site/l" },
                { "zone": "Projekt 1", "file": downloads.join("Projekt 1/fehlt.png") },
            ]
        });
        let json = tmp.path().join("archery-export.json");
        fs::write(&json, export.to_string()).unwrap();

        let root = tmp.path().join("Bib");
        let lib = Library::open(&root).unwrap();
        let r = lib.import_extension_export(&json).unwrap();
        assert_eq!((r.imported, r.missing, r.with_source), (2, 1, 2));
        assert!(root.join("Projekt 2").is_dir());
        let items = lib.items().unwrap();
        let x = items.iter().find(|i| i.filename == "x.png").unwrap();
        assert_eq!(x.zone, "Projekt 1");
        assert_eq!(x.page_title.as_deref(), Some("Seite X"));
        assert_eq!(items.iter().find(|i| i.filename == "lose.png").unwrap().zone, "Unsorted");
        assert!(root.join(META_DIR).join("metadata.json").is_file());

        let r = lib.import_folder(&downloads).unwrap();
        assert_eq!((r.imported, r.duplicates), (0, 3));
        assert!(!lib.scan().unwrap(), "Import hält die DB aktuell");
    }

    #[test]
    fn adopt_from_extension() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        let lib = Library::open(&root).unwrap();
        let dl = tmp.path().join("Downloads/Archiver-Transfer");
        write_png(&dl.join("foto.png"), 900, 500, 30);
        let source = Source {
            image_url: Some("https://x.test/foto.png".into()),
            page_url: Some("https://x.test/".into()),
            page_title: Some("Seite".into()),
            captured_at: Some(5),
            quote: Some("Form folgt Funktion".into()),
        };
        let (rel, _) = lib.adopt_file(&dl.join("foto.png"), None, "Typo", &source).unwrap();
        assert_eq!(rel, "Typo/foto.png");
        assert!(!dl.join("foto.png").exists(), "Datei wurde verschoben");
        let item = lib.items().unwrap().into_iter().find(|i| i.path == rel).unwrap();
        assert_eq!(item.page_title.as_deref(), Some("Seite"));
        assert_eq!(item.quote.as_deref(), Some("Form folgt Funktion"));
        let meta = std::fs::read_to_string(root.join(META_DIR).join("metadata.json")).unwrap();
        assert!(meta.contains("Form folgt Funktion"), "Zitat steht auch in metadata.json");
        assert!(!lib.scan().unwrap(), "DB ist aktuell");

        // gleiches Bild nochmal: keine Kopie, vorhandener Pfad
        write_png(&dl.join("nochmal.png"), 900, 500, 30);
        assert_eq!(lib.adopt_file(&dl.join("nochmal.png"), None, "Unsorted", &source).unwrap(), (rel.clone(), true));
        assert!(!dl.join("nochmal.png").exists());

        // ohne Endung (z. B. Download-ID als Name): Endung aus dem Inhalt
        image::RgbImage::from_pixel(30, 30, image::Rgb([90, 0, 0]))
            .save_with_format(dl.join("1234-abcd"), image::ImageFormat::Png)
            .unwrap();
        assert_eq!(lib.adopt_file(&dl.join("1234-abcd"), Some("wunsch"), "Unsorted", &source).unwrap().0, "Unsorted/wunsch.png");

        // PDF und Video ohne Endung: am Inhalt erkannt, als Eintrag gezählt
        std::fs::write(dl.join("dl-1"), b"%PDF-1.7\n%...").unwrap();
        assert_eq!(lib.adopt_file(&dl.join("dl-1"), Some("katalog"), "Unsorted", &source).unwrap().0, "Unsorted/katalog.pdf");
        std::fs::write(dl.join("dl-2"), b"\0\0\0\x18ftypisom\0\0\x02\0isomiso2").unwrap();
        assert_eq!(lib.adopt_file(&dl.join("dl-2"), Some("clip"), "Unsorted", &source).unwrap().0, "Unsorted/clip.mp4");
        std::fs::write(dl.join("dl-3"), b"%PDF-1.4\n").unwrap();
        assert_eq!(lib.adopt_file(&dl.join("dl-3"), Some("download.jpg"), "Unsorted", &source).unwrap().0, "Unsorted/download.pdf");
        let pdf = lib.items().unwrap().into_iter().find(|i| i.path == "Unsorted/katalog.pdf").unwrap();
        assert_eq!((pdf.format.as_str(), pdf.pages, pdf.width), ("pdf", None, None));
        assert!(!lib.scan().unwrap(), "PDF und Video bleiben beim Scan erhalten");
        assert_eq!(lib.items().unwrap().len(), 5);

        let (abs, hash) = lib.locate(&rel).unwrap();
        let (mime, bytes) = lib.small_preview(&hash, &abs).unwrap();
        assert_eq!(mime, "image/jpeg");
        let img = image::load_from_memory(&bytes).unwrap();
        assert_eq!((img.width(), img.height()), (400, 222));
    }

    #[test]
    fn text_cards_as_files() {
        let tmp = tempfile::tempdir().unwrap();
        let lib = Library::open(&tmp.path().join("Bib")).unwrap();
        let dl = tmp.path().join("dl");
        std::fs::create_dir_all(&dl).unwrap();
        std::fs::write(dl.join("x"), "Weniger, aber besser\n").unwrap();
        let source = Source { quote: Some("Weniger, aber besser".into()), ..Default::default() };
        let (rel, _) = lib.adopt_file(&dl.join("x"), Some("Weniger aber besser.txt"), "Zitate", &source).unwrap();
        assert_eq!(rel, "Zitate/Weniger aber besser.txt");
        // im Finder angelegt: Text kommt aus der Datei
        std::fs::write(lib.root.join("Zitate/notiz.txt"), "  Gute Form  ").unwrap();
        lib.scan().unwrap();
        let items = lib.items().unwrap();
        assert_eq!(items.len(), 2);
        let own = items.iter().find(|i| i.path == "Zitate/notiz.txt").unwrap();
        assert_eq!(own.quote.as_deref(), Some("Gute Form"));
        assert!(lib.missing_thumbs().unwrap().is_empty(), "keine Vorschaubilder für Text");
    }

    #[test]
    fn default_zone_replaces_inbox() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        let lib = Library::open(&root).unwrap();
        assert_eq!(lib.default_zone(), INBOX);
        lib.create_zone("Sammeln").unwrap();
        lib.set_default_zone("Sammeln").unwrap();
        assert_eq!(lib.default_zone(), "Sammeln");
        assert!(lib.trash_zone("Sammeln").is_err(), "Standard-Zone bleibt");
        let src = tmp.path().join("lose");
        std::fs::create_dir_all(&src).unwrap();
        write_png(&src.join("a.png"), 40, 40, 3);
        lib.import_folder(&src).unwrap();
        assert!(root.join("Sammeln/a.png").is_file());
        assert_eq!(lib.zones().unwrap()[0].path, "Sammeln");
        // ohne Inbox-Ordner erneut öffnen: keine neue Inbox
        std::fs::remove_dir_all(root.join(INBOX)).unwrap();
        drop(lib);
        let _ = Library::open(&root).unwrap();
        assert!(!root.join(INBOX).exists());
    }

    #[test]
    fn existing_folder_system() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Work");
        let deep = root.join("01_Outra/02_Layout/04_Products/_Mock/Cap");
        std::fs::create_dir_all(&deep).unwrap();
        write_png(&deep.join("cap.png"), 40, 40, 1);
        write_png(&root.join("01_Outra/02_Layout/logo.png"), 40, 40, 2);
        std::fs::create_dir_all(root.join("01_Outra/02_Layout/Links")).unwrap();
        write_png(&root.join("01_Outra/02_Layout/Links/placed.png"), 40, 40, 3);
        std::fs::create_dir_all(root.join("01_Outra/01_Intern")).unwrap();
        std::fs::write(root.join("01_Outra/01_Intern/brief.indd"), "x").unwrap();
        std::fs::create_dir_all(root.join("Plugin.app/Contents")).unwrap();
        write_png(&root.join("Plugin.app/Contents/icon.png"), 40, 40, 4);
        std::fs::create_dir_all(root.join("Neu")).unwrap();

        let lib = Library::open(&root).unwrap();
        assert!(root.join(INBOX).is_dir(), "Unsorted auch in bestehenden Ordnern (0.33.1)");
        lib.scan().unwrap();
        let items = lib.items().unwrap();
        assert_eq!(items.len(), 2, "Links und Pakete übersprungen");
        assert!(items.iter().all(|i| i.zone == "01_Outra"), "eine Zone pro Ordner der obersten Ebene");
        let mut folders: Vec<&str> = items.iter().map(|i| i.folder.as_str()).collect();
        folders.sort();
        assert_eq!(folders, ["02_Layout", "02_Layout"], "Unterordner als Ordner der Zone");
        let zones: Vec<String> = lib.zones().unwrap().into_iter().map(|z| z.path).collect();
        assert_eq!(zones, vec![INBOX.to_string(), "01_Outra".to_string(), "Neu".to_string()], "Unsorted zuerst");

        lib.set_structure(&Structure { depth: 2, skip: DEFAULT_SKIP.iter().map(|s| s.to_string()).collect(), subzones: true }).unwrap();
        let items = lib.items().unwrap();
        assert!(items.iter().all(|i| i.zone == "01_Outra/02_Layout"));
        assert!(items.iter().any(|i| i.folder == "04_Products") && items.iter().any(|i| i.folder.is_empty()));
        let zones: Vec<String> = lib.zones().unwrap().into_iter().map(|z| z.path).collect();
        assert_eq!(zones, vec![INBOX.to_string(), "01_Outra/02_Layout".to_string(), "Neu".to_string()], "nur Ebene 2, ohne reine Projektordner");

        lib.set_structure(&Structure { depth: 3, skip: vec![], subzones: true }).unwrap();
        let items = lib.items().unwrap();
        assert_eq!(items.len(), 3, "Links wieder dabei");
        assert!(items.iter().any(|i| i.zone == "01_Outra/02_Layout/04_Products"));
    }

    #[test]
    fn inbox_becomes_unsorted() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Alt");
        write_png(&root.join("Inbox/a.png"), 20, 20, 1);
        write_png(&root.join("Typo/b.png"), 20, 20, 2);
        std::fs::create_dir_all(root.join(META_DIR)).unwrap();
        std::fs::write(root.join(META_DIR).join("zones.json"), r#"{"order":["Typo","Inbox"],"hot":["Inbox"]}"#).unwrap();
        let lib = Library::open(&root).unwrap();
        assert!(root.join("Unsorted/a.png").exists() && !root.join("Inbox").exists());
        assert_eq!(lib.default_zone(), "Unsorted");
        assert_eq!(lib.zone_order(), (vec!["Typo".to_string(), "Unsorted".to_string()], vec!["Unsorted".to_string()]));
        // eine andere gewählte Standard-Zone: Inbox bleibt, wie sie ist
        let root2 = tmp.path().join("Eigen");
        write_png(&root2.join("Inbox/a.png"), 20, 20, 1);
        std::fs::create_dir_all(root2.join("Typo")).unwrap();
        std::fs::create_dir_all(root2.join(META_DIR)).unwrap();
        std::fs::write(root2.join(META_DIR).join("zones.json"), r#"{"default":"Typo"}"#).unwrap();
        Library::open(&root2).unwrap();
        assert!(root2.join("Inbox").is_dir());
    }
}

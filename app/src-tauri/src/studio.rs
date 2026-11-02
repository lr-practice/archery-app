//! studio.rs — Werkzeuge für Designer und Studios:
//! Notiz und Rechte pro Objekt, „abgelegt von“, geteilte
//! Bibliothek (eine Datei pro Person statt einer gemeinsamen Datenbank),
//! eigene Reihenfolge in Zonen und Zone als Paket.
//!
//! Alles hängt wie die Quellen am Content-Hash und steht zusätzlich in
//! .archiver/metadata.json (bzw. geteilt in .archiver/people/<name>.json),
//! damit nichts an der Datenbank hängt.

use crate::library::{err, now_ms, Item, Library, Source, META_DIR};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, String>;

const SHARED_MARKER: &str = "shared.json";
const PEOPLE_DIR: &str = "people";

/// Notiz, Rechte und Herkunft eines Objekts.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub internal: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub added_by: Option<String>,
    /// Farbe einer Textkarte (red, green, violet, cyan, yellow, white)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tone: Option<String>,
    /// Bildtext (0.29): Credits oder Unterschrift, wie unter einem Bild in einem Magazin
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    /// Notiz-Objekt (0.29): verknüpfte Objekte als Hashes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<String>>,
    /// Farbmarke in der Übersicht (0.41.0): gehört zu einer Auswahl in dieser Farbe
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mark: Option<String>,
    #[serde(default)]
    pub updated_at: i64,
}

/// Änderung aus der Oberfläche: nur gesetzte Felder werden übernommen.
#[derive(Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct NotePatch {
    pub note: Option<String>,
    pub credit: Option<String>,
    pub license: Option<String>,
    pub internal: Option<bool>,
    pub tone: Option<String>,
    pub caption: Option<String>,
    pub links: Option<Vec<String>>,
    /// Farbmarke, leer heißt keine
    pub mark: Option<String>,
}

/// Erlaubte Farben einer Textkarte; alles andere heißt „keine eigene Farbe“.
pub const TONES: [&str; 7] = ["red", "green", "violet", "cyan", "yellow", "white", "black"];

/// Verknüpfungen einer Notiz: als JSON-Liste in der Datenbank, leer heißt keine
pub(crate) fn links_json(links: &Option<Vec<String>>) -> Option<String> {
    links.as_ref().filter(|l| !l.is_empty()).and_then(|l| serde_json::to_string(l).ok())
}

pub(crate) fn links_of(json: Option<String>) -> Option<Vec<String>> {
    json.and_then(|j| serde_json::from_str::<Vec<String>>(&j).ok()).filter(|l| !l.is_empty())
}

pub fn clean_tone(t: Option<&str>) -> Option<String> {
    t.filter(|t| TONES.contains(t)).map(String::from)
}

/// Farbmarken: die fünf Farben der Textkarten ohne Weiß und Schwarz
pub const MARKS: [&str; 5] = ["red", "green", "violet", "cyan", "yellow"];

pub fn clean_mark(t: Option<&str>) -> Option<String> {
    t.filter(|t| MARKS.contains(t)).map(String::from)
}

/// Inhalt von metadata.json und den Personen-Dateien.
#[derive(Deserialize, Default)]
pub struct MetaDoc {
    #[serde(default)]
    pub sources: HashMap<String, Source>,
    #[serde(default)]
    pub notes: HashMap<String, Note>,
}

fn clean(s: Option<String>) -> Option<String> {
    s.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Liegt die Bibliothek als geteilt markiert?
pub fn is_shared(root: &Path) -> bool {
    root.join(META_DIR).join(SHARED_MARKER).is_file()
}

/// Lokaler Ort der Datenbank für eine geteilte Bibliothek (pro Mac, pro Bibliothek).
pub fn local_db(root: &Path) -> Result<PathBuf> {
    let base = dirs::data_local_dir().ok_or("Kein lokaler Datenordner")?;
    let key = blake3::hash(root.to_string_lossy().as_bytes()).to_hex()[..16].to_string();
    let dir = base.join(crate::bridge::APP_IDENTIFIER).join("libraries").join(key);
    fs::create_dir_all(&dir).map_err(err)?;
    Ok(dir.join("library.db"))
}

/// Dateiname aus Name oder Kürzel: nur Buchstaben, Ziffern und Bindestriche.
pub fn person_slug(name: &str) -> String {
    let s: String = name
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    let s = s.split('-').filter(|p| !p.is_empty()).collect::<Vec<_>>().join("-");
    if s.is_empty() { "ich".into() } else { s }
}

/// Bibliothek als geteilt markieren oder die Markierung entfernen.
/// Vorher sollte der aktuelle Stand geschrieben sein (write_metadata).
pub fn set_shared(root: &Path, on: bool) -> Result<()> {
    let marker = root.join(META_DIR).join(SHARED_MARKER);
    if on {
        let body = serde_json::json!({ "version": 1, "since": now_ms() });
        fs::write(marker, serde_json::to_vec_pretty(&body).map_err(err)?).map_err(err)
    } else {
        match fs::remove_file(marker) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(err(e)),
        }
    }
}

impl Library {
    pub fn set_user(&self, user: Option<String>) {
        *self.user.lock().unwrap() = clean(user);
    }

    pub(crate) fn person_file(&self) -> PathBuf {
        let name = self.user.lock().unwrap().clone().or_else(|| std::env::var("USER").ok()).unwrap_or_default();
        self.root.join(META_DIR).join(PEOPLE_DIR).join(format!("{}.json", person_slug(&name)))
    }

    fn people_files(&self) -> Vec<PathBuf> {
        let Ok(dir) = fs::read_dir(self.root.join(META_DIR).join(PEOPLE_DIR)) else { return vec![] };
        let mut files: Vec<PathBuf> = dir
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
            .collect();
        files.sort();
        files
    }

    /// Fingerabdruck der Personen-Dateien (Name, Größe, Änderungszeit).
    pub(crate) fn people_signature(&self) -> u64 {
        let mut h = blake3::Hasher::new();
        for f in self.people_files() {
            h.update(f.to_string_lossy().as_bytes());
            if let Ok(m) = fs::metadata(&f) {
                h.update(&m.len().to_le_bytes());
                let t = m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_nanos()).unwrap_or(0);
                h.update(&t.to_le_bytes());
            }
        }
        u64::from_le_bytes(h.finalize().as_bytes()[..8].try_into().unwrap())
    }

    /// Geteilt: Quellen und Notizen aller Personen übernehmen (nur bei Änderung).
    /// Quellen werden ergänzt, bei Notizen gewinnt die neuere Fassung.
    pub(crate) fn merge_people(&self) -> Result<bool> {
        if !self.shared {
            return Ok(false);
        }
        let sig = self.people_signature();
        if *self.people_sig.lock().unwrap() == sig {
            return Ok(false);
        }
        let mut changed = false;
        for f in self.people_files() {
            let Ok(text) = fs::read_to_string(&f) else { continue };
            let Ok(doc) = serde_json::from_str::<MetaDoc>(&text) else { continue };
            changed |= self.merge_notes(&doc.notes)?;
            for (hash, s) in &doc.sources {
                changed |= self.set_source(hash, s)?;
            }
        }
        *self.people_sig.lock().unwrap() = sig;
        Ok(changed)
    }

    /// Notizen übernehmen, wenn sie neuer sind als die eigenen. true bei Änderung.
    pub(crate) fn merge_notes(&self, notes: &HashMap<String, Note>) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let mut changed = false;
        for (hash, n) in notes {
            let rows = conn
                .execute(
                    "INSERT INTO note (hash, note, credit, license, internal, added_by, updated_at, tone, caption, links, mark)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                     ON CONFLICT(hash) DO UPDATE SET note=?2, credit=?3, license=?4,
                       internal=?5, added_by=COALESCE(?6, note.added_by), updated_at=?7, tone=?8, caption=?9, links=?10, mark=?11
                     WHERE ?7 > note.updated_at",
                    params![hash, n.note, n.credit, n.license, n.internal as i64, n.added_by, n.updated_at, n.tone, n.caption, links_json(&n.links), clean_mark(n.mark.as_deref())],
                )
                .map_err(err)?;
            changed |= rows > 0;
        }
        Ok(changed)
    }

    pub(crate) fn all_notes(&self) -> Result<serde_json::Map<String, serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT hash, note, credit, license, internal, added_by, updated_at, tone, caption, links, mark FROM note ORDER BY hash")
            .map_err(err)?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    Note {
                        note: r.get(1)?,
                        credit: r.get(2)?,
                        license: r.get(3)?,
                        internal: r.get::<_, Option<i64>>(4)?.unwrap_or(0) != 0,
                        added_by: r.get(5)?,
                        updated_at: r.get(6)?,
                        tone: r.get(7)?,
                        caption: r.get(8)?,
                        links: links_of(r.get(9)?),
                        mark: r.get(10)?,
                    },
                ))
            })
            .map_err(err)?;
        let mut out = serde_json::Map::new();
        for row in rows {
            let (hash, n) = row.map_err(err)?;
            out.insert(hash, serde_json::to_value(n).map_err(err)?);
        }
        Ok(out)
    }

    fn note_of(&self, hash: &str) -> Result<Note> {
        let conn = self.conn.lock().unwrap();
        let n = conn
            .query_row(
                "SELECT note, credit, license, internal, added_by, updated_at, tone, caption, links, mark FROM note WHERE hash = ?1",
                params![hash],
                |r| {
                    Ok(Note {
                        note: r.get(0)?,
                        credit: r.get(1)?,
                        license: r.get(2)?,
                        internal: r.get::<_, Option<i64>>(3)?.unwrap_or(0) != 0,
                        added_by: r.get(4)?,
                        updated_at: r.get(5)?,
                        tone: r.get(6)?,
                        caption: r.get(7)?,
                        links: links_of(r.get(8)?),
                        mark: r.get(9)?,
                    })
                },
            )
            .optional()
            .map_err(err)?;
        // Nur „keine Zeile“ heißt leere Notiz. Ein DB-Fehler (z. B. gesperrt) darf
        // nicht als leer gelten, sonst überschriebe set_note alle Angaben (0.31.0).
        Ok(n.unwrap_or_default())
    }

    /// Notiz oder Rechte für mehrere Objekte setzen.
    pub fn set_note(&self, hashes: &[String], patch: &NotePatch) -> Result<()> {
        let now = now_ms();
        for hash in hashes {
            let mut n = self.note_of(hash)?;
            if let Some(v) = &patch.note {
                n.note = clean(Some(v.clone()));
            }
            if let Some(v) = &patch.credit {
                n.credit = clean(Some(v.clone()));
            }
            if let Some(v) = &patch.license {
                n.license = clean(Some(v.clone()));
            }
            if let Some(v) = patch.internal {
                n.internal = v;
            }
            if let Some(v) = &patch.tone {
                n.tone = clean_tone(Some(v));
            }
            if let Some(v) = &patch.caption {
                n.caption = clean(Some(v.chars().take(4000).collect()));
            }
            if let Some(v) = &patch.mark {
                n.mark = clean_mark(Some(v));
            }
            if let Some(v) = &patch.links {
                let mut seen = HashSet::new();
                let list: Vec<String> = v.iter().filter(|h| *h != hash && seen.insert(h.to_string())).cloned().collect();
                n.links = Some(list).filter(|l| !l.is_empty());
            }
            // Immer neuer als alles Bisherige, auch bei schiefer Uhr eines anderen Macs
            n.updated_at = now.max(n.updated_at + 1);
            let mut map = HashMap::new();
            map.insert(hash.clone(), n);
            self.merge_notes(&map)?;
        }
        self.write_metadata()
    }

    /// Neues Objekt: merken, wer es abgelegt hat (nur mit Namen in den Einstellungen).
    pub(crate) fn mark_added_by(&self, hash: &str) -> Result<()> {
        let Some(user) = self.user.lock().unwrap().clone() else { return Ok(()) };
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO note (hash, added_by, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(hash) DO UPDATE SET added_by = ?2, updated_at = MAX(note.updated_at + 1, ?3)
             WHERE note.added_by IS NULL",
            params![hash, user, now_ms()],
        )
        .map_err(err)?;
        Ok(())
    }

    // ---- Eigene Reihenfolge in Zonen ----

    /// Reihenfolge je Zone als Liste von Hashes (aus zones.json, Schlüssel "sort").
    pub fn item_order(&self) -> HashMap<String, Vec<String>> {
        self.zone_file()
            .get("sort")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    /// Reihenfolge einer Zone speichern; leere Liste setzt sie zurück.
    pub fn set_item_order(&self, zone: &str, hashes: &[String]) -> Result<()> {
        let mut v = self.zone_file();
        if !v.get("sort").map(|s| s.is_object()).unwrap_or(false) {
            v["sort"] = serde_json::json!({});
        }
        let sort = v["sort"].as_object_mut().unwrap();
        if hashes.is_empty() {
            sort.remove(zone);
        } else {
            sort.insert(zone.to_string(), serde_json::json!(hashes));
        }
        self.write_zone_file(&v)
    }

    /// Neuer Hash eines Objekts (Notiz bearbeitet): Platz in der Reihenfolge bleibt.
    pub(crate) fn replace_in_order(&self, old: &str, new: &str) -> Result<()> {
        let mut v = self.zone_file();
        let Some(sort) = v.get_mut("sort").and_then(|s| s.as_object_mut()) else { return Ok(()) };
        let mut hit = false;
        for list in sort.values_mut() {
            if let Some(arr) = list.as_array_mut() {
                for h in arr.iter_mut() {
                    if h.as_str() == Some(old) {
                        *h = serde_json::json!(new);
                        hit = true;
                    }
                }
            }
        }
        if hit { self.write_zone_file(&v) } else { Ok(()) }
    }

    /// Nach Umbenennen einer Zone: Reihenfolge wandert mit.
    /// Zone oder Ordner umbenannt/verschoben: Sortierung, Standard-Zone,
    /// Reihenfolge und Hot Zones in zones.json mitziehen (Standard seit 0.31.0,
    /// sonst fiel die Standard-Zone still auf „Unsorted“ zurück).
    pub(crate) fn rename_item_order(&self, old: &str, new: &str) -> Result<()> {
        let mut v = self.zone_file();
        let renamed = |k: &str| -> Option<String> {
            (k == old || k.starts_with(&format!("{old}/"))).then(|| format!("{new}{}", &k[old.len()..]))
        };
        let mut changed = false;
        if let Some(sort) = v.get_mut("sort").and_then(|s| s.as_object_mut()) {
            let moves: Vec<(String, String)> = sort.keys().filter_map(|k| renamed(k).map(|to| (k.clone(), to))).collect();
            for (from, to) in moves {
                if let Some(list) = sort.remove(&from) {
                    sort.insert(to, list);
                    changed = true;
                }
            }
        }
        for key in ["default", "expire"] {
            if let Some(to) = v.get(key).and_then(|d| d.as_str()).and_then(renamed) {
                v[key] = serde_json::json!(to);
                changed = true;
            }
        }
        for key in ["order", "hot"] {
            if let Some(list) = v.get_mut(key).and_then(|a| a.as_array_mut()) {
                for z in list.iter_mut() {
                    if let Some(to) = z.as_str().and_then(renamed) {
                        *z = serde_json::json!(to);
                        changed = true;
                    }
                }
            }
        }
        if !changed {
            return Ok(());
        }
        self.write_zone_file(&v)
    }

    // ---- Zone als Paket ----

    /// Kopiert Objekte in einen neuen Ordner unter `dest`, dazu Quellen.csv,
    /// Quellen.md und einen Kontaktbogen. Gibt den Paketordner zurück.
    pub fn export_package(&self, paths: &[String], title: &str, meta: &str, dest: &Path, sheet: bool) -> Result<PackageReport> {
        let items: Vec<Item> = {
            let all = self.items()?;
            let by_path: HashMap<&str, &Item> = all.iter().map(|i| (i.path.as_str(), i)).collect();
            paths.iter().filter_map(|p| by_path.get(p.as_str()).map(|i| (*i).clone())).collect()
        };
        if items.is_empty() {
            return Err("Nichts ausgewählt".into());
        }
        let name = crate::library::clean_segment(title);
        let name = if name.is_empty() { "Archery".to_string() } else { name };
        let dir = crate::library::unique_path(dest, &format!("{name} (Archery)"));
        fs::create_dir_all(&dir).map_err(err)?;

        let mut files = Vec::new();
        for it in &items {
            let dst = crate::library::unique_path(&dir, &it.filename);
            fs::copy(&it.abs, &dst).map_err(err)?;
            files.push(dst.file_name().unwrap().to_string_lossy().into_owned());
        }

        fs::write(dir.join("Quellen.csv"), sources_csv(&items, &files)).map_err(err)?;
        fs::write(dir.join("Quellen.md"), sources_md(title, meta, &items, &files)).map_err(err)?;
        if sheet {
            let sheet_items: Vec<crate::pdf::SheetItem> = items
                .iter()
                .map(|i| crate::pdf::SheetItem {
                    thumb: i.thumb.as_ref().map(PathBuf::from),
                    file: crate::library::is_image(Path::new(&i.abs)).then(|| PathBuf::from(&i.abs)),
                    name: i.filename.clone(),
                    source: i.page_url.as_deref().map(domain).filter(|d| !d.is_empty()),
                    quote: i.quote.clone(),
                    label: i.format.to_uppercase(),
                    note: i.caption.clone().or_else(|| i.note.clone()),
                })
                .collect();
            let layout = crate::pdf::Layout::auto(sheet_items.len());
            let pdf = crate::pdf::contact_sheet(title, meta, &sheet_items, crate::pdf::Layout { notes: true, ..layout });
            fs::write(dir.join("Kontaktbogen.pdf"), pdf).map_err(err)?;
        }
        let internal = items.iter().filter(|i| i.internal).count();
        Ok(PackageReport { path: dir.to_string_lossy().into_owned(), count: items.len(), internal })
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackageReport {
    pub path: String,
    pub count: usize,
    /// Davon „nur intern“ markiert: die Oberfläche weist darauf hin
    pub internal: usize,
}

fn domain(url: &str) -> String {
    url.split("://").nth(1).unwrap_or(url).split('/').next().unwrap_or("").trim_start_matches("www.").to_string()
}

fn date(ms: Option<i64>) -> String {
    let Some(ms) = ms else { return String::new() };
    // Tage seit 1970 → Datum (proleptisch gregorianisch, UTC)
    let days = ms.div_euclid(86_400_000);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

fn csv_cell(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r', ';']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

pub fn sources_csv(items: &[Item], files: &[String]) -> String {
    let mut out = String::from(
        "Datei,Seitentitel,Seite,Bild-URL,Gesichert,Urheber,Lizenz,Nur intern,Bildtext,Abgelegt von\n",
    );
    for (i, f) in items.iter().zip(files) {
        let row = [
            f.clone(),
            i.page_title.clone().unwrap_or_default(),
            i.page_url.clone().unwrap_or_default(),
            i.image_url.clone().unwrap_or_default(),
            date(i.captured_at.or(Some(i.added_at))),
            i.credit.clone().unwrap_or_default(),
            i.license.clone().unwrap_or_default(),
            if i.internal { "ja".into() } else { String::new() },
            i.caption.clone().or_else(|| i.note.clone()).unwrap_or_default(),
            i.added_by.clone().unwrap_or_default(),
        ];
        out.push_str(&row.iter().map(|c| csv_cell(c)).collect::<Vec<_>>().join(","));
        out.push('\n');
    }
    // BOM, damit Excel und Numbers Umlaute richtig lesen
    format!("\u{feff}{out}")
}

pub fn sources_md(title: &str, meta: &str, items: &[Item], files: &[String]) -> String {
    let mut out = format!("# {title}\n\n{meta}\n\n");
    for (i, f) in items.iter().zip(files) {
        out.push_str(&format!("## {f}\n\n"));
        let mut line = |label: &str, value: Option<String>| {
            if let Some(v) = value.filter(|v| !v.is_empty()) {
                out.push_str(&format!("- **{label}:** {v}\n"));
            }
        };
        let page = match (&i.page_title, &i.page_url) {
            (Some(t), Some(u)) => Some(format!("[{t}]({u})")),
            (None, Some(u)) => Some(u.clone()),
            _ => None,
        };
        line("Quelle", page);
        line("Bild", i.image_url.clone());
        line("Gesichert", Some(date(i.captured_at.or(Some(i.added_at)))));
        line("Urheber", i.credit.clone());
        line("Lizenz", i.license.clone());
        line("Nur intern", i.internal.then(|| "ja".to_string()));
        line("Abgelegt von", i.added_by.clone());
        // 0.30: Bildtext (das frühere Feld „Notiz“ ist darin aufgegangen)
        if let Some(n) = i.caption.as_ref().or(i.note.as_ref()).filter(|n| !n.is_empty()) {
            out.push_str(&format!("\n> {}\n", n.replace('\n', "\n> ")));
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_png(path: &Path, rgb: [u8; 3]) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        image::RgbImage::from_pixel(8, 8, image::Rgb(rgb)).save(path).unwrap();
    }

    #[test]
    fn slug_and_date() {
        assert_eq!(person_slug(" Lukas R. "), "lukas-r");
        assert_eq!(person_slug("!!"), "ich");
        assert_eq!(date(Some(0)), "1970-01-01");
        assert_eq!(date(Some(1_791_590_400_000)), "2026-10-10");
    }

    #[test]
    fn notes_roundtrip_and_restore() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        let lib = Library::open(&root).unwrap();
        lib.set_user(Some("Lukas".into()));
        let src = tmp.path().join("in/a.png");
        write_png(&src, [200, 10, 10]);
        let report = lib.import_files(&[src], "Inbox").unwrap();
        assert_eq!(report.imported, 1);
        let item = lib.items().unwrap().remove(0);
        assert_eq!(item.added_by.as_deref(), Some("Lukas"));

        let patch = NotePatch {
            note: Some("Kontrast  ".into()),
            credit: Some("Studio X".into()),
            internal: Some(true),
            caption: Some("Pause II at Ormside Projects, 2025".into()),
            ..Default::default()
        };
        lib.set_note(&[item.hash.clone()], &patch).unwrap();
        let item = lib.items().unwrap().remove(0);
        assert_eq!(item.note.as_deref(), Some("Kontrast"));
        assert_eq!(item.credit.as_deref(), Some("Studio X"));
        assert!(item.internal);
        assert_eq!(item.added_by.as_deref(), Some("Lukas"));

        // Datenbank weg: alles kommt aus metadata.json zurück
        drop(lib);
        fs::remove_file(root.join(META_DIR).join("library.db")).unwrap();
        let lib = Library::open(&root).unwrap();
        lib.scan().unwrap();
        let item = lib.items().unwrap().remove(0);
        assert_eq!(item.note.as_deref(), Some("Kontrast"));
        assert_eq!(item.added_by.as_deref(), Some("Lukas"));
        assert_eq!(item.caption.as_deref(), Some("Pause II at Ormside Projects, 2025"), "Bildtext übersteht den Verlust der Datenbank");

        // Leere Notiz löscht
        lib.set_note(&[item.hash.clone()], &NotePatch { note: Some(" ".into()), ..Default::default() }).unwrap();
        assert_eq!(lib.items().unwrap()[0].note, None);

        // Farbmarke: nur die fünf Farben, leer löscht, übersteht den Verlust der Datenbank
        lib.set_note(&[item.hash.clone()], &NotePatch { mark: Some("white".into()), ..Default::default() }).unwrap();
        assert_eq!(lib.items().unwrap()[0].mark, None);
        lib.set_note(&[item.hash.clone()], &NotePatch { mark: Some("cyan".into()), ..Default::default() }).unwrap();
        drop(lib);
        fs::remove_file(root.join(META_DIR).join("library.db")).unwrap();
        let lib = Library::open(&root).unwrap();
        lib.scan().unwrap();
        assert_eq!(lib.items().unwrap()[0].mark.as_deref(), Some("cyan"));
        lib.set_note(&[item.hash.clone()], &NotePatch { mark: Some(String::new()), ..Default::default() }).unwrap();
        assert_eq!(lib.items().unwrap()[0].mark, None);
    }

    #[test]
    fn shared_library_merges_people() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Studio");
        // Zwei Macs simulieren: gleiche Bibliothek, getrennte Datenbanken
        let a = Library::open(&root).unwrap();
        write_png(&root.join("Inbox/x.png"), [10, 200, 10]);
        a.scan().unwrap();
        a.write_metadata().unwrap();
        drop(a);
        set_shared(&root, true).unwrap();
        assert!(is_shared(&root));

        let a = Library::open(&root).unwrap();
        a.set_user(Some("Anna".into()));
        a.scan().unwrap();
        let hash = a.items().unwrap()[0].hash.clone();
        a.set_note(&[hash.clone()], &NotePatch { note: Some("von Anna".into()), ..Default::default() }).unwrap();
        assert!(root.join(META_DIR).join("people/anna.json").is_file());

        // Ben: eigener Cache (anderer DB-Pfad), liest Annas Datei
        let ben_db = tmp.path().join("ben.db");
        let b = open_with_db(&root, &ben_db);
        b.set_user(Some("Ben".into()));
        b.scan().unwrap();
        assert_eq!(b.items().unwrap()[0].note.as_deref(), Some("von Anna"));
        b.set_note(&[hash.clone()], &NotePatch { note: Some("Ben war hier".into()), ..Default::default() }).unwrap();

        // Anna sieht Bens neuere Notiz beim nächsten Scan
        assert!(a.scan().unwrap());
        assert_eq!(a.items().unwrap()[0].note.as_deref(), Some("Ben war hier"));
        // metadata.json bleibt unberührt, jeder schreibt nur seine Datei
        assert!(root.join(META_DIR).join("people/ben.json").is_file());
    }

    /// Wie Library::open, aber mit frei gewählter Datenbank (zweiter Mac im Test).
    fn open_with_db(root: &Path, db: &Path) -> Library {
        let lib = Library::open(root).unwrap();
        let conn = rusqlite::Connection::open(db).unwrap();
        conn.execute_batch(crate::library::SCHEMA).unwrap();
        let _ = conn.execute("ALTER TABLE source ADD COLUMN quote TEXT", []);
        for col in ["title TEXT", "artist TEXT", "album TEXT", "sample_rate INTEGER", "bit_depth INTEGER", "channels INTEGER"] {
            let _ = conn.execute(&format!("ALTER TABLE media ADD COLUMN {col}"), []);
        }
        *lib.conn.lock().unwrap() = conn;
        *lib.people_sig.lock().unwrap() = 0;
        lib
    }

    #[test]
    fn item_order_survives_rename() {
        let tmp = tempfile::tempdir().unwrap();
        let lib = Library::open(&tmp.path().join("Bib")).unwrap();
        let zone = lib.create_zone("Typo").unwrap();
        lib.set_item_order(&zone, &["b".into(), "a".into()]).unwrap();
        lib.set_zone_order(&[zone.clone()], &[]).unwrap();
        let new = lib.rename_zone(&zone, "Schrift").unwrap();
        assert_eq!(lib.item_order().get(&new).unwrap(), &vec!["b".to_string(), "a".to_string()]);
        assert!(lib.item_order().get(&zone).is_none());
        lib.set_item_order(&new, &[]).unwrap();
        assert!(lib.item_order().is_empty());
        // Reihenfolge der Zonen und Standard-Zone ziehen mit (0.31.0)
        assert_eq!(lib.zone_order().0, vec![new.clone()]);
        lib.set_default_zone(&new).unwrap();
        let renamed = lib.rename_zone(&new, "Schriften").unwrap();
        assert_eq!(lib.default_zone(), renamed);
        // Nur Groß-/Kleinschreibung: kein „schriften 2“
        assert_eq!(lib.rename_zone(&renamed, "schriften").unwrap(), "schriften");
    }

    #[test]
    fn note_keeps_tone_links_and_place_when_edited() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        let lib = Library::open(&root).unwrap();
        write_png(&root.join("Typo/a.png"), [0, 0, 250]);
        lib.scan().unwrap();
        let img = lib.items().unwrap()[0].hash.clone();
        let rel = lib.create_note("Typo", "Neue Notiz").unwrap();
        assert_eq!(rel, "Typo/Neue Notiz.md");
        let note = lib.items().unwrap().into_iter().find(|i| i.path == rel).unwrap();
        assert_eq!(note.quote.as_deref(), Some("# Neue Notiz"));
        lib.set_note(&[note.hash.clone()], &NotePatch { tone: Some("black".into()), links: Some(vec![img.clone(), img.clone(), note.hash.clone()]), ..Default::default() })
            .unwrap();
        lib.set_item_order("Typo", &[note.hash.clone(), img.clone()]).unwrap();
        // Schreiben ohne Umbenennen: gleicher Pfad, neuer Hash, Angaben wandern mit
        let (p, h) = lib.write_note(&rel, "# Pause II\n\nRot über Foto", false).unwrap();
        assert_eq!(p, rel);
        assert_ne!(h, note.hash);
        assert!(!lib.scan().unwrap(), "Scan sieht nichts Neues");
        let after = lib.items().unwrap().into_iter().find(|i| i.path == rel).unwrap();
        assert_eq!(after.tone.as_deref(), Some("black"));
        assert_eq!(after.links, Some(vec![img.clone()]));
        assert_eq!(after.added_at, note.added_at);
        assert_eq!(lib.item_order().get("Typo").unwrap(), &vec![h.clone(), img.clone()]);
        // Beim Verlassen: Titel wird Dateiname
        let (p, _) = lib.write_note(&rel, "# Pause II\n\nRot über Foto", true).unwrap();
        assert_eq!(p, "Typo/Pause II.md");
        assert!(!root.join(&rel).exists());
        assert_eq!(lib.read_note(&p).unwrap(), "# Pause II\n\nRot über Foto\n");
        assert_eq!(lib.items().unwrap().iter().filter(|i| i.format == "md").count(), 1);
    }

    #[test]
    fn package_has_files_sources_and_sheet() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Bib");
        let lib = Library::open(&root).unwrap();
        write_png(&root.join("Plakate/a.png"), [0, 0, 250]);
        write_png(&root.join("Plakate/b.png"), [250, 250, 0]);
        lib.scan().unwrap();
        let items = lib.items().unwrap();
        lib.set_note(&[items[0].hash.clone()], &NotePatch { credit: Some("M. Muster".into()), internal: Some(true), ..Default::default() })
            .unwrap();
        let paths: Vec<String> = items.iter().map(|i| i.path.clone()).collect();
        let out = tmp.path().join("out");
        fs::create_dir_all(&out).unwrap();
        let rep = lib.export_package(&paths, "Plakate", "2 Objekte", &out, true).unwrap();
        assert_eq!((rep.count, rep.internal), (2, 1));
        let dir = PathBuf::from(&rep.path);
        assert!(dir.ends_with("Plakate (Archery)"));
        for f in ["a.png", "b.png", "Quellen.csv", "Quellen.md", "Kontaktbogen.pdf"] {
            assert!(dir.join(f).is_file(), "{f} fehlt");
        }
        let csv = fs::read_to_string(dir.join("Quellen.csv")).unwrap();
        assert!(csv.contains("M. Muster") && csv.contains(",ja,"));
        assert!(fs::read(dir.join("Kontaktbogen.pdf")).unwrap().starts_with(b"%PDF"));
        // zweiter Export legt einen neuen Ordner an
        let rep2 = lib.export_package(&paths, "Plakate", "", &out, false).unwrap();
        assert_ne!(rep.path, rep2.path);
    }
}

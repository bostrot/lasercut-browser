//! Walks the library and turns folders into "projects".
//!
//! Heuristics:
//! * a project is a folder that directly contains design/preview files;
//! * format-only sub folders ("SVG", "DXF files", "3mm", "png") are merged into their parent;
//! * a "dump" folder with many unrelated designs (e.g. "400 VETORES") is split into one
//!   project per file stem;
//! * identical projects that appear in several packs share a `dup_key` so results can be deduped.

use crate::categories::categorize;
use anyhow::Result;
use rusqlite::{params, Connection};
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::time::Instant;
use walkdir::WalkDir;

pub const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "jfif", "png", "gif", "bmp", "webp", "tif", "tiff"];

const JUNK_EXTS: &[&str] = &[
    "ini", "db", "ds_store", "url", "lnk", "zip", "rar", "7z", "exe", "dll", "js", "bat", "com",
    "scr", "msi", "tmp", "bak", "log", "htm", "html", "webloc",
];

const GENERIC_TOKENS: &[&str] = &[
    "svg", "png", "jpg", "jpeg", "dxf", "eps", "pdf", "cdr", "ai", "dwg", "cmx", "plt", "stl",
    "lbrn", "lbrn2", "format", "formats", "formato", "formatos", "file", "files", "archivos",
    "arquivos", "others", "other", "otros", "preview", "previews", "image", "images", "img",
    "imagenes", "imagens", "fotos", "photos", "mm", "x", "version", "ver", "vector", "vectors",
    "vectores", "vetores", "cut", "cutting", "laser", "corte", "and", "y", "e", "de", "for",
    "mockup", "mockups", "commercial", "license", "personal", "use", "jpgs", "pngs", "svgs",
    "dxfs", "cdrs", "pdfs", "eps10", "illustrator", "coreldraw", "corel", "lightburn", "glowforge",
    "cricut", "silhouette", "studio", "cnc", "plasma", "router", "templates", "template",
    "graphics", "swf", "gif", "tif", "tiff", "wmf", "emf", "dwf", "bmp", "webp", "ps", "zip", "sources", "source",
];

pub fn ext_of(name: &str) -> String {
    match name.rsplit_once('.') {
        Some((stem, e)) if !stem.is_empty() && e.len() <= 8 => e.to_ascii_lowercase(),
        _ => String::new(),
    }
}

fn stem_of(name: &str) -> &str {
    match name.rsplit_once('.') {
        Some((stem, e)) if !stem.is_empty() && e.len() <= 8 => stem,
        _ => name,
    }
}

fn tokens(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase())
        .collect()
}

/// A folder whose name only describes a format/thickness ("SVG", "DXF 3mm", "EPS formats").
fn is_generic_dir(name: &str) -> bool {
    let toks: Vec<String> = tokens(name)
        .into_iter()
        .map(|t| t.chars().filter(|c| !c.is_ascii_digit()).collect::<String>())
        .filter(|t| !t.is_empty())
        .collect();
    !toks.is_empty() && toks.iter().all(|t| GENERIC_TOKENS.contains(&t.as_str()))
}

fn is_junk_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.starts_with("._")
        || lower == "thumbs.db"
        || lower == ".ds_store"
        || lower == "desktop.ini"
        || JUNK_EXTS.contains(&ext_of(&lower).as_str())
}

struct FileRec {
    rel: String,
    name: String,
    ext: String,
    size: i64,
}

fn nice_name(dir_name: &str, parent_name: Option<&str>) -> String {
    let letters = dir_name.chars().filter(|c| c.is_alphabetic()).count();
    let base = dir_name.replace('_', " ");
    match parent_name {
        Some(p) if letters < 3 || is_generic_dir(dir_name) => format!("{} · {}", p.replace('_', " "), base),
        _ => base,
    }
}

pub fn build_index(root: &Path, db_path: &Path, skip_dir: &Path) -> Result<()> {
    let t0 = Instant::now();
    let tmp = db_path.with_extension("db.tmp");
    let _ = std::fs::remove_file(&tmp);
    let mut conn = Connection::open(&tmp)?;
    conn.execute_batch(
        r#"
        PRAGMA journal_mode=OFF; PRAGMA synchronous=OFF;
        CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT);
        CREATE TABLE projects(
            id INTEGER PRIMARY KEY, name TEXT, dir TEXT, pack TEXT, split INTEGER,
            file_count INTEGER, total_size INTEGER, exts TEXT, has_image INTEGER,
            search TEXT, dup_key TEXT, dup_count INTEGER DEFAULT 1, is_primary INTEGER DEFAULT 1);
        CREATE TABLE files(
            id INTEGER PRIMARY KEY, project_id INTEGER, name TEXT, ext TEXT, size INTEGER, rel TEXT);
        CREATE TABLE project_cats(project_id INTEGER, cat TEXT);
        CREATE VIRTUAL TABLE projects_fts USING fts5(name, path, files, tokenize='unicode61 remove_diacritics 2');
        "#,
    )?;

    // 1. collect files grouped by owning (non-generic) folder
    let mut groups: BTreeMap<String, Vec<FileRec>> = BTreeMap::new();
    let mut n_files = 0usize;
    let walker = WalkDir::new(root).min_depth(1).follow_links(false).into_iter().filter_entry(|e| {
        let n = e.file_name().to_string_lossy();
        !(e.file_type().is_dir()
            && (n == "__MACOSX" || n.starts_with('.') || e.path() == skip_dir))
    });
    for entry in walker.filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let rel_path = entry.path().strip_prefix(root)?;
        if rel_path.components().count() < 2 || is_junk_file(&name) {
            continue; // top-level loose files (logs) and junk
        }
        // owner = nearest ancestor folder that is not format-only (never above the pack folder)
        let mut owner = rel_path.parent().unwrap().to_path_buf();
        while owner.components().count() > 1
            && is_generic_dir(&owner.file_name().unwrap().to_string_lossy())
        {
            owner = owner.parent().unwrap().to_path_buf();
        }
        let size = entry.metadata().map(|m| m.len() as i64).unwrap_or(0);
        groups.entry(owner.to_string_lossy().to_string()).or_default().push(FileRec {
            rel: rel_path.to_string_lossy().to_string(),
            ext: ext_of(&name),
            name,
            size,
        });
        n_files += 1;
    }
    println!("scanned {n_files} files in {} folders ({:.1?})", groups.len(), t0.elapsed());

    // 2. turn groups into projects
    let tx = conn.transaction()?;
    {
        let mut ins_p = tx.prepare(
            "INSERT INTO projects(name,dir,pack,split,file_count,total_size,exts,has_image,search,dup_key) VALUES(?,?,?,?,?,?,?,?,?,?)",
        )?;
        let mut ins_f = tx.prepare("INSERT INTO files(project_id,name,ext,size,rel) VALUES(?,?,?,?,?)")?;
        let mut ins_cat = tx.prepare("INSERT INTO project_cats(project_id,cat) VALUES(?,?)")?;
        let mut ins_fts = tx.prepare("INSERT INTO projects_fts(rowid,name,path,files) VALUES(?,?,?,?)")?;

        for (dir, files) in groups {
            let dir_path = Path::new(&dir);
            let pack = dir_path.components().next().unwrap().as_os_str().to_string_lossy().to_string();
            let dir_name = dir_path.file_name().unwrap().to_string_lossy().to_string();
            let parent_name = dir_path.parent().and_then(|p| p.file_name()).map(|p| p.to_string_lossy().to_string());

            // split dump folders: many distinct stems, few files per stem
            let mut by_stem: BTreeMap<String, Vec<&FileRec>> = BTreeMap::new();
            for f in &files {
                by_stem.entry(stem_of(&f.name).trim().to_lowercase()).or_default().push(f);
            }
            // each design typically comes in up to ~8 formats (dxf, svg, cdr, ai, eps, pdf, dwg, jpg)
            let split = by_stem.len() > 30 && files.len() <= by_stem.len() * 8;
            let parts: Vec<(String, Vec<&FileRec>)> = if split {
                by_stem
                    .into_values()
                    .map(|v| (stem_of(&v[0].name).replace('_', " ").trim().to_string(), v))
                    .collect()
            } else {
                vec![(nice_name(&dir_name, parent_name.as_deref()), files.iter().collect())]
            };

            for (name, pfiles) in parts {
                let exts: BTreeSet<&str> = pfiles.iter().map(|f| f.ext.as_str()).filter(|e| !e.is_empty()).collect();
                let has_image = exts.iter().any(|e| IMAGE_EXTS.contains(e) || *e == "svg");
                let total: i64 = pfiles.iter().map(|f| f.size).sum();
                let file_names: Vec<&str> = pfiles.iter().map(|f| stem_of(&f.name)).collect();
                let file_text = file_names.join(" ");
                let search = format!("{} {} {}", name, dir, file_text).to_lowercase();
                let mut sig: Vec<(String, i64)> = pfiles.iter().map(|f| (f.name.to_lowercase(), f.size)).collect();
                sig.sort();
                let mut h = DefaultHasher::new();
                sig.hash(&mut h);
                let dup_key = format!("{:016x}", h.finish());
                let exts_s = exts.into_iter().collect::<Vec<_>>().join(" ");
                ins_p.execute(params![name, dir, pack, split as i32, pfiles.len() as i64, total, exts_s, has_image as i32, search, dup_key])?;
                let pid = tx.last_insert_rowid();
                // path without the pack wrapper noise is still useful for "similar" lookups
                ins_fts.execute(params![pid, name, dir.replace(['/', '_', '-'], " "), file_text])?;
                for cat in categorize(&search) {
                    ins_cat.execute(params![pid, cat])?;
                }
                for f in pfiles {
                    ins_f.execute(params![pid, f.name, f.ext, f.size, f.rel])?;
                }
            }
        }
    }
    tx.commit()?;

    conn.execute_batch(
        r#"
        CREATE INDEX files_project ON files(project_id);
        CREATE INDEX projects_dup ON projects(dup_key);
        CREATE INDEX projects_pack ON projects(pack);
        CREATE INDEX project_cats_cat ON project_cats(cat, project_id);
        CREATE INDEX project_cats_pid ON project_cats(project_id);
        UPDATE projects SET dup_count = (SELECT COUNT(*) FROM projects p2 WHERE p2.dup_key = projects.dup_key);
        UPDATE projects SET is_primary = (id = (SELECT MIN(id) FROM projects p2 WHERE p2.dup_key = projects.dup_key));
        "#,
    )?;
    conn.execute("INSERT INTO meta VALUES('root', ?)", [root.to_string_lossy()])?;
    conn.execute("INSERT INTO meta VALUES('indexed_at', datetime('now'))", [])?;
    let (np, nu): (i64, i64) = conn.query_row(
        "SELECT COUNT(*), SUM(is_primary) FROM projects",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    drop(conn);
    std::fs::rename(&tmp, db_path)?;
    println!("indexed {np} projects ({nu} unique) in {:.1?}", t0.elapsed());
    Ok(())
}

/// Order in which files of a project are tried for a thumbnail.
pub fn preview_rank(ext: &str) -> Option<i32> {
    Some(match ext {
        "jpg" | "jpeg" | "jfif" | "png" | "webp" | "gif" | "bmp" => 0,
        "svg" => 1,
        "dxf" => 2,
        "cdr" => 3,
        "pdf" | "ai" => 4,
        "tif" | "tiff" => 5,
        "eps" | "ps" => 6,
        _ => return None,
    })
}

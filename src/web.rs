use crate::categories::CATS;
use crate::index::preview_rank;
use crate::preview::{thumb_for_file, Thumb};
use axum::{
    body::Body,
    extract::{Path as AxPath, Query, Request, State},
    http::{header, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use rusqlite::{params_from_iter, Connection, OpenFlags, OptionalExtension};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;

const INDEX_HTML: &str = include_str!("../static/index.html");

struct App {
    root: PathBuf,
    thumbs: PathBuf,
    db: Mutex<Connection>,
    gen_slots: Semaphore,
}
type St = State<Arc<App>>;

/// Multilingual synonyms: the library mixes English, Spanish, Portuguese and German names.
const SYNONYMS: &[&[&str]] = &[
    &["christmas", "xmas", "navidad", "navideño", "navideños", "navideã±os", "natal", "weihnachten", "noel", "santa"],
    &["furniture", "mueble", "muebles", "moveis", "móveis", "möbel", "moebel"],
    &["lamp", "lampara", "lámpara", "luminaria", "lampe", "lantern", "farol", "laterne"],
    &["box", "caja", "cajas", "caixa", "kiste", "schachtel", "boxes"],
    &["clock", "reloj", "relogio", "relógio", "uhr"],
    &["car", "cars", "carro", "coche", "truck", "camion", "caminhao"],
    &["tree", "arbol", "árbol", "arboles", "arvore", "árvore", "baum"],
    &["heart", "corazon", "corazón", "coracao", "coração", "herz"],
    &["dog", "perro", "cachorro", "hund"],
    &["cat", "gato", "katze"],
    &["horse", "caballo", "cavalo", "pferd"],
    &["bird", "pajaro", "pájaro", "passaro", "pássaro", "vogel"],
    &["fish", "pez", "peixe", "fisch"],
    &["house", "casa", "casita", "haus", "home"],
    &["castle", "castillo", "castelo", "burg", "schloss"],
    &["puzzle", "rompecabezas", "quebra", "puzle", "puzzles"],
    &["flower", "flor", "flores", "blume", "flowers"],
    &["angel", "angeles", "anjo", "engel"],
    &["wine", "vino", "vinho", "wein"],
    &["beer", "cerveza", "cerveja", "bier"],
    &["toy", "juguete", "brinquedo", "spielzeug"],
    &["shelf", "estante", "prateleira", "regal"],
    &["table", "mesa", "tisch"],
    &["chair", "silla", "cadeira", "stuhl"],
    &["wall", "pared", "parede", "wand"],
    &["skull", "calavera", "caveira", "totenkopf"],
    &["dinosaur", "dino", "dinosaurio", "dinossauro", "dinos"],
    &["ship", "barco", "boat", "schiff", "boot"],
    &["plane", "avion", "avião", "aviao", "airplane", "flugzeug"],
    &["moon", "luna", "lua", "mond"],
    &["star", "estrella", "estrela", "stern"],
    &["easter", "pascua", "pascoa", "páscoa", "ostern"],
    &["love", "amor", "liebe", "valentine", "valentines"],
    &["frame", "marco", "moldura", "rahmen"],
    &["holder", "porta", "soporte", "halter"],
    &["candle", "vela", "velas", "kerze"],
    &["bear", "oso", "urso", "bär", "baer"],
    &["owl", "buho", "búho", "coruja", "eule"],
    &["elephant", "elefante", "elefant"],
    &["lion", "leon", "león", "leao", "leão", "löwe", "loewe"],
    &["deer", "ciervo", "venado", "reno", "reindeer", "hirsch"],
    &["wolf", "lobo"],
    &["bike", "bicicleta", "moto", "motorcycle", "motorrad", "fahrrad"],
    &["letters", "letter", "letras", "letra", "buchstaben", "alphabet", "abecedario"],
    &["birthday", "cumpleaños", "cumpleanos", "aniversario", "geburtstag"],
    &["wedding", "boda", "casamento", "hochzeit"],
    &["jewelry", "joyero", "joyas", "schmuck", "jewellery", "jewel"],
    &["keychain", "llavero", "chaveiro", "schlüsselanhänger", "keyring"],
    &["world", "mundo", "welt", "globe"],
    &["church", "iglesia", "igreja", "kirche"],
    &["cross", "cruz", "kreuz"],
    &["mirror", "espejo", "espelho", "spiegel"],
    &["napkin", "servilletero", "servilleta", "guardanapo", "serviette"],
    &["kitchen", "cocina", "cozinha", "küche", "kueche"],
    &["sign", "cartel", "letrero", "placa", "schild"],
    &["coaster", "posavasos", "portacopos", "untersetzer"],
];

const STOP: &[&str] = &[
    "the", "and", "for", "with", "laser", "cut", "cutting", "file", "files", "svg", "dxf", "cdr",
    "eps", "pdf", "pack", "part", "model", "models", "bundle", "design", "designs", "vector",
    "vectors", "vectores", "vetores", "coleccion", "colección", "nueva", "carpeta", "new", "folder",
    "mega", "multilayer", "various", "varios", "laser_cut", "lasercut", "template", "plans",
    "3mm", "4mm", "5mm", "6mm", "copy", "copia", "final", "version", "del", "los", "las", "para",
    "con", "una", "uno", "por", "das", "der", "die", "und", "corte", "cnc", "mdf", "wood",
    "wooden", "madera", "free", "download", "set", "collection", "big", "small", "huge",
    "jpg", "jpeg", "png", "gif", "bmp", "tif", "dwg", "plt", "cmx", "jfif", "preview", "graphics", "dise",
];

fn expand(term: &str) -> Vec<String> {
    let t = term.to_lowercase();
    let mut out = vec![t.clone()];
    for g in SYNONYMS {
        if g.contains(&t.as_str()) {
            out.extend(g.iter().filter(|s| **s != t).map(|s| s.to_string()));
        }
    }
    out
}

fn like_pat(s: &str) -> String {
    let esc = s.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
    format!("%{esc}%")
}

fn err500<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

pub async fn serve(root: PathBuf, db: PathBuf, thumbs: PathBuf, port: u16) -> anyhow::Result<()> {
    let conn = Connection::open_with_flags(&db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let app = Arc::new(App { root, thumbs, db: Mutex::new(conn), gen_slots: Semaphore::new(6) });
    let router = Router::new()
        .route("/", get(|| async { Html(INDEX_HTML) }))
        .route("/api/stats", get(stats))
        .route("/api/search", get(search))
        .route("/api/project/:id", get(project))
        .route("/api/project/:id/similar", get(similar))
        .route("/api/open", post(open))
        .route("/thumb/p/:id", get(thumb_project))
        .route("/thumb/f/:id", get(thumb_file))
        .layer(middleware::from_fn(local_only))
        .with_state(app);
    let addr = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("▶ open http://{addr}");
    axum::serve(listener, router).await?;
    Ok(())
}

/// Reject requests whose Host is not localhost (blocks DNS-rebinding against the "open" endpoint).
async fn local_only(req: Request, next: Next) -> Response {
    let host = req.headers().get(header::HOST).and_then(|h| h.to_str().ok()).unwrap_or("");
    let h = host.rsplit_once(':').map(|(h, _)| h).unwrap_or(host);
    if h == "127.0.0.1" || h == "localhost" {
        next.run(req).await
    } else {
        (StatusCode::FORBIDDEN, "local access only").into_response()
    }
}

async fn stats(State(app): St) -> Result<Json<Value>, (StatusCode, String)> {
    let db = app.db.lock().unwrap();
    let (projects, unique, files, size): (i64, i64, i64, i64) = db
        .query_row(
            "SELECT COUNT(*), SUM(is_primary), SUM(file_count), SUM(total_size) FROM projects",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .map_err(err500)?;
    let mut st = db
        .prepare("SELECT pack, SUM(is_primary) FROM projects GROUP BY pack ORDER BY pack COLLATE NOCASE")
        .map_err(err500)?;
    let packs: Vec<Value> = st
        .query_map([], |r| Ok(json!({"pack": r.get::<_, String>(0)?, "count": r.get::<_, i64>(1)?})))
        .map_err(err500)?
        .filter_map(|r| r.ok())
        .collect();
    let mut st = db
        .prepare("SELECT ext, COUNT(DISTINCT project_id) c FROM files WHERE ext != '' GROUP BY ext ORDER BY c DESC LIMIT 16")
        .map_err(err500)?;
    let exts: Vec<Value> = st
        .query_map([], |r| Ok(json!({"ext": r.get::<_, String>(0)?, "count": r.get::<_, i64>(1)?})))
        .map_err(err500)?
        .filter_map(|r| r.ok())
        .collect();
    let mut st = db
        .prepare("SELECT c.cat, COUNT(*) FROM project_cats c JOIN projects p ON p.id = c.project_id WHERE p.is_primary = 1 GROUP BY c.cat")
        .map_err(err500)?;
    let counts: std::collections::HashMap<String, i64> =
        st.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).map_err(err500)?.filter_map(|r| r.ok()).collect();
    let uncategorized: i64 = db
        .query_row("SELECT COUNT(*) FROM projects WHERE is_primary = 1 AND id NOT IN (SELECT project_id FROM project_cats)", [], |r| r.get(0))
        .map_err(err500)?;
    let mut cats: Vec<Value> = CATS
        .iter()
        .map(|c| json!({"slug": c.slug, "label": c.label, "icon": c.icon, "count": counts.get(c.slug).copied().unwrap_or(0)}))
        .collect();
    cats.push(json!({"slug": "none", "label": "Uncategorized", "icon": "📁", "count": uncategorized}));
    let indexed: String = db.query_row("SELECT value FROM meta WHERE key='indexed_at'", [], |r| r.get(0)).unwrap_or_default();
    Ok(Json(json!({"projects": projects, "unique": unique, "files": files, "bytes": size, "packs": packs, "exts": exts, "categories": cats, "indexed_at": indexed})))
}

#[derive(Deserialize)]
struct SearchQ {
    #[serde(default)]
    q: String,
    #[serde(default)]
    pack: String,
    #[serde(default)]
    ext: String,
    #[serde(default)]
    cat: String,
    #[serde(default)]
    dupes: bool,
    #[serde(default)]
    sort: String,
    #[serde(default)]
    offset: i64,
    #[serde(default)]
    limit: Option<i64>,
}

const ITEM_COLS: &str = "p.id, p.name, p.dir, p.pack, p.exts, p.file_count, p.total_size, p.dup_count, p.has_image";

fn item(r: &rusqlite::Row) -> rusqlite::Result<Value> {
    Ok(json!({
        "id": r.get::<_, i64>(0)?, "name": r.get::<_, String>(1)?, "dir": r.get::<_, String>(2)?,
        "pack": r.get::<_, String>(3)?, "exts": r.get::<_, String>(4)?, "files": r.get::<_, i64>(5)?,
        "size": r.get::<_, i64>(6)?, "dups": r.get::<_, i64>(7)?, "image": r.get::<_, i64>(8)? == 1,
    }))
}

async fn search(State(app): St, Query(q): Query<SearchQ>) -> Result<Json<Value>, (StatusCode, String)> {
    let limit = q.limit.unwrap_or(60).clamp(1, 200);
    let mut wh: Vec<String> = vec!["1=1".into()];
    let mut args: Vec<String> = Vec::new();
    if !q.dupes {
        wh.push("p.is_primary = 1".into());
    }
    if !q.pack.is_empty() {
        wh.push("p.pack = ?".into());
        args.push(q.pack.clone());
    }
    if q.cat == "none" {
        wh.push("p.id NOT IN (SELECT project_id FROM project_cats)".into());
    } else if !q.cat.is_empty() {
        wh.push("p.id IN (SELECT project_id FROM project_cats WHERE cat = ?)".into());
        args.push(q.cat.clone());
    }
    if !q.ext.is_empty() {
        wh.push("(' ' || p.exts || ' ') LIKE ?".into());
        args.push(format!("% {} %", q.ext.to_lowercase()));
    }
    let terms: Vec<&str> = q.q.split_whitespace().take(8).collect();
    let mut score_parts: Vec<String> = Vec::new();
    let mut score_args: Vec<String> = Vec::new();
    for t in &terms {
        let alts = expand(t);
        let ors: Vec<&str> = alts.iter().map(|_| "p.search LIKE ? ESCAPE '\\'").collect();
        wh.push(format!("({})", ors.join(" OR ")));
        for a in &alts {
            args.push(like_pat(a));
            score_parts.push("(p.name LIKE ? ESCAPE '\\')".into());
            score_args.push(like_pat(a));
        }
    }
    let where_sql = wh.join(" AND ");
    let order = if q.sort == "random" {
        "random()".to_string()
    } else if q.sort == "name" || score_parts.is_empty() {
        "p.has_image DESC, p.name COLLATE NOCASE".to_string()
    } else {
        format!("({}) DESC, p.has_image DESC, length(p.name), p.name COLLATE NOCASE", score_parts.join(" + "))
    };
    let db = app.db.lock().unwrap();
    let total: i64 = db
        .query_row(&format!("SELECT COUNT(*) FROM projects p WHERE {where_sql}"), params_from_iter(args.iter()), |r| r.get(0))
        .map_err(err500)?;
    let sql = format!("SELECT {ITEM_COLS} FROM projects p WHERE {where_sql} ORDER BY {order} LIMIT {limit} OFFSET {}", q.offset.max(0));
    let mut all_args = args.clone();
    if q.sort != "random" && q.sort != "name" {
        all_args.extend(score_args);
    }
    let mut st = db.prepare(&sql).map_err(err500)?;
    let items: Vec<Value> = st.query_map(params_from_iter(all_args.iter()), item).map_err(err500)?.filter_map(|r| r.ok()).collect();
    let expanded: Vec<Value> = terms.iter().map(|t| json!(expand(t))).collect();
    Ok(Json(json!({"total": total, "items": items, "expanded": expanded})))
}

async fn project(State(app): St, AxPath(id): AxPath<i64>) -> Result<Json<Value>, (StatusCode, String)> {
    let db = app.db.lock().unwrap();
    let mut p = db
        .query_row(&format!("SELECT {ITEM_COLS}, p.dup_key, p.split FROM projects p WHERE id=?"), [id], |r| {
            let mut v = item(r)?;
            v["dup_key"] = json!(r.get::<_, String>(9)?);
            v["split"] = json!(r.get::<_, i64>(10)? == 1);
            Ok(v)
        })
        .optional()
        .map_err(err500)?
        .ok_or((StatusCode::NOT_FOUND, "no such project".into()))?;
    let mut st = db.prepare("SELECT id, name, ext, size, rel FROM files WHERE project_id=? ORDER BY rel COLLATE NOCASE").map_err(err500)?;
    let files: Vec<Value> = st
        .query_map([id], |r| {
            let ext: String = r.get(2)?;
            Ok(json!({"id": r.get::<_, i64>(0)?, "name": r.get::<_, String>(1)?, "ext": ext,
                      "size": r.get::<_, i64>(3)?, "rel": r.get::<_, String>(4)?, "preview": preview_rank(&ext).is_some()}))
        })
        .map_err(err500)?
        .filter_map(|r| r.ok())
        .collect();
    let mut st = db.prepare("SELECT id, dir, pack FROM projects WHERE dup_key=? AND id != ? ORDER BY id").map_err(err500)?;
    let copies: Vec<Value> = st
        .query_map(rusqlite::params![p["dup_key"].as_str().unwrap_or(""), id], |r| {
            Ok(json!({"id": r.get::<_, i64>(0)?, "dir": r.get::<_, String>(1)?, "pack": r.get::<_, String>(2)?}))
        })
        .map_err(err500)?
        .filter_map(|r| r.ok())
        .collect();
    let mut st = db.prepare("SELECT cat FROM project_cats WHERE project_id=?").map_err(err500)?;
    let slugs: Vec<String> = st.query_map([id], |r| r.get(0)).map_err(err500)?.filter_map(|r| r.ok()).collect();
    let cats: Vec<Value> = CATS
        .iter()
        .filter(|c| slugs.iter().any(|s| s == c.slug))
        .map(|c| json!({"slug": c.slug, "label": c.label, "icon": c.icon}))
        .collect();
    p["categories"] = json!(cats);
    p["file_list"] = json!(files);
    p["copies"] = json!(copies);
    p["root"] = json!(app.root.to_string_lossy());
    Ok(Json(p))
}

fn similar_tokens(name: &str, dir: &str, files: &[String]) -> Vec<String> {
    let mut toks: Vec<String> = Vec::new();
    let add = |s: &str, toks: &mut Vec<String>| {
        for t in s.split(|c: char| !c.is_alphanumeric()) {
            let t = t.to_lowercase();
            if t.chars().count() >= 3 && !t.chars().all(|c| c.is_ascii_digit()) && !STOP.contains(&t.as_str()) && !toks.contains(&t) {
                toks.push(t);
            }
        }
    };
    add(name, &mut toks);
    if toks.len() < 2 {
        // numeric/generic folder names: lean on the file names and parent folder
        for f in files.iter().take(6) {
            add(f.rsplit_once('.').map(|(s, _)| s).unwrap_or(f), &mut toks);
        }
        if let Some(parent) = Path::new(dir).parent().and_then(|p| p.file_name()) {
            add(&parent.to_string_lossy(), &mut toks);
        }
    }
    toks.truncate(8);
    toks
}

async fn similar(State(app): St, AxPath(id): AxPath<i64>) -> Result<Json<Value>, (StatusCode, String)> {
    let db = app.db.lock().unwrap();
    let (name, dir, dup_key): (String, String, String) = db
        .query_row("SELECT name, dir, dup_key FROM projects WHERE id=?", [id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .map_err(|_| (StatusCode::NOT_FOUND, "no such project".to_string()))?;
    let mut st = db.prepare("SELECT name FROM files WHERE project_id=? LIMIT 12").map_err(err500)?;
    let files: Vec<String> = st.query_map([id], |r| r.get(0)).map_err(err500)?.filter_map(|r| r.ok()).collect();
    let toks = similar_tokens(&name, &dir, &files);
    if toks.is_empty() {
        return Ok(Json(json!({"tokens": [], "items": []})));
    }
    let mut alts: Vec<String> = Vec::new();
    for t in &toks {
        for a in expand(t) {
            let clean: String = a.chars().filter(|c| c.is_alphanumeric()).collect();
            if clean.chars().count() >= 3 && !alts.contains(&clean) {
                alts.push(clean);
            }
        }
    }
    let fts_q = alts.iter().map(|a| format!("\"{a}\"*")).collect::<Vec<_>>().join(" OR ");
    let sql = format!(
        "SELECT {ITEM_COLS} FROM projects_fts f JOIN projects p ON p.id = f.rowid
         WHERE projects_fts MATCH ?1 AND p.id != ?2 AND p.dup_key != ?3 AND p.is_primary = 1
         ORDER BY bm25(projects_fts, 10.0, 1.0, 3.0) - p.has_image * 0.5 LIMIT 24"
    );
    let mut st = db.prepare(&sql).map_err(err500)?;
    let items: Vec<Value> = st
        .query_map(rusqlite::params![fts_q, id, dup_key], item)
        .map_err(err500)?
        .filter_map(|r| r.ok())
        .collect();
    Ok(Json(json!({"tokens": toks, "items": items})))
}

#[derive(Deserialize)]
struct OpenReq {
    project: Option<i64>,
    file: Option<i64>,
}

async fn open(State(app): St, Json(req): Json<OpenReq>) -> Result<Json<Value>, (StatusCode, String)> {
    let (rel, reveal): (String, bool) = {
        let db = app.db.lock().unwrap();
        if let Some(fid) = req.file {
            (db.query_row("SELECT rel FROM files WHERE id=?", [fid], |r| r.get(0)).map_err(|_| (StatusCode::NOT_FOUND, "no file".to_string()))?, true)
        } else if let Some(pid) = req.project {
            let (dir, split): (String, i64) = db
                .query_row("SELECT dir, split FROM projects WHERE id=?", [pid], |r| Ok((r.get(0)?, r.get(1)?)))
                .map_err(|_| (StatusCode::NOT_FOUND, "no project".to_string()))?;
            if split == 1 {
                // a stem-split project lives inside a big folder: reveal its first file
                (db.query_row("SELECT rel FROM files WHERE project_id=? ORDER BY id LIMIT 1", [pid], |r| r.get(0)).map_err(err500)?, true)
            } else {
                (dir, false)
            }
        } else {
            return Err((StatusCode::BAD_REQUEST, "project or file required".into()));
        }
    };
    let abs = app.root.join(&rel);
    if !abs.starts_with(&app.root) || !abs.exists() {
        return Err((StatusCode::NOT_FOUND, format!("missing on disk: {rel}")));
    }
    let mut cmd = std::process::Command::new("/usr/bin/open");
    if reveal {
        cmd.arg("-R");
    }
    cmd.arg(&abs).spawn().map_err(err500)?;
    Ok(Json(json!({"opened": abs.to_string_lossy()})))
}

fn thumb_response(t: Thumb) -> Response {
    let (path, ctype) = match &t {
        Thumb::Jpeg(p) => (p, "image/jpeg"),
        Thumb::Svg(p) => (p, "image/svg+xml"),
    };
    match std::fs::read(path) {
        Ok(bytes) => {
            let mut r = Response::new(Body::from(bytes));
            let h = r.headers_mut();
            h.insert(header::CONTENT_TYPE, HeaderValue::from_static(ctype));
            h.insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=604800"));
            // SVGs from the library are untrusted: never let them run script
            h.insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static("default-src 'none'; style-src 'unsafe-inline'; img-src data:"));
            h.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
            r
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn gen_thumb(app: &Arc<App>, fid: i64, rel: String, ext: String) -> Option<Thumb> {
    let _slot = app.gen_slots.acquire().await.ok()?;
    let (root, cache) = (app.root.clone(), app.thumbs.clone());
    tokio::task::spawn_blocking(move || thumb_for_file(&root.join(rel), &ext, &cache, fid)).await.ok().flatten()
}

async fn thumb_file(State(app): St, AxPath(id): AxPath<i64>) -> Response {
    let row: Option<(String, String)> = {
        let db = app.db.lock().unwrap();
        db.query_row("SELECT rel, ext FROM files WHERE id=?", [id], |r| Ok((r.get(0)?, r.get(1)?))).ok()
    };
    let Some((rel, ext)) = row else { return StatusCode::NOT_FOUND.into_response() };
    match gen_thumb(&app, id, rel, ext).await {
        Some(t) => thumb_response(t),
        None => StatusCode::NO_CONTENT.into_response(),
    }
}

async fn thumb_project(State(app): St, AxPath(id): AxPath<i64>) -> Response {
    let mut cands: Vec<(i32, bool, i64, i64, String, String)> = {
        let db = app.db.lock().unwrap();
        let Ok(mut st) = db.prepare("SELECT id, name, ext, size, rel FROM files WHERE project_id=?") else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };
        st.query_map([id], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, i64>(3)?, r.get::<_, String>(4)?)))
            .map(|rows| {
                rows.filter_map(|r| r.ok())
                    .filter_map(|(fid, name, ext, size, rel)| {
                        let rank = preview_rank(&ext)?;
                        let l = name.to_lowercase();
                        let preferred = ["preview", "cover", "mockup", "foto", "photo", "main"].iter().any(|k| l.contains(k));
                        // skip giant images / vectors that would be slow to decode
                        if size > 60_000_000 { return None; }
                        Some((rank, !preferred, -size, fid, rel, ext))
                    })
                    .collect()
            })
            .unwrap_or_default()
    };
    cands.sort();
    for (_, _, _, fid, rel, ext) in cands.into_iter().take(5) {
        if let Some(t) = gen_thumb(&app, fid, rel, ext).await {
            return thumb_response(t);
        }
    }
    StatusCode::NO_CONTENT.into_response()
}

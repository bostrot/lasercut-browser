mod categories;
mod index;
mod preview;
mod web;

use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut root = manifest.parent().unwrap().to_path_buf();
    let mut port: u16 = 8787;
    let mut reindex = false;
    let mut cmd = "serve".to_string();

    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--root" => root = PathBuf::from(args.next().expect("--root needs a path")),
            "--port" => port = args.next().expect("--port needs a number").parse()?,
            "--reindex" => reindex = true,
            "index" | "serve" => cmd = a,
            "-h" | "--help" => {
                println!("lasercut-browser [serve|index] [--root DIR] [--port N] [--reindex]");
                return Ok(());
            }
            other => anyhow::bail!("unknown argument: {other}"),
        }
    }
    let root = root.canonicalize()?;
    let data = manifest.join("data");
    let thumbs = data.join("thumbs");
    std::fs::create_dir_all(&thumbs)?;
    let db = data.join("index.db");

    if cmd == "index" || reindex || !db.exists() {
        println!("indexing {} …", root.display());
        index::build_index(&root, &db, &manifest)?;
        // thumbnails are keyed by file id, which changes on re-index
        let _ = std::fs::remove_dir_all(&thumbs);
        std::fs::create_dir_all(&thumbs)?;
        if cmd == "index" {
            return Ok(());
        }
    }
    tokio::runtime::Runtime::new()?.block_on(web::serve(root, db, thumbs, port))
}

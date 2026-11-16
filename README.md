# Kerf: local laser-cut library browser

A small Rust webserver that indexes everything in `../` (the `extract` folder) into a SQLite
database and gives you a searchable, previewable gallery at **http://127.0.0.1:8787**.

```bash
cd lasercut-browser
cargo run --release                 # indexes on first start, then serves
cargo run --release -- --reindex    # rebuild the index after adding/removing files (~5 s)
cargo run --release -- --port 9000  # other port
```

## What you get

- **Search**: every word must match (SQL `LIKE`) somewhere in the project name, folder path or
  file names. Words are expanded with multilingual synonyms, because the packs mix EN/ES/PT names:
  `clock` also finds *reloj / relógio / uhr*, `christmas` finds *navidad / natal / weihnachten*.
- **Categories**: 28 auto-detected categories (Christmas, Animals, Lamps & Lighting, Clocks,
  3D Puzzles, Multilayer, Mandalas, Vehicles, Furniture…) shown as a strip under the filters.
  Projects are matched by multilingual keywords (EN/ES/PT/DE/RU/TR) in their name, folder path
  and file names, and can be in several categories. The rules live in `src/categories.rs`; edit
  the word lists and `--reindex` to adjust. "Uncategorized" lists whatever matched nothing.
- **Filters**: pack, file type (DXF, SVG, CDR, …), sort (best match / A→Z / shuffle), *Surprise me*.
- **Duplicates collapsed**: many packs contain the same designs. Identical projects (same file
  names + sizes) are shown once with a `×N` badge; tick *show duplicates* to see all copies.
- **Project detail**: preview, full file list (click to preview any file), **Open folder**
  (opens it in Finder), per-file **reveal in Finder**, copy path.
- **Similar projects**: full-text (FTS5/bm25) search on the distinctive words of the project.
- Keyboard: `/` focuses search. Search state lives in the URL, so you can bookmark it.

## Previews (generated lazily, cached in `data/thumbs/`)

| Format | How |
|---|---|
| JPG/PNG/GIF/BMP/WebP/TIFF | resized |
| SVG | served directly (scripts blocked via CSP) |
| DXF | built-in renderer → SVG (lines, polylines with bulges, arcs, circles, ellipses, splines, blocks) |
| CDR | embedded thumbnail (X4+ zip or older RIFF `DISP` chunk) |
| EPS | embedded TIFF preview, else Ghostscript (`-dSAFER`) |
| PDF / AI (PDF-based) | Ghostscript, else QuickLook |

Old Illustrator 7 `.ai` files (PostScript) can't be rendered outside Illustrator; those projects
usually have a CDR/EPS sibling that is used instead.

## How "projects" are detected

- A project is a folder that directly contains files.
- Format-only subfolders (`SVG`, `DXF 3mm`, `EPS formats`, `CDR12`…) are merged into their parent.
- "Dump" folders with >30 unrelated designs (e.g. `400 VETORES`) are split into one project per
  file name. **Open folder** then reveals that design's file inside the big folder.
- Archives (`.zip/.rar/.7z`), `desktop.ini`, `Thumbs.db`, executables etc. are not indexed.

## Files

- `src/index.rs`: filesystem walk → SQLite (`data/index.db`)
- `src/web.rs`: axum server, search/similar/open API, synonyms
- `src/preview.rs`: thumbnail generation incl. DXF/CDR/EPS decoders
- `static/index.html`: the whole UI (embedded into the binary)
- `scripts/extract_nested.sh`: extracts nested archives next to themselves (used once)

The server binds to 127.0.0.1 only and rejects non-localhost `Host` headers.

Requires: Rust, and optionally `brew install ghostscript` for EPS/PDF/AI previews.

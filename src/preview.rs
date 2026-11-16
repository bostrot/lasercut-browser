//! Thumbnail generation. Everything is generated lazily on first request and cached on disk.
//!
//! * raster images  -> resized JPEG
//! * SVG            -> served as-is (with a script-blocking CSP)
//! * DXF            -> own mini renderer -> SVG
//! * CDR            -> embedded thumbnail (zip-based X4+ or RIFF "DISP" chunk)
//! * EPS            -> embedded TIFF preview, else Ghostscript
//! * PDF / AI       -> Ghostscript (-dSAFER), else macOS QuickLook

use image::{DynamicImage, ImageFormat, Rgb, RgbImage};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const THUMB: u32 = 560;

pub enum Thumb {
    Jpeg(PathBuf),
    Svg(PathBuf),
}

pub fn thumb_for_file(abs: &Path, ext: &str, cache: &Path, file_id: i64) -> Option<Thumb> {
    let jpg = cache.join(format!("f{file_id}.jpg"));
    let svg = cache.join(format!("f{file_id}.svg"));
    let none = cache.join(format!("f{file_id}.none"));
    if jpg.exists() {
        return Some(Thumb::Jpeg(jpg));
    }
    if svg.exists() {
        return Some(Thumb::Svg(svg));
    }
    if ext == "svg" {
        return Some(Thumb::Svg(abs.to_path_buf()));
    }
    if none.exists() {
        return None;
    }
    let result: Option<Thumb> = match ext {
        "dxf" => dxf_to_svg(abs).and_then(|s| std::fs::write(&svg, s).ok()).map(|_| Thumb::Svg(svg.clone())),
        _ => {
            let img = match ext {
                "jpg" | "jpeg" | "jfif" | "png" | "gif" | "bmp" | "webp" | "tif" | "tiff" => open_image(abs),
                "cdr" => cdr_thumb(abs),
                "eps" | "ps" => eps_tiff_preview(abs).or_else(|| ghostscript(abs, cache, file_id)),
                // pre-CS AI files are PostScript needing Adobe procsets: nothing but Illustrator renders them
                "pdf" | "ai" if starts_with(abs, b"%PDF") => {
                    ghostscript(abs, cache, file_id).or_else(|| quicklook(abs, cache, file_id))
                }
                _ => None,
            };
            img.and_then(|i| save_thumb(i, &jpg)).map(|_| Thumb::Jpeg(jpg.clone()))
        }
    };
    if result.is_none() {
        let _ = std::fs::write(&none, b"");
    }
    result
}

fn open_image(p: &Path) -> Option<DynamicImage> {
    image::ImageReader::open(p).ok()?.with_guessed_format().ok()?.decode().ok()
}

fn save_thumb(img: DynamicImage, out: &Path) -> Option<()> {
    let img = img.thumbnail(THUMB, THUMB);
    // flatten transparency on white: laser designs are mostly black-on-transparent
    let rgba = img.to_rgba8();
    let mut rgb = RgbImage::new(rgba.width(), rgba.height());
    for (x, y, p) in rgba.enumerate_pixels() {
        let a = p[3] as u32;
        let blend = |c: u8| ((c as u32 * a + 255 * (255 - a)) / 255) as u8;
        rgb.put_pixel(x, y, Rgb([blend(p[0]), blend(p[1]), blend(p[2])]));
    }
    // a fully white image means nothing useful was rendered
    if rgb.pixels().all(|p| p.0.iter().all(|&c| c > 250)) {
        return None;
    }
    rgb.save_with_format(out, ImageFormat::Jpeg).ok()
}

fn run_with_timeout(mut cmd: Command, secs: u64) -> bool {
    let Ok(mut child) = cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn() else {
        return false;
    };
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(st)) => return st.success(),
            Ok(None) if start.elapsed() > Duration::from_secs(secs) => {
                let _ = child.kill();
                let _ = child.wait();
                return false;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(40)),
            Err(_) => return false,
        }
    }
}

fn ghostscript(abs: &Path, cache: &Path, id: i64) -> Option<DynamicImage> {
    let out = cache.join(format!("gs{id}.png"));
    let mut cmd = Command::new(gs_bin());
    cmd.args([
        "-q", "-dSAFER", "-dBATCH", "-dNOPAUSE", "-dNOPROMPT", "-dFirstPage=1", "-dLastPage=1",
        "-dEPSCrop", "-sDEVICE=png16m", "-r36", "-dTextAlphaBits=4", "-dGraphicsAlphaBits=4",
    ]);
    cmd.arg(format!("-sOutputFile={}", out.display())).arg(abs);
    let ok = run_with_timeout(cmd, 25);
    let img = if ok { open_image(&out) } else { None };
    let _ = std::fs::remove_file(&out);
    img
}

fn gs_bin() -> &'static str {
    if Path::new("/opt/homebrew/bin/gs").exists() { "/opt/homebrew/bin/gs" } else { "gs" }
}

fn quicklook(abs: &Path, cache: &Path, id: i64) -> Option<DynamicImage> {
    let dir = cache.join(format!("ql{id}"));
    std::fs::create_dir_all(&dir).ok()?;
    let mut cmd = Command::new("/usr/bin/qlmanage");
    cmd.args(["-t", "-s", &THUMB.to_string(), "-o"]).arg(&dir).arg(abs);
    run_with_timeout(cmd, 10);
    let img = std::fs::read_dir(&dir).ok()?.filter_map(|e| e.ok()).find_map(|e| open_image(&e.path()));
    let _ = std::fs::remove_dir_all(&dir);
    img
}

fn starts_with(p: &Path, magic: &[u8]) -> bool {
    let mut buf = vec![0u8; 1024];
    let Ok(mut f) = std::fs::File::open(p) else { return false };
    let n = f.read(&mut buf).unwrap_or(0);
    buf[..n].windows(magic.len()).any(|w| w == magic)
}

// ---------------------------------------------------------------- EPS

fn eps_tiff_preview(abs: &Path) -> Option<DynamicImage> {
    let mut f = std::fs::File::open(abs).ok()?;
    let mut hdr = [0u8; 30];
    f.read_exact(&mut hdr).ok()?;
    if hdr[..4] != [0xC5, 0xD0, 0xD3, 0xC6] {
        return None;
    }
    let u = |i: usize| u32::from_le_bytes(hdr[i..i + 4].try_into().unwrap()) as usize;
    let (off, len) = (u(20), u(24));
    if len == 0 {
        return None;
    }
    let data = std::fs::read(abs).ok()?;
    let tiff = data.get(off..off + len)?;
    image::load_from_memory_with_format(tiff, ImageFormat::Tiff).ok()
}

// ---------------------------------------------------------------- CDR

fn cdr_thumb(abs: &Path) -> Option<DynamicImage> {
    let data = std::fs::read(abs).ok()?;
    if data.starts_with(b"PK") {
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&data)).ok()?;
        let names: Vec<String> = zip.file_names().map(String::from).collect();
        let pick = names
            .iter()
            .filter(|n| {
                let l = n.to_lowercase();
                (l.contains("thumbnail") || l.contains("preview")) && (l.ends_with(".bmp") || l.ends_with(".png"))
            })
            .max_by_key(|n| zip.by_name(n).map(|f| f.size()).unwrap_or(0))?
            .clone();
        let mut buf = Vec::new();
        zip.by_name(&pick).ok()?.read_to_end(&mut buf).ok()?;
        return image::load_from_memory(&buf).ok();
    }
    if data.starts_with(b"RIFF") {
        let pos = data.windows(4).position(|w| w == b"DISP")?;
        let chunk_len = u32::from_le_bytes(data.get(pos + 4..pos + 8)?.try_into().ok()?) as usize;
        let body = data.get(pos + 8..(pos + 8 + chunk_len).min(data.len()))?;
        // the DIB starts with a BITMAPINFOHEADER (size field == 40) a few bytes in
        let dib_start = (0..16).find(|&i| body.get(i..i + 4) == Some(&[40, 0, 0, 0]))?;
        let dib = &body[dib_start..];
        let bpp = u16::from_le_bytes(dib.get(14..16)?.try_into().ok()?) as u32;
        let used = u32::from_le_bytes(dib.get(32..36)?.try_into().ok()?);
        let colors = if bpp <= 8 { if used > 0 { used } else { 1 << bpp } } else { used };
        let offset = 14 + 40 + colors * 4;
        let mut bmp = Vec::with_capacity(dib.len() + 14);
        bmp.extend_from_slice(b"BM");
        bmp.extend_from_slice(&((dib.len() + 14) as u32).to_le_bytes());
        bmp.extend_from_slice(&[0, 0, 0, 0]);
        bmp.extend_from_slice(&offset.to_le_bytes());
        bmp.extend_from_slice(dib);
        return image::load_from_memory_with_format(&bmp, ImageFormat::Bmp).ok();
    }
    None
}

// ---------------------------------------------------------------- DXF

type Pt = (f64, f64);

#[derive(Clone, Copy)]
struct Xf {
    m: [f64; 6], // x' = a*x + b*y + e ; y' = c*x + d*y + f
}
impl Xf {
    fn id() -> Self {
        Xf { m: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0] }
    }
    fn apply(&self, p: Pt) -> Pt {
        let [a, b, c, d, e, f] = self.m;
        (a * p.0 + b * p.1 + e, c * p.0 + d * p.1 + f)
    }
    fn then(&self, o: &Xf) -> Xf {
        // returns o ∘ self  (apply self first, then o)
        let [a1, b1, c1, d1, e1, f1] = self.m;
        let [a2, b2, c2, d2, e2, f2] = o.m;
        Xf {
            m: [
                a2 * a1 + b2 * c1,
                a2 * b1 + b2 * d1,
                c2 * a1 + d2 * c1,
                c2 * b1 + d2 * d1,
                a2 * e1 + b2 * f1 + e2,
                c2 * e1 + d2 * f1 + f2,
            ],
        }
    }
}

struct Ent {
    kind: String,
    codes: Vec<(i32, String)>,
}
impl Ent {
    fn f(&self, code: i32) -> Option<f64> {
        self.codes.iter().find(|(c, _)| *c == code).and_then(|(_, v)| v.trim().parse().ok())
    }
    fn fd(&self, code: i32, d: f64) -> f64 {
        self.f(code).unwrap_or(d)
    }
    fn all(&self, code: i32) -> Vec<f64> {
        self.codes.iter().filter(|(c, _)| *c == code).filter_map(|(_, v)| v.trim().parse().ok()).collect()
    }
    fn s(&self, code: i32) -> Option<&str> {
        self.codes.iter().find(|(c, _)| *c == code).map(|(_, v)| v.trim())
    }
}

fn arc_pts(c: Pt, r: f64, a0: f64, sweep: f64) -> Vec<Pt> {
    let n = ((sweep.abs() / std::f64::consts::TAU) * 72.0).ceil().max(4.0) as usize;
    (0..=n).map(|i| {
        let a = a0 + sweep * i as f64 / n as f64;
        (c.0 + r * a.cos(), c.1 + r * a.sin())
    }).collect()
}

fn bulge_pts(p1: Pt, p2: Pt, b: f64) -> Vec<Pt> {
    if b.abs() < 1e-9 {
        return vec![p1, p2];
    }
    let (dx, dy) = (p2.0 - p1.0, p2.1 - p1.1);
    let mid = ((p1.0 + p2.0) / 2.0, (p1.1 + p2.1) / 2.0);
    let k = (1.0 - b * b) / (4.0 * b);
    let c = (mid.0 - dy * k, mid.1 + dx * k);
    let r = ((p1.0 - c.0).powi(2) + (p1.1 - c.1).powi(2)).sqrt();
    let a0 = (p1.1 - c.1).atan2(p1.0 - c.0);
    arc_pts(c, r, a0, 4.0 * b.atan())
}

fn bspline(ctrl: &[Pt], knots: &[f64], deg: usize) -> Vec<Pt> {
    let n = ctrl.len();
    if n < 2 || deg == 0 || knots.len() != n + deg + 1 {
        return ctrl.to_vec();
    }
    let (t0, t1) = (knots[deg], knots[n]);
    let samples = (n * 8).clamp(16, 400);
    let mut out = Vec::with_capacity(samples + 1);
    for s in 0..=samples {
        let t = t0 + (t1 - t0) * s as f64 / samples as f64;
        let mut k = deg;
        while k < n - 1 && t >= knots[k + 1] {
            k += 1;
        }
        let mut d: Vec<Pt> = (0..=deg).map(|j| ctrl[j + k - deg]).collect();
        for r in 1..=deg {
            for j in (r..=deg).rev() {
                let i = j + k - deg;
                let den = knots[i + deg + 1 - r] - knots[i];
                let a = if den.abs() < 1e-12 { 0.0 } else { (t - knots[i]) / den };
                d[j] = ((1.0 - a) * d[j - 1].0 + a * d[j].0, (1.0 - a) * d[j - 1].1 + a * d[j].1);
            }
        }
        out.push(d[deg]);
    }
    out
}

fn parse_dxf(text: &str) -> (Vec<Ent>, std::collections::HashMap<String, Vec<Ent>>) {
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    let mut section = String::new();
    let mut entities = Vec::new();
    let mut blocks: std::collections::HashMap<String, Vec<Ent>> = Default::default();
    let mut cur_block: Option<String> = None;
    let mut cur: Option<Ent> = None;
    let push = |e: Ent, section: &str, cur_block: &Option<String>, entities: &mut Vec<Ent>, blocks: &mut std::collections::HashMap<String, Vec<Ent>>| {
        if section == "ENTITIES" {
            entities.push(e);
        } else if section == "BLOCKS" {
            if let Some(b) = cur_block {
                blocks.entry(b.clone()).or_default().push(e);
            }
        }
    };
    while i + 1 < lines.len() {
        let code: i32 = match lines[i].trim().parse() {
            Ok(c) => c,
            Err(_) => { i += 1; continue; }
        };
        let val = lines[i + 1];
        i += 2;
        if code == 0 {
            if let Some(e) = cur.take() {
                if e.kind == "BLOCK" {
                    cur_block = e.s(2).map(String::from);
                } else {
                    push(e, &section, &cur_block, &mut entities, &mut blocks);
                }
            }
            let v = val.trim();
            match v {
                "SECTION" | "ENDSEC" | "EOF" => {
                    if v != "SECTION" { section.clear(); }
                }
                "ENDBLK" => cur_block = None,
                _ => cur = Some(Ent { kind: v.to_string(), codes: Vec::new() }),
            }
        } else if code == 2 && cur.is_none() && section.is_empty() {
            section = val.trim().to_string();
        } else if let Some(e) = cur.as_mut() {
            e.codes.push((code, val.to_string()));
        }
    }
    if let Some(e) = cur.take() {
        push(e, &section, &cur_block, &mut entities, &mut blocks);
    }
    (entities, blocks)
}

fn ents_to_polys(
    ents: &[Ent],
    blocks: &std::collections::HashMap<String, Vec<Ent>>,
    xf: &Xf,
    depth: u32,
    out: &mut Vec<Vec<Pt>>,
    budget: &mut usize,
) {
    let mut i = 0;
    while i < ents.len() {
        if *budget == 0 {
            return;
        }
        let e = &ents[i];
        i += 1;
        let mut poly: Vec<Pt> = Vec::new();
        match e.kind.as_str() {
            "LINE" => poly = vec![(e.fd(10, 0.0), e.fd(20, 0.0)), (e.fd(11, 0.0), e.fd(21, 0.0))],
            "CIRCLE" => poly = arc_pts((e.fd(10, 0.0), e.fd(20, 0.0)), e.fd(40, 0.0), 0.0, std::f64::consts::TAU),
            "ARC" => {
                let (a0, a1) = (e.fd(50, 0.0).to_radians(), e.fd(51, 360.0).to_radians());
                let mut sw = a1 - a0;
                if sw <= 0.0 { sw += std::f64::consts::TAU; }
                poly = arc_pts((e.fd(10, 0.0), e.fd(20, 0.0)), e.fd(40, 0.0), a0, sw);
            }
            "ELLIPSE" => {
                let c = (e.fd(10, 0.0), e.fd(20, 0.0));
                let mj = (e.fd(11, 1.0), e.fd(21, 0.0));
                let ratio = e.fd(40, 1.0);
                let (p0, mut p1) = (e.fd(41, 0.0), e.fd(42, std::f64::consts::TAU));
                if p1 <= p0 { p1 += std::f64::consts::TAU; }
                let mn = (-mj.1 * ratio, mj.0 * ratio);
                let n = 72;
                poly = (0..=n).map(|k| {
                    let t = p0 + (p1 - p0) * k as f64 / n as f64;
                    (c.0 + mj.0 * t.cos() + mn.0 * t.sin(), c.1 + mj.1 * t.cos() + mn.1 * t.sin())
                }).collect();
            }
            "LWPOLYLINE" => {
                // walk codes in order so bulges stay attached to their vertex
                let mut verts: Vec<(Pt, f64)> = Vec::new();
                for (c, v) in &e.codes {
                    let num = v.trim().parse::<f64>().unwrap_or(0.0);
                    match c {
                        10 => verts.push(((num, 0.0), 0.0)),
                        20 => if let Some(l) = verts.last_mut() { l.0 .1 = num },
                        42 => if let Some(l) = verts.last_mut() { l.1 = num },
                        _ => {}
                    }
                }
                let closed = (e.fd(70, 0.0) as i64) & 1 == 1;
                poly = polyline_with_bulges(&verts, closed);
            }
            "POLYLINE" => {
                let closed = (e.fd(70, 0.0) as i64) & 1 == 1;
                let mut verts = Vec::new();
                while i < ents.len() && ents[i].kind == "VERTEX" {
                    verts.push(((ents[i].fd(10, 0.0), ents[i].fd(20, 0.0)), ents[i].fd(42, 0.0)));
                    i += 1;
                }
                if i < ents.len() && ents[i].kind == "SEQEND" { i += 1; }
                poly = polyline_with_bulges(&verts, closed);
            }
            "SPLINE" => {
                let (fx, fy) = (e.all(11), e.all(21));
                let (cx, cy) = (e.all(10), e.all(20));
                if cx.len() >= 2 && cx.len() == cy.len() {
                    let ctrl: Vec<Pt> = cx.into_iter().zip(cy).collect();
                    poly = bspline(&ctrl, &e.all(40), e.fd(71, 3.0) as usize);
                } else if fx.len() >= 2 && fx.len() == fy.len() {
                    poly = fx.into_iter().zip(fy).collect();
                }
            }
            "INSERT" if depth < 5 => {
                if let Some(b) = e.s(2).and_then(|n| blocks.get(n)) {
                    let (sx, sy) = (e.fd(41, 1.0), e.fd(42, 1.0));
                    let r = e.fd(50, 0.0).to_radians();
                    let (c, s) = (r.cos(), r.sin());
                    let local = Xf { m: [c * sx, -s * sy, s * sx, c * sy, e.fd(10, 0.0), e.fd(20, 0.0)] };
                    ents_to_polys(b, blocks, &local.then(xf), depth + 1, out, budget);
                }
            }
            _ => {}
        }
        if poly.len() >= 2 {
            *budget = budget.saturating_sub(poly.len());
            out.push(poly.into_iter().map(|p| xf.apply(p)).collect());
        }
    }
}

fn polyline_with_bulges(verts: &[(Pt, f64)], closed: bool) -> Vec<Pt> {
    let mut pts = Vec::new();
    let n = verts.len();
    if n < 2 {
        return pts;
    }
    let segs = if closed { n } else { n - 1 };
    for k in 0..segs {
        let (p1, b) = verts[k];
        let p2 = verts[(k + 1) % n].0;
        let seg = bulge_pts(p1, p2, b);
        if pts.is_empty() { pts.extend(seg) } else { pts.extend(seg.into_iter().skip(1)) }
    }
    pts
}

pub fn dxf_to_svg(abs: &Path) -> Option<String> {
    let bytes = std::fs::read(abs).ok()?;
    if bytes.starts_with(b"AutoCAD Binary DXF") {
        return None;
    }
    let text = String::from_utf8_lossy(&bytes);
    let (ents, blocks) = parse_dxf(&text);
    let mut polys = Vec::new();
    let mut budget = 400_000usize;
    ents_to_polys(&ents, &blocks, &Xf::id(), 0, &mut polys, &mut budget);
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for p in polys.iter().flatten() {
        if !p.0.is_finite() || !p.1.is_finite() { continue; }
        x0 = x0.min(p.0); y0 = y0.min(p.1); x1 = x1.max(p.0); y1 = y1.max(p.1);
    }
    if polys.is_empty() || x1 <= x0 || y1 <= y0 {
        return None;
    }
    let (w, h) = (x1 - x0, y1 - y0);
    let pad = w.max(h) * 0.03;
    let mut d = String::with_capacity(polys.len() * 64);
    for poly in &polys {
        for (k, p) in poly.iter().enumerate() {
            use std::fmt::Write;
            // flip Y: DXF is y-up, SVG is y-down
            let _ = write!(d, "{}{:.2} {:.2}", if k == 0 { "M" } else { "L" }, p.0 - x0, y1 - p.1);
        }
    }
    Some(format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{:.2} {:.2} {:.2} {:.2}"><rect x="{:.2}" y="{:.2}" width="100%" height="100%" fill="#fff"/><path d="{}" fill="none" stroke="#1d2433" stroke-width="1.1" vector-effect="non-scaling-stroke" stroke-linejoin="round"/></svg>"##,
        -pad, -pad, w + 2.0 * pad, h + 2.0 * pad, -pad, -pad, d
    ))
}

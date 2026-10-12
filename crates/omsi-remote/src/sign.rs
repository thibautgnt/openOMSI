use glam::Vec2;
use omsi_sim::{VehicleInstance, VehicleType};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const FRONT: &[&str] = &["matrix", "ziel", "anzeige", "front", "dest", "display", "annax", "krueger", "kruger", "lawo", "mobitec", "aesys", "buse", "hanover", "led"];
const NOT_FRONT: &[&str] = &["innen", "inside", "interior", "seite", "side", "heck", "rear", "back", "hinten", "ibis", "drucker", "printer", "cover", "leer", "kasten", "tacho", "cockpit", "dash"];

enum Source {
    Mask { script: usize, colour: String, freetex: Option<String> },
    Script(usize),
    Text(usize),
}

struct Part {
    source: Source,
    lo: Vec2,
    hi: Vec2,
    x: f32,
    y: f32,
}

pub struct Sign {
    parts: Vec<Part>,
    dirs: Vec<PathBuf>,
    colours: HashMap<String, Option<Arc<omsi_texture::Image>>>,
    last: u64,
}

fn slot_face(data: &omsi_geometry::MeshData, slot: u32) -> Option<(Vec2, Vec2, f32, f32)> {
    let mut indices: Vec<u32> = Vec::new();
    let mut ahead: Vec<u32> = Vec::new();
    for &(first, count, s) in data.ranges.iter().filter(|r| r.2 == slot) {
        let Some(tris) = data.indices.get(first as usize..(first + count) as usize) else { continue };
        for t in tris.chunks_exact(3) {
            let p = |i: u32| data.positions.get(i as usize).copied().unwrap_or_default();
            let n = (p(t[1]) - p(t[0])).cross(p(t[2]) - p(t[0])).normalize_or_zero();
            indices.extend_from_slice(t);
            if n.y.abs() > 0.7 {
                ahead.extend_from_slice(t);
            }
        }
        let _ = s;
    }
    let chosen = if ahead.is_empty() { indices } else { ahead };
    let ranges = [(0u32, chosen.len() as u32, slot)];
    let face = crate::skins::front_face(&data.positions, &data.uvs, &chosen, &ranges)?;
    let n = data.positions.len().max(1) as f32;
    let x = data.positions.iter().map(|p| p.x).sum::<f32>() / n;
    let y = data.positions.iter().map(|p| p.y).sum::<f32>() / n;
    Some((face.lo, face.hi, x, y))
}

fn script_index(s: &str) -> Option<usize> {
    s.trim().strip_prefix("\\S:").or_else(|| s.trim().strip_prefix("\\s:")).and_then(|n| n.trim().parse().ok())
}

fn sample(w: u32, h: u32, data: &[u8], u: f32, v: f32) -> [u8; 4] {
    let x = ((u.rem_euclid(1.0) * w as f32) as u32).min(w.saturating_sub(1));
    let y = ((v.rem_euclid(1.0) * h as f32) as u32).min(h.saturating_sub(1));
    let i = ((y * w + x) * 4) as usize;
    data.get(i..i + 4).map(|p| [p[0], p[1], p[2], p[3]]).unwrap_or([0; 4])
}

fn size_of(lo: Vec2, hi: Vec2, w: u32, h: u32) -> (u32, u32) {
    let pw = (((hi.x - lo.x) * w as f32).round() as u32).clamp(1, 1024);
    let ph = (((hi.y - lo.y) * h as f32).round() as u32).clamp(1, 512);
    (pw, ph)
}

impl Sign {
    pub fn describe(&self) -> Vec<String> {
        self.parts
            .iter()
            .map(|p| {
                let what = match &p.source {
                    Source::Mask { script, colour, freetex } => format!("mask S:{script} colour {colour} {freetex:?}"),
                    Source::Script(n) => format!("script {n}"),
                    Source::Text(n) => format!("text {n}"),
                };
                format!("{what} uv {:?}-{:?} x {:.2} y {:.2}", p.lo, p.hi, p.x, p.y)
            })
            .collect()
    }

    pub fn find(ty: &VehicleType, root: &Path) -> Option<Sign> {
        let mut parts: Vec<(i32, Part)> = Vec::new();
        for (i, vm) in ty.meshes.iter().enumerate() {
            let Some(def) = ty.model.meshes.get(vm.def_index) else { continue };
            if def.viewpoint == 2 {
                continue;
            }
            let file = def.file.to_ascii_lowercase();
            if NOT_FRONT.iter().any(|n| file.contains(n)) {
                continue;
            }
            let named = FRONT.iter().any(|n| file.contains(n)) as i32;
            let Some(data) = ty.mesh_data(i) else { continue };
            for o in &vm.overrides {
                let Some(slot) = omsi_sim::vehicle::override_slot(&vm.materials, o) else { continue };
                let source = if let Some(n) = o.transmap.as_deref().and_then(script_index) {
                    Source::Mask { script: n, colour: o.texture.clone(), freetex: o.freetex.as_ref().map(|f| f.1.clone()) }
                } else if let Some(n) = o.use_script_texture.filter(|n| *n >= 0) {
                    Source::Script(n as usize)
                } else if let Some(n) = o.use_text_texture.filter(|n| *n >= 0) {
                    Source::Text(n as usize)
                } else {
                    continue;
                };
                let Some((lo, hi, x, y)) = slot_face(&data, slot as u32) else { continue };
                parts.push((named, Part { source, lo, hi, x, y }));
            }
        }
        parts.dedup_by(|a, b| a.1.lo == b.1.lo && a.1.hi == b.1.hi);
        let best = parts.iter().map(|(named, _)| *named).max()?;
        parts.retain(|(named, _)| *named == best);
        let front = parts.iter().map(|(_, p)| p.y).fold(f32::MIN, f32::max);
        let mut parts: Vec<Part> = parts.into_iter().map(|(_, p)| p).filter(|p| p.y > front - 0.3).collect();
        parts.sort_by(|a, b| b.x.total_cmp(&a.x));
        let mut dirs = ty.texture_dirs(root);
        dirs.push(ty.def.dir().to_path_buf());
        (!parts.is_empty()).then_some(Sign { parts, dirs, colours: HashMap::new(), last: 0 })
    }

    fn colour(&mut self, name: &str) -> Option<Arc<omsi_texture::Image>> {
        let key = name.trim().to_ascii_lowercase();
        if let Some(c) = self.colours.get(&key) {
            return c.clone();
        }
        let dirs: Vec<&Path> = self.dirs.iter().map(PathBuf::as_path).collect();
        let img = omsi_texture::find_texture(name.trim(), &dirs).and_then(|p| omsi_texture::decode_file(&p).ok()).map(Arc::new);
        self.colours.insert(key, img.clone());
        img
    }

    fn part(&mut self, i: usize, v: &VehicleInstance) -> Option<image::RgbaImage> {
        let (lo, hi) = (self.parts[i].lo, self.parts[i].hi);
        let (lo, hi) = (lo - lo.floor(), hi - lo.floor());
        match &self.parts[i].source {
            Source::Script(n) => {
                let st = v.host.script_textures.get(*n)?;
                let (w, h) = size_of(lo, hi, st.width, st.height);
                Some(image::RgbaImage::from_fn(w, h, |x, y| {
                    let p = sample(st.width, st.height, &st.rgba, lo.x + (x as f32 + 0.5) / w as f32 * (hi.x - lo.x), lo.y + (y as f32 + 0.5) / h as f32 * (hi.y - lo.y));
                    image::Rgba([p[0], p[1], p[2], 255])
                }))
            }
            Source::Text(n) => {
                let tt = v.text_textures.get(*n)?;
                let rgba = tt.image(tt.last_text.as_deref().unwrap_or(""));
                let (tw, th) = (tt.def.width.max(1) as u32, tt.def.height.max(1) as u32);
                let (w, h) = size_of(lo, hi, tw, th);
                Some(image::RgbaImage::from_fn(w, h, |x, y| {
                    let p = sample(tw, th, &rgba, lo.x + (x as f32 + 0.5) / w as f32 * (hi.x - lo.x), lo.y + (y as f32 + 0.5) / h as f32 * (hi.y - lo.y));
                    let a = p[3] as u32;
                    image::Rgba([(p[0] as u32 * a / 255) as u8, (p[1] as u32 * a / 255) as u8, (p[2] as u32 * a / 255) as u8, 255])
                }))
            }
            Source::Mask { script, colour, freetex } => {
                let (script, colour, freetex) = (*script, colour.clone(), freetex.clone());
                let name = freetex.map(|var| v.str_var(&var)).filter(|s| !s.trim().is_empty()).unwrap_or(colour);
                let tint = self.colour(&name);
                let st = v.host.script_textures.get(script)?;
                let (w, h) = size_of(lo, hi, st.width, st.height);
                Some(image::RgbaImage::from_fn(w, h, |x, y| {
                    let (u, vv) = (lo.x + (x as f32 + 0.5) / w as f32 * (hi.x - lo.x), lo.y + (y as f32 + 0.5) / h as f32 * (hi.y - lo.y));
                    let a = sample(st.width, st.height, &st.rgba, u, vv)[3] as u32;
                    let c = tint.as_ref().map_or([255, 176, 0, 255], |t| sample(t.width, t.height, &t.rgba, u, vv));
                    image::Rgba([(c[0] as u32 * a / 255) as u8, (c[1] as u32 * a / 255) as u8, (c[2] as u32 * a / 255) as u8, 255])
                }))
            }
        }
    }

    pub fn render(&mut self, v: &VehicleInstance) -> Option<Vec<u8>> {
        let parts: Vec<image::RgbaImage> = (0..self.parts.len()).filter_map(|i| self.part(i, v)).collect();
        let height = parts.iter().map(|p| p.height()).max()?;
        let width: u32 = parts.iter().map(|p| p.width()).sum::<u32>() + 4 * (parts.len() as u32 - 1);
        let mut out = image::RgbaImage::from_pixel(width, height, image::Rgba([0, 0, 0, 255]));
        let mut x = 0;
        for p in &parts {
            image::imageops::overlay(&mut out, p, x as i64, ((height - p.height()) / 2) as i64);
            x += p.width() + 4;
        }
        let hash = out.as_raw().chunks(8).fold(1469598103934665603u64, |h, c| (h ^ c.iter().fold(0u64, |a, b| a << 8 | *b as u64)).wrapping_mul(1099511628211));
        if hash == self.last {
            return None;
        }
        self.last = hash;
        let mut png = Vec::new();
        image::DynamicImage::ImageRgba8(out).write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).ok()?;
        Some(png)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mask_slot_names_its_script_texture() {
        assert_eq!(script_index("\\S:1"), Some(1));
        assert_eq!(script_index(" \\S:12 "), Some(12));
        assert_eq!(script_index("matrix_alpha.bmp"), None);
    }

    #[test]
    fn samples_wrap_and_stay_inside() {
        let data = [1, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(sample(2, 1, &data, 0.75, 0.0), [5, 6, 7, 8]);
        assert_eq!(sample(2, 1, &data, 1.25, 0.0), [1, 2, 3, 4]);
        assert_eq!(sample(2, 1, &data, 0.999, 0.999), [5, 6, 7, 8]);
    }
}

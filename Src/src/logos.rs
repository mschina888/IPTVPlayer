use slint::{Rgba8Pixel, SharedPixelBuffer};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

pub type Pending = Arc<Mutex<Vec<(usize, SharedPixelBuffer<Rgba8Pixel>)>>>;

const THUMB: u32 = 40;

fn fnv(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn to_buffer(img: &image::DynamicImage) -> SharedPixelBuffer<Rgba8Pixel> {
    let rgba = img.to_rgba8();
    SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(rgba.as_raw(), rgba.width(), rgba.height())
}

fn load_one(url: &str) -> Option<SharedPixelBuffer<Rgba8Pixel>> {
    let path = crate::settings::cache_dir().join(format!("{:016x}.png", fnv(url)));
    if let Ok(img) = image::open(&path) {
        return Some(to_buffer(&img));
    }
    let bytes = crate::net::bytes(url, 3 * 1024 * 1024).ok()?;
    let img = image::load_from_memory(&bytes).ok()?;
    let thumb = img.resize(THUMB, THUMB, image::imageops::FilterType::Triangle);
    let _ = thumb.save_with_format(&path, image::ImageFormat::Png);
    Some(to_buffer(&thumb))
}

/// Downloads/decodes channel logos in background threads; results are put into `out`.
pub fn spawn(items: Vec<(usize, String)>, out: Pending, current: Arc<AtomicU64>, generation: u64) {
    let queue = Arc::new(Mutex::new(items.into_iter().collect::<VecDeque<_>>()));
    for _ in 0..3 {
        let (q, out, current) = (queue.clone(), out.clone(), current.clone());
        std::thread::spawn(move || loop {
            if current.load(Ordering::Relaxed) != generation {
                break;
            }
            let item = q.lock().unwrap().pop_front();
            let Some((idx, url)) = item else { break };
            if let Some(buf) = load_one(&url) {
                out.lock().unwrap().push((idx, buf));
            }
            std::thread::sleep(std::time::Duration::from_millis(30));
        });
    }
}

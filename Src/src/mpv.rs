//! Minimal libmpv client (loaded dynamically from libmpv-2.dll).
//! API reference: libmpv/client.h (mpv_create, mpv_set_option_string, mpv_initialize,
//! mpv_command, mpv_set_property_string, mpv_get_property_string, mpv_wait_event, ...).

use libloading::Library;
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

#[repr(C)]
struct MpvEvent {
    event_id: c_int,
    error: c_int,
    reply_userdata: u64,
    data: *mut c_void,
}

#[repr(C)]
struct ClientMessage {
    num_args: c_int,
    args: *const *const c_char,
}

const EV_NONE: c_int = 0;
const EV_SHUTDOWN: c_int = 1;
const EV_CLIENT_MESSAGE: c_int = 16;

type FnCreate = unsafe extern "C" fn() -> *mut c_void;
type FnInit = unsafe extern "C" fn(*mut c_void) -> c_int;
type FnSetOpt = unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> c_int;
type FnCmd = unsafe extern "C" fn(*mut c_void, *const *const c_char) -> c_int;
type FnWait = unsafe extern "C" fn(*mut c_void, f64) -> *mut MpvEvent;
type FnDestroy = unsafe extern "C" fn(*mut c_void);
type FnGetStr = unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_char;
type FnFree = unsafe extern "C" fn(*mut c_void);

pub struct Mpv {
    _lib: Library,
    h: *mut c_void,
    f_init: FnInit,
    f_set_opt: FnSetOpt,
    f_set_prop: FnSetOpt,
    f_cmd: FnCmd,
    f_wait: FnWait,
    f_destroy: FnDestroy,
    f_get_str: FnGetStr,
    f_free: FnFree,
    destroyed: Mutex<bool>,
}

// libmpv client API is documented as thread-safe (client.h).
unsafe impl Send for Mpv {}
unsafe impl Sync for Mpv {}

fn cs(s: &str) -> CString {
    CString::new(s.replace('\0', "")).unwrap()
}

impl Mpv {
    pub fn load(dll: &std::path::Path) -> Result<Mpv, String> {
        unsafe {
            let lib = Library::new(dll).map_err(|e| format!("libmpv-2.dll: {e}"))?;
            let create: FnCreate = *lib.get::<FnCreate>(b"mpv_create\0").map_err(|e| e.to_string())?;
            let f_init: FnInit = *lib.get::<FnInit>(b"mpv_initialize\0").map_err(|e| e.to_string())?;
            let f_set_opt: FnSetOpt = *lib.get::<FnSetOpt>(b"mpv_set_option_string\0").map_err(|e| e.to_string())?;
            let f_set_prop: FnSetOpt = *lib.get::<FnSetOpt>(b"mpv_set_property_string\0").map_err(|e| e.to_string())?;
            let f_cmd: FnCmd = *lib.get::<FnCmd>(b"mpv_command\0").map_err(|e| e.to_string())?;
            let f_wait: FnWait = *lib.get::<FnWait>(b"mpv_wait_event\0").map_err(|e| e.to_string())?;
            let f_destroy: FnDestroy = *lib.get::<FnDestroy>(b"mpv_terminate_destroy\0").map_err(|e| e.to_string())?;
            let f_get_str: FnGetStr = *lib.get::<FnGetStr>(b"mpv_get_property_string\0").map_err(|e| e.to_string())?;
            let f_free: FnFree = *lib.get::<FnFree>(b"mpv_free\0").map_err(|e| e.to_string())?;
            let h = create();
            if h.is_null() {
                return Err("mpv_create failed".into());
            }
            Ok(Mpv {
                _lib: lib,
                h,
                f_init,
                f_set_opt,
                f_set_prop,
                f_cmd,
                f_wait,
                f_destroy,
                f_get_str,
                f_free,
                destroyed: Mutex::new(false),
            })
        }
    }

    pub fn option(&self, name: &str, value: &str) {
        let (n, v) = (cs(name), cs(value));
        unsafe {
            (self.f_set_opt)(self.h, n.as_ptr(), v.as_ptr());
        }
    }

    pub fn init(&self) -> Result<(), String> {
        let r = unsafe { (self.f_init)(self.h) };
        if r < 0 {
            Err(format!("mpv_initialize error {r}"))
        } else {
            Ok(())
        }
    }

    pub fn set(&self, name: &str, value: &str) {
        if *self.destroyed.lock().unwrap() {
            return;
        }
        let (n, v) = (cs(name), cs(value));
        unsafe {
            (self.f_set_prop)(self.h, n.as_ptr(), v.as_ptr());
        }
    }

    pub fn get(&self, name: &str) -> Option<String> {
        if *self.destroyed.lock().unwrap() {
            return None;
        }
        let n = cs(name);
        unsafe {
            let p = (self.f_get_str)(self.h, n.as_ptr());
            if p.is_null() {
                return None;
            }
            let s = CStr::from_ptr(p).to_string_lossy().into_owned();
            (self.f_free)(p as *mut c_void);
            Some(s)
        }
    }

    pub fn command(&self, args: &[&str]) -> i32 {
        if *self.destroyed.lock().unwrap() {
            return -1;
        }
        let owned: Vec<CString> = args.iter().map(|a| cs(a)).collect();
        let mut ptrs: Vec<*const c_char> = owned.iter().map(|c| c.as_ptr()).collect();
        ptrs.push(std::ptr::null());
        unsafe { (self.f_cmd)(self.h, ptrs.as_ptr()) }
    }

    fn destroy(&self) {
        let mut d = self.destroyed.lock().unwrap();
        if !*d {
            *d = true;
            unsafe { (self.f_destroy)(self.h) };
        }
    }
}

/// (cache-secs, demuxer-max-bytes в MiB) для заданного значения кэша в мс.
/// 0 (Авто) -> исходные значения (6с / 16MiB). Больше кэша -> пропорционально больше буфер демультиплексора (от той же базовой пропорции 6с/16MiB), но не меньше 16MiB.
fn cache_values(cache_ms: i32) -> (f64, u64) {
    if cache_ms <= 0 {
        return (6.0, 16);
    }
    let secs = (cache_ms as f64 / 1000.0).clamp(0.5, 120.0);
    let mb = ((secs / 6.0) * 16.0).ceil().max(16.0) as u64;
    (secs, mb)
}

pub struct Player {
    pub mpv: Arc<Mpv>,
    running: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Player {
    /// `on_message` receives client messages sent with `script-message iptv <action> ...`
    pub fn start(
        dll: &std::path::Path,
        wid: isize,
        gpu: bool,
        cache_ms: i32,
        on_message: Box<dyn Fn(Vec<String>) + Send + 'static>,
    ) -> Result<Player, String> {
        let mpv = Mpv::load(dll)?;
        mpv.option("wid", &wid.to_string());
        mpv.option("config", "no");
        mpv.option("load-scripts", "no");
        mpv.option("ytdl", "no");
        mpv.option("terminal", "no");
        mpv.option("idle", "yes");
        mpv.option("force-window", "yes");
        mpv.option("input-default-bindings", "no");
        mpv.option("input-vo-keyboard", "yes");
        mpv.option("osc", "no");
    mpv.option("window-dragging", "no");
        mpv.option("osd-level", "1");
        mpv.option("osd-font-size", "34");
        mpv.option("osd-align-x", "left");
        mpv.option("osd-align-y", "top");
        mpv.option("osd-margin-x", "24");
        mpv.option("osd-margin-y", "20");
        mpv.option("sub-auto", "no");
        mpv.option("audio-display", "no");
        mpv.option("msg-level", "all=warn");
        mpv.option("network-timeout", "15");
        mpv.option("cache", "yes");
        let (cache_secs, demux_mb) = cache_values(cache_ms);
        mpv.option("demuxer-max-bytes", &format!("{demux_mb}MiB"));
        mpv.option("demuxer-max-back-bytes", "1MiB");
        mpv.option("cache-secs", &format!("{cache_secs}"));
        mpv.option("hwdec", if gpu { "auto-safe" } else { "no" });
        mpv.init()?;
        let mpv = Arc::new(mpv);
        let running = Arc::new(AtomicBool::new(true));
        let (m2, r2) = (mpv.clone(), running.clone());
        let thread = std::thread::spawn(move || {
            while r2.load(Ordering::Relaxed) {
                unsafe {
                    let ev = (m2.f_wait)(m2.h, 0.4);
                    if ev.is_null() {
                        continue;
                    }
                    match (*ev).event_id {
                        EV_NONE => {}
                        EV_SHUTDOWN => break,
                        EV_CLIENT_MESSAGE => {
                            let msg = (*ev).data as *const ClientMessage;
                            if !msg.is_null() {
                                let n = (*msg).num_args.max(0) as usize;
                                let mut v = Vec::with_capacity(n);
                                for i in 0..n {
                                    let p = *(*msg).args.add(i);
                                    if !p.is_null() {
                                        v.push(CStr::from_ptr(p).to_string_lossy().into_owned());
                                    }
                                }
                                on_message(v);
                            }
                        }
                        _ => {}
                    }
                }
            }
        });
        Ok(Player { mpv, running, thread: Some(thread) })
    }

    pub fn bind(&self, key: &str, action: &str) {
        self.mpv.command(&["keybind", key, &format!("script-message iptv {action}")]);
    }

    /// Изменить размер буфера без перезапуска mpv (применяется к текущему/следующему файлу). cache_ms=0 -> Авто.
    pub fn set_cache(&self, cache_ms: i32) {
        let (secs, mb) = cache_values(cache_ms);
        self.mpv.set("cache-secs", &format!("{secs}"));
        self.mpv.set("demuxer-max-bytes", &format!("{mb}MiB"));
    }

    /// (секунды буфера вперёд, идёт ли сейчас буферизация из-за нехватки данных). None если mpv ещё не отдаёт значение (нет активного потока).
    pub fn cache_stats(&self) -> Option<(f64, bool)> {
        let dur = self.mpv.get("demuxer-cache-duration")?.parse::<f64>().ok()?;
        let paused = self.mpv.get("paused-for-cache").map(|v| v == "yes").unwrap_or(false);
        Some((dur, paused))
    }

    pub fn unbind(&self, key: &str) {
        self.mpv.command(&["keybind", key, "ignore"]);
    }

    pub fn shutdown(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
        self.mpv.destroy();
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.shutdown();
    }
}


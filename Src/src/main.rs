#![windows_subsystem = "windows"]

mod epg;
mod i18n;
mod logos;
mod m3u;
mod mpv;
mod net;
mod settings;
mod winutil;

use settings::{mpv_key, norm_key, Settings, ACTIONS};
use slint::{CloseRequestResponse, ComponentHandle, Image, Model, ModelRc, PhysicalPosition, PhysicalSize, SharedString, Timer, TimerMode, VecModel};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

slint::include_modules!();

const VERSION: &str = env!("CARGO_PKG_VERSION");

struct State {
    s: Settings,
    channels: Vec<m3u::Channel>,
    url_tvg: Vec<String>,
    epg: Option<Arc<epg::Epg>>,
    epg_ids: Vec<Option<String>>,
    cur: Option<usize>,
    logos: HashMap<usize, Image>,
    audio_names: Vec<String>,
    audio_descs: Vec<String>,
}

struct App {
    win: MainWindow,
    tray: AppTray,
    settings_win: RefCell<Option<SettingsWindow>>,
    st: RefCell<State>,
    player: RefCell<Option<mpv::Player>>,
    rows: Rc<VecModel<ChannelRow>>,
    video: Cell<isize>,
    pending_logos: logos::Pending,
    logo_gen: Arc<AtomicU64>,
    timers: RefCell<Vec<Timer>>,
    update_timer: RefCell<Option<Timer>>,
    tray_hidden: Cell<bool>,
}

thread_local! {
    static APP: RefCell<Option<Rc<App>>> = const { RefCell::new(None) };
}

fn with_app(f: impl FnOnce(&Rc<App>)) {
    let app = APP.with(|a| a.borrow().clone());
    if let Some(app) = app {
        f(&app);
    }
}

pub fn dlog(msg: &str) {
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(settings::data_dir().join("log.txt")) {
        let _ = writeln!(f, "{} {}", epg::hhmm(epg::now_unix()), msg);
    }
}

fn tr(lang: i32, ru: &str, en: &str) -> String {
    i18n::tr(lang, ru, en)
}

const REPO_URL: &str = "https://github.com/ssergo71/Apps/tree/main/IPTVPlayer";
const RELEASES_URL: &str = "https://github.com/ssergo71/Apps/releases";

// ------------------------------------------------------------------ main

fn main() -> Result<(), slint::PlatformError> {
    // only one instance: a second copy exits silently (installer uses the same mutex name)
    let _single = match winutil::single_instance("IPTVPlayer_SingleInstance_MSChina888") {
        Some(h) => h,
        None => return Ok(()),
    };
    let s = Settings::load();
    let win = MainWindow::new()?;
    let tray = AppTray::new()?;
    let rows = Rc::new(VecModel::<ChannelRow>::default());
    win.set_rows(ModelRc::from(rows.clone()));

    let app = Rc::new(App {
        win,
        tray,
        settings_win: RefCell::new(None),
        st: RefCell::new(State {
            s,
            channels: vec![],
            url_tvg: vec![],
            epg: None,
            epg_ids: vec![],
            cur: None,
            logos: HashMap::new(),
            audio_names: vec![],
    audio_descs: vec![],
        }),
        player: RefCell::new(None),
        rows,
        video: Cell::new(0),
        pending_logos: Arc::new(Mutex::new(Vec::new())),
        logo_gen: Arc::new(AtomicU64::new(0)),
        timers: RefCell::new(vec![]),
        update_timer: RefCell::new(None),
        tray_hidden: Cell::new(false),
    });
    APP.with(|a| *a.borrow_mut() = Some(app.clone()));

    apply_settings_to_window(&app);
    wire_main(&app);

    app.win.show()?;
    app.tray.show()?;

    // restore window geometry
    {
        let st = app.st.borrow();
        if st.s.win_w > 200 && st.s.win_h > 150 {
            app.win.window().set_size(PhysicalSize::new(st.s.win_w, st.s.win_h));
        }
        if st.s.win_x > -10000 && st.s.win_y > -10000 && st.s.win_x != i32::MIN {
            app.win.window().set_position(PhysicalPosition::new(st.s.win_x, st.s.win_y));
        }
        if st.s.maximized {
            app.win.window().set_maximized(true);
        }
    }

    // the native window handle exists only after the event loop started
    Timer::single_shot(Duration::from_millis(250), || with_app(init_video));

    // periodic: refresh "now playing" texts, clock, pending logos
    {
        let t = Timer::default();
        t.start(TimerMode::Repeated, Duration::from_millis(900), || {
            with_app(|a| {
                drain_logos(a);
            })
        });
        app.timers.borrow_mut().push(t);

        let t2 = Timer::default();
        t2.start(TimerMode::Repeated, Duration::from_secs(30), || {
            with_app(|a| {
                a.win.set_clock(epg::hhmm(epg::now_unix()).into());
                rebuild_rows(a);
            })
        });
        app.timers.borrow_mut().push(t2);
    }
    app.win.set_clock(epg::hhmm(epg::now_unix()).into());
    schedule_updates(&app);

    slint::run_event_loop()
}

// ------------------------------------------------------------------ window <-> settings

fn apply_settings_to_window(app: &Rc<App>) {
    let st = app.st.borrow();
    let w = &app.win;
    w.set_lang(st.s.lang);
    w.set_theme(st.s.theme);
    w.set_list_visible(st.s.list_visible);
    w.set_list_left(st.s.list_left);
    w.set_font_size(st.s.font_size);
    w.set_show_clock(st.s.show_clock);
    w.set_volume(st.s.volume as f32);
    w.set_muted(st.s.muted);
    w.set_on_top(st.s.on_top);
    w.set_osd_on(st.s.osd_on);
    app.tray.set_label_show(tr(st.s.lang, "Показать", "Show").into());
    app.tray.set_label_quit(tr(st.s.lang, "Выход", "Exit").into());
}

fn init_video(app: &Rc<App>) {
    let Some(parent) = winutil::hwnd_of(app.win.window()) else {
        // not ready yet, retry shortly
        Timer::single_shot(Duration::from_millis(250), || with_app(init_video));
        return;
    };
    let (x, y, w, h) = video_rect_phys(app);
    let child = winutil::create_video_child(parent, x, y, w, h);
    app.video.set(child);
    winutil::VIDEO_HWND.store(child, Ordering::Relaxed);
    winutil::install_mouse_hook(|kind, delta, ctrl| {
        let _ = slint::invoke_from_event_loop(move || {
            with_app(|a| match kind {
                0 => do_action(a, "click"),
                1 => do_action(a, "menu"),
                2 => do_action(a, "mute"),
                _ => {
                    let up = delta > 0;
                    let act = match (ctrl, up) {
                        (true, true) => "prev",
                        (true, false) => "next",
                        (false, true) => "vol_up",
                        (false, false) => "vol_down",
                    };
                    do_action(a, act);
                }
            });
        });
    });

    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())).unwrap_or_default();
    let dll = exe_dir.join("libmpv-2.dll");
    let gpu = app.st.borrow().s.gpu;
    let res = mpv::Player::start(
        &dll,
        child,
        gpu,
        Box::new(|msg: Vec<String>| {
            let _ = slint::invoke_from_event_loop(move || with_app(|a| on_mpv_message(a, msg)));
        }),
    );
    match res {
        Ok(p) => {
            dlog("mpv started");
            *app.player.borrow_mut() = Some(p);
            rebind_hotkeys(app, &HashMap::new());
            bind_mouse(app);
            apply_audio_video(app);
            // audio device list
            if let Some(p) = app.player.borrow().as_ref() {
                let mut names = vec![String::new()];
                let mut descs = vec![String::new()];
                if let Some(json) = p.mpv.get("audio-device-list") {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json) {
                        if let Some(arr) = v.as_array() {
                            for d in arr {
                                if let Some(n) = d.get("name").and_then(|n| n.as_str()) {
                                    if n == "auto" { continue; }
                                    let desc = d.get("description").and_then(|x| x.as_str()).unwrap_or(n);
                                    names.push(n.to_string());
                                    descs.push(desc.to_string());
                                }
                            }
                        }
                    }
                }
                dlog(&format!("audio devices: {} found; selected=[{}]", names.len() - 1, app.st.borrow().s.audio_device));
                {
                    let mut st = app.st.borrow_mut();
                    st.audio_names = names;
                    st.audio_descs = descs;
                }
            }
        }
        Err(e) => {
            dlog(&format!("mpv error: {e}"));
            app.win.set_notice(i18n::fmt(tr(app.st.borrow().s.lang, "Движок воспроизведения не загружен: {}", "Playback engine not loaded: {}"), &[&e.to_string()]).into());
        }
    }
    reload(app, true);
}

fn video_rect_phys(app: &Rc<App>) -> (i32, i32, i32, i32) {
    let sc = app.win.window().scale_factor();
    let w = &app.win;
    (
        (w.get_vx() * sc).round() as i32,
        (w.get_vy() * sc).round() as i32,
        (w.get_vw() * sc).round() as i32,
        (w.get_vh() * sc).round() as i32,
    )
}

fn rebind_hotkeys(app: &Rc<App>, old: &HashMap<String, String>) {
    let st = app.st.borrow();
    if let Some(p) = app.player.borrow().as_ref() {
        for k in old.values() {
            if !k.is_empty() {
                p.unbind(&mpv_key(k));
            }
        }
        for (action, _, _, _) in ACTIONS {
            let k = st.s.hotkey(action);
            if !k.is_empty() {
                p.bind(&mpv_key(&k), action);
            }
        }
    }
}

fn bind_mouse(app: &Rc<App>) {
    if let Some(p) = app.player.borrow().as_ref() {
        // mouse over the video is handled by the low-level hook in winutil (see init_video)
        for k in ["MBTN_LEFT", "MBTN_RIGHT", "MBTN_MID", "WHEEL_UP", "WHEEL_DOWN", "Ctrl+WHEEL_UP", "Ctrl+WHEEL_DOWN"] {
            p.unbind(k);
        }
    }
}

fn apply_audio_video(app: &Rc<App>) {
    let st = app.st.borrow();
    let s = &st.s;
    if let Some(p) = app.player.borrow().as_ref() {
        let m = &p.mpv;
        m.set("volume", &s.volume.to_string());
        m.set("mute", if s.muted { "yes" } else { "no" });
        m.set("brightness", &s.brightness.to_string());
        m.set("contrast", &s.contrast.to_string());
        m.set("saturation", &s.saturation.to_string());
        m.set("deinterlace", if s.deint { "yes" } else { "no" });
        m.set("hwdec", if s.gpu { "auto-safe" } else { "no" });
        let aspect = match s.aspect {
            1 => "4:3",
            2 => "16:9",
            3 => "16:10",
            _ => "-1",
        };
        m.set("video-aspect-override", aspect);
        m.set("audio-channels", if s.mono { "mono" } else { "auto-safe" });
        m.set("alang", &s.audio_lang);
        if !s.audio_device.is_empty() {
            m.set("audio-device", &s.audio_device);
        } else {
            m.set("audio-device", "auto");
        }
    }
}

// ------------------------------------------------------------------ playlist / epg

fn reload(app: &Rc<App>, with_epg: bool) {
    let (file, url) = {
        let st = app.st.borrow();
        (st.s.playlist_file.clone(), st.s.playlist_url.clone())
    };
    if file.is_empty() && url.is_empty() {
        app.win.set_notice(
            tr(app.st.borrow().s.lang, "Откройте Настройки → Источники и укажите список каналов (M3U/M3U8)", "Open Settings → Sources and set the channel list (M3U/M3U8)").into(),
        );
        return;
    }
    app.win.set_notice(tr(app.st.borrow().s.lang, "Загрузка списка каналов...", "Loading channel list...").into());
    std::thread::spawn(move || {
        let text = if !file.is_empty() {
            std::fs::read(&file).map(|b| String::from_utf8_lossy(&b).into_owned()).map_err(|e| format!("{file}: {e}"))
        } else {
            net::text(&url)
        };
        match text {
            Ok(t) => {
                let pl = m3u::parse(&t);
                let _ = slint::invoke_from_event_loop(move || with_app(|a| on_playlist(a, pl, with_epg)));
            }
            Err(e) => {
                let _ = slint::invoke_from_event_loop(move || with_app(|a| a.win.set_notice(i18n::fmt(tr(a.st.borrow().s.lang, "Ошибка списка: {}", "List error: {}"), &[&e.to_string()]).into())));
            }
        }
    });
}

fn on_playlist(app: &Rc<App>, pl: m3u::Playlist, with_epg: bool) {
    {
        let mut st = app.st.borrow_mut();
        st.channels = pl.channels;
        st.url_tvg = pl.url_tvg;
        st.logos.clear();
        st.epg_ids = vec![None; st.channels.len()];
        // keep current channel pointer if still present
        let last = st.s.last_channel.clone();
        st.cur = st.channels.iter().position(|c| c.key() == last);
    }
    app.win.set_notice("".into());
    dlog(&format!("playlist: {} channels, url-tvg {:?}", app.st.borrow().channels.len(), app.st.borrow().url_tvg));
    start_logos(app);
    rebuild_rows(app);
    refresh_cur_info(app);
    if let Some(idx) = app.st.borrow().cur {
        if let Some(pos) = app.rows.iter().position(|r| r.kind == 1 && r.idx == idx as i32) {
            app.win.set_scroll_to_row(-1);
            app.win.set_scroll_to_row(pos as i32);
        }
    }
    if with_epg {
        load_epg(app);
    }
    // debug/self-test: IPTV_SNAPSHOT=1 saves PNG snapshots of the Slint windows
    if std::env::var("IPTV_SNAPSHOT").is_ok() {
        Timer::single_shot(Duration::from_secs(8), || {
            with_app(|a| {
                save_snapshot(a.win.window(), "snapshot_main.png");
                open_settings(a);
                Timer::single_shot(Duration::from_millis(1500), || {
                    with_app(|a| {
                        if let Some(sw) = a.settings_win.borrow().as_ref() {
                            save_snapshot(sw.window(), "snapshot_settings.png");
                        }
                    })
                });
            })
        });
    }
    // debug/self-test: IPTV_AUTOPLAY=<channel index>
    if let Some(i) = std::env::var("IPTV_AUTOPLAY").ok().and_then(|n| n.parse::<usize>().ok()) {
        play(app, i);
        Timer::single_shot(Duration::from_secs(14), || {
            with_app(|a| {
                if let Some(p) = a.player.borrow().as_ref() {
                    let g = |k: &str| p.mpv.get(k).unwrap_or_default();
                    dlog(&format!(
                        "autoplay: time-pos={} codec={} size={}x{} hwdec={} audio={} paused={} ao={} device={} vol={} mute={} atime={}",
                        g("time-pos"), g("video-codec"), g("width"), g("height"), g("hwdec-current"), g("audio-codec-name"), g("pause"), g("current-ao"), g("audio-device"), g("volume"), g("mute"), g("audio-pts")
                    ));
                }
            })
        });
    }
}

fn save_snapshot(w: &slint::Window, name: &str) {
    match w.take_snapshot() {
        Ok(buf) => {
            if let Some(img) = image::RgbaImage::from_raw(buf.width(), buf.height(), buf.as_bytes().to_vec()) {
                let _ = img.save(settings::data_dir().join(name));
                dlog(&format!("snapshot saved: {name} {}x{}", buf.width(), buf.height()));
            }
        }
        Err(e) => dlog(&format!("snapshot error: {e}")),
    }
}

fn load_epg(app: &Rc<App>) {
    let (urls, wanted) = {
        let st = app.st.borrow();
        let urls: Vec<String> = if !st.s.epg_url.trim().is_empty() {
            st.s.epg_url.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()
        } else {
            st.url_tvg.clone()
        };
        let mut ids = HashSet::new();
        let mut names = HashSet::new();
        for c in &st.channels {
            if !c.tvg_id.is_empty() {
                ids.insert(c.tvg_id.clone());
            }
            if !c.tvg_name.is_empty() {
                names.insert(epg::norm(&c.tvg_name));
            }
            names.insert(epg::norm(&c.name));
        }
        (urls, epg::Wanted { ids, names })
    };
    if urls.is_empty() {
        return;
    }
    std::thread::spawn(move || {
        let mut merged = epg::Epg::default();
        let mut err = String::new();
        for u in urls {
            match epg::load(&u, &wanted) {
                Ok(e) => {
                    merged.progs.extend(e.progs);
                    for (k, v) in e.names {
                        merged.names.entry(k).or_insert(v);
                    }
                    merged.ids.extend(e.ids);
                    for (k, v) in e.icons {
                        merged.icons.entry(k).or_insert(v);
                    }
                }
                Err(e) => err = e,
            }
        }
        let merged = Arc::new(merged);
        let _ = slint::invoke_from_event_loop(move || {
            with_app(|a| {
                if merged.progs.is_empty() && !err.is_empty() {
                    a.win.set_notice(i18n::fmt(tr(a.st.borrow().s.lang, "Ошибка телепрограммы: {}", "Guide error: {}"), &[&err.to_string()]).into());
                    return;
                }
                {
                    let mut st = a.st.borrow_mut();
                    let ids: Vec<Option<String>> = st.channels.iter().map(|c| merged.resolve(&c.tvg_id, &c.tvg_name, &c.name)).collect();
                    dlog(&format!("epg: {} channels with programmes, matched {} of {}", merged.progs.len(), ids.iter().filter(|x| x.is_some()).count(), ids.len()));
                    st.epg_ids = ids;
                    st.epg = Some(merged.clone());
                }
                start_logos(a);
                rebuild_rows(a);
            })
        });
    });
}

fn schedule_updates(app: &Rc<App>) {
    let hours = app.st.borrow().s.update_hours;
    let mut slot = app.update_timer.borrow_mut();
    *slot = None;
    if hours > 0 {
        let t = Timer::default();
        t.start(TimerMode::Repeated, Duration::from_secs(hours as u64 * 3600), || with_app(|a| reload(a, true)));
        *slot = Some(t);
    }
}

/// logo source: tvg-logo from the playlist, otherwise the icon from the EPG
fn start_logos(app: &Rc<App>) {
    let gen = app.logo_gen.fetch_add(1, Ordering::Relaxed) + 1;
    let items: Vec<(usize, String)> = {
        let st = app.st.borrow();
        st.channels
            .iter()
            .enumerate()
            .filter_map(|(i, c)| {
                if c.logo.starts_with("http") {
                    return Some((i, c.logo.clone()));
                }
                let id = st.epg_ids.get(i)?.as_ref()?;
                let url = st.epg.as_ref()?.icons.get(id)?;
                if url.starts_with("http") { Some((i, url.clone())) } else { None }
            })
            .collect()
    };
    logos::spawn(items, app.pending_logos.clone(), app.logo_gen.clone(), gen);
}

fn drain_logos(app: &Rc<App>) {
    let items: Vec<_> = std::mem::take(&mut *app.pending_logos.lock().unwrap());
    if items.is_empty() {
        return;
    }
    {
        let mut st = app.st.borrow_mut();
        for (i, buf) in items {
            st.logos.insert(i, Image::from_rgba8(buf));
        }
    }
    rebuild_rows(app);
}

// ------------------------------------------------------------------ rows

fn epg_now_title(st: &State, idx: usize, now: i64) -> Option<(String, i64, i64)> {
    let epg = st.epg.as_ref()?;
    let id = st.epg_ids.get(idx)?.as_ref()?;
    epg.now(id, now).map(|p| (p.title.clone(), p.start, p.stop))
}

fn rebuild_rows(app: &Rc<App>) {
    let st = app.st.borrow();
    let hidden: HashSet<String> = st.s.hidden.iter().cloned().collect();
    let coll: HashSet<String> = st.s.collapsed.iter().cloned().collect();
    let now = epg::now_unix();
    let mut rows: Vec<ChannelRow> = Vec::with_capacity(st.channels.len() + 32);
    let mut last: Option<&str> = None;
    let mut skip = false;
    for (i, c) in st.channels.iter().enumerate() {
        let key = c.key();
        if hidden.contains(&key) {
            continue;
        }
        if last != Some(c.group.as_str()) {
            last = Some(c.group.as_str());
            skip = false;
            if !c.group.is_empty() {
                let collapsed = coll.contains(&c.group);
                skip = collapsed;
                rows.push(ChannelRow {
                    kind: 0,
                    idx: -1,
                    num: 0,
                    name: c.group.clone().into(),
                    now: SharedString::new(),
                    logo: Image::default(),
                    has_logo: false,
                    playing: false,
                    collapsed,
                });
            }
        }
        if skip {
            continue;
        }
        let logo = st.logos.get(&i);
        rows.push(ChannelRow {
            kind: 1,
            idx: i as i32,
            num: (i + 1) as i32,
            name: c.name.clone().into(),
            now: epg_now_title(&st, i, now).map(|t| t.0).unwrap_or_default().into(),
            logo: logo.cloned().unwrap_or_default(),
            has_logo: logo.is_some(),
            playing: st.cur == Some(i),
            collapsed: false,
        });
    }
    drop(st);
    app.rows.set_vec(rows);
}

fn refresh_cur_info(app: &Rc<App>) {
    let st = app.st.borrow();
    if let Some(i) = st.cur {
        if let Some(c) = st.channels.get(i) {
            let mut line = format!("{}. {}", i + 1, c.name);
            if let Some((title, _, stop)) = epg_now_title(&st, i, epg::now_unix()) {
                line.push_str(&format!(" — {} ({} {})", title, tr(st.s.lang, "до", "until"), epg::hhmm(stop)));
            }
            app.win.set_cur_name(line.into());
        }
    }
}

// ------------------------------------------------------------------ playback

fn effective_url(s: &Settings, url: &str) -> String {
    if let Some(rest) = url.strip_prefix("udp://") {
        let rest = rest.trim_start_matches('@');
        if !s.udp_proxy.trim().is_empty() {
            return format!("http://{}/udp/{}", s.udp_proxy.trim(), rest);
        }
        if !s.local_ip.trim().is_empty() {
            let sep = if url.contains('?') { '&' } else { '?' };
            return format!("{}{}localaddr={}", url, sep, s.local_ip.trim());
        }
    }
    url.to_string()
}

fn play(app: &Rc<App>, idx: usize) {
    let (url, name_line) = {
        let mut st = app.st.borrow_mut();
        let Some(c) = st.channels.get(idx).cloned() else { return };
        st.cur = Some(idx);
        st.s.last_channel = c.key();
        (effective_url(&st.s, &c.url), format!("{}. {}", idx + 1, c.name))
    };
    if let Some(p) = app.player.borrow().as_ref() {
        p.mpv.set("pause", "no");
        p.mpv.command(&["loadfile", &url, "replace"]);
    }
    app.win.set_cur_name(name_line.into());
    app.win.set_playing(true);
    refresh_cur_info(app);
    if app.st.borrow().s.osd_on {
        show_osd(app);
    }
    rebuild_rows(app);
    if let Some(pos) = app.rows.iter().position(|r| r.kind == 1 && r.idx == idx as i32) {
        app.win.set_scroll_to_row(-1);
        app.win.set_scroll_to_row(pos as i32);
    }
}

fn show_osd(app: &Rc<App>) {
    let st = app.st.borrow();
    let Some(i) = st.cur else { return };
    let Some(c) = st.channels.get(i) else { return };
    let mut text = format!("{}. {}", i + 1, c.name);
    if let Some((title, a, b)) = epg_now_title(&st, i, epg::now_unix()) {
        text.push_str(&format!("\n{}  {}–{}", title, epg::hhmm(a), epg::hhmm(b)));
    }
    if let Some(p) = app.player.borrow().as_ref() {
        p.mpv.command(&["show-text", &text, "5000"]);
    }
}

fn visible_channels(st: &State) -> Vec<usize> {
    st.channels
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            let k = c.key();
            !st.s.hidden.contains(&k)
        })
        .map(|(i, _)| i)
        .collect()
}

fn step_channel(app: &Rc<App>, dir: i32) {
    let target = {
        let st = app.st.borrow();
        let vis = visible_channels(&st);
        if vis.is_empty() {
            return;
        }
        let pos = st.cur.and_then(|c| vis.iter().position(|&v| v == c));
        let n = vis.len() as i32;
        let next = match pos {
            Some(p) => ((p as i32 + dir).rem_euclid(n)) as usize,
            None => if dir > 0 { 0 } else { vis.len() - 1 },
        };
        vis[next]
    };
    play(app, target);
}

fn set_volume(app: &Rc<App>, v: i32) {
    let v = v.clamp(0, 100);
    app.st.borrow_mut().s.volume = v;
    app.win.set_volume(v as f32);
    if let Some(p) = app.player.borrow().as_ref() {
        p.mpv.set("volume", &v.to_string());
    }
}


// ------------------------------------------------------------------ actions

fn on_mpv_message(app: &Rc<App>, msg: Vec<String>) {
    if msg.len() >= 2 && msg[0] == "iptv" {
        do_action(app, &msg[1]);
    }
}

fn do_action(app: &Rc<App>, action: &str) {
    match action {
        "click" => {
            // double click on the video toggles full screen
            thread_local! { static LAST_CLICK: Cell<Option<std::time::Instant>> = Cell::new(None); }
            let now = std::time::Instant::now();
            let dbl = LAST_CLICK.with(|c| c.get()).map(|t| now.duration_since(t) < Duration::from_millis(450)).unwrap_or(false);
            if dbl {
                LAST_CLICK.with(|c| c.set(None));
                app.win.set_fullscreen(!app.win.get_fullscreen());
            } else {
                LAST_CLICK.with(|c| c.set(Some(now)));
            }
        }
        "fullscreen" => app.win.set_fullscreen(!app.win.get_fullscreen()),
        "esc" => {
            if app.win.get_fullscreen() {
                app.win.set_fullscreen(false);
            }
        }
        "other_monitor" => {
            if let Some(h) = winutil::hwnd_of(app.win.window()) {
                if let Some((x, y)) = winutil::next_monitor_origin(h) {
                    let was_fs = app.win.get_fullscreen();
                    if was_fs {
                        app.win.set_fullscreen(false);
                    }
                    app.win.window().set_position(PhysicalPosition::new(x, y));
                    if was_fs {
                        app.win.set_fullscreen(true);
                    }
                }
            }
        }
        "on_top" => {
            let v = !app.win.get_on_top();
            app.win.set_on_top(v);
            app.st.borrow_mut().s.on_top = v;
        }
        "list" => {
            let v = !app.win.get_list_visible();
            app.win.set_list_visible(v);
            app.st.borrow_mut().s.list_visible = v;
            if v {
                if let Some(idx) = app.st.borrow().cur {
                    if let Some(pos) = app.rows.iter().position(|r| r.kind == 1 && r.idx == idx as i32) {
                        app.win.set_scroll_to_row(-1);
                        app.win.set_scroll_to_row(pos as i32);
                    }
                }
            }
        }
        "settings" => open_settings(app),
        "tray" => hide_to_tray(app),
        "quit" => quit_app(app),
        "pause" => toggle_pause(app),
        "stop" => {
            if let Some(p) = app.player.borrow().as_ref() {
                p.mpv.command(&["stop"]);
            }
            app.win.set_playing(false);
        }
        "mute" => {
            let v = !app.win.get_muted();
            app.win.set_muted(v);
            app.st.borrow_mut().s.muted = v;
            if let Some(p) = app.player.borrow().as_ref() {
                p.mpv.set("mute", if v { "yes" } else { "no" });
            }
        }
        "vol_up" => {
            let v = app.st.borrow().s.volume + 5;
            set_volume(app, v)
        }
        "vol_down" => {
            let v = app.st.borrow().s.volume - 5;
            set_volume(app, v)
        }
        "next" => step_channel(app, 1),
        "prev" => step_channel(app, -1),
        "osd" => {
            let v = !app.win.get_osd_on();
            app.win.set_osd_on(v);
            app.st.borrow_mut().s.osd_on = v;
            if v {
                show_osd(app);
            } else if let Some(p) = app.player.borrow().as_ref() {
                p.mpv.command(&["show-text", "", "1"]);
            }
        }
        "menu" => show_menu(app),
        _ => {}
    }
}

fn toggle_pause(app: &Rc<App>) {
    if let Some(p) = app.player.borrow().as_ref() {
        let paused = p.mpv.get("pause").as_deref() == Some("yes");
        let idle = p.mpv.get("idle-active").as_deref() == Some("yes");
        if !idle {
            p.mpv.set("pause", if paused { "no" } else { "yes" });
            app.win.set_playing(paused);
            return;
        }
    }
    let cur = app.st.borrow().cur;
    if let Some(i) = cur {
        play(app, i);
    }
}

fn hide_to_tray(app: &Rc<App>) {
    if let Some(h) = winutil::hwnd_of(app.win.window()) {
        winutil::show_window(h, false);
        app.tray_hidden.set(true);
    }
}

fn restore_from_tray(app: &Rc<App>) {
    if let Some(h) = winutil::hwnd_of(app.win.window()) {
        winutil::show_window(h, true);
        app.tray_hidden.set(false);
    }
}

fn quit_app(app: &Rc<App>) {
    {
        let mut st = app.st.borrow_mut();
        let w = app.win.window();
        st.s.maximized = w.is_maximized();
        if !w.is_maximized() && !w.is_fullscreen() && !w.is_minimized() {
            let p = w.position();
            let sz = w.size();
            st.s.win_x = p.x;
            st.s.win_y = p.y;
            st.s.win_w = sz.width;
            st.s.win_h = sz.height;
        }
        st.s.volume = app.win.get_volume() as i32;
        st.s.save();
    }
    if let Some(mut p) = app.player.borrow_mut().take() {
        p.shutdown();
    }
    let _ = slint::quit_event_loop();
}

fn show_menu(app: &Rc<App>) {
    let Some(owner) = winutil::hwnd_of(app.win.window()) else { return };
    let st = app.st.borrow();
    let l = st.s.lang;
    let hk = |a: &str| {
        let k = st.s.hotkey(a);
        if k.is_empty() { String::new() } else { format!("\t{}", k) }
    };
    let mk = |id: usize, label: String, checked: bool| winutil::MenuItemSpec { id, label, checked, separator: false };
    let sep = || winutil::MenuItemSpec { id: 0, label: String::new(), checked: false, separator: true };
    let items = vec![
        mk(1, format!("{}{}", tr(l, "На полный экран", "Full screen"), hk("fullscreen")), app.win.get_fullscreen()),
        mk(2, format!("{}{}", tr(l, "На другой монитор", "Move to other monitor"), hk("other_monitor")), false),
        mk(3, format!("{}{}", tr(l, "Поверх всех окон", "Always on top"), hk("on_top")), app.win.get_on_top()),
        sep(),
        mk(4, format!("{}{}", tr(l, "Список каналов", "Channel list"), hk("list")), app.win.get_list_visible()),
        mk(5, format!("{}{}", tr(l, "Телепрограмма поверх видео", "Programme overlay"), hk("osd")), app.win.get_osd_on()),
        sep(),
        mk(7, format!("{}{}", tr(l, "Настройки", "Settings"), hk("settings")), false),
        sep(),
        mk(8, format!("{}{}", tr(l, "Свернуть в трей", "Minimize to tray"), hk("tray")), false),
        mk(9, format!("{}{}", tr(l, "Выход", "Exit"), hk("quit")), false),
    ];
    drop(st);
    let id = winutil::popup_menu(owner, &items);
    let action = match id {
        1 => "fullscreen",
        2 => "other_monitor",
        3 => "on_top",
        4 => "list",
        5 => "osd",
        7 => "settings",
        8 => "tray",
        9 => "quit",
        _ => return,
    };
    do_action(app, action);
}

// ------------------------------------------------------------------ wiring of main window

fn wire_main(app: &Rc<App>) {
    let w = &app.win;

    let a = app.clone();
    w.window().on_close_requested(move || {
        quit_app(&a);
        CloseRequestResponse::HideWindow
    });

    let a = app.clone();
    w.on_video_geometry(move |x, y, ww, hh| {
        let child = a.video.get();
        if child != 0 {
            let sc = a.win.window().scale_factor();
            winutil::move_child(child, (x * sc).round() as i32, (y * sc).round() as i32, (ww * sc).round() as i32, (hh * sc).round() as i32);
        }
    });

    let a = app.clone();
    w.on_channel_clicked(move |i| play(&a, i as usize));

    let a = app.clone();
    w.on_group_toggled(move |name| {
        {
            let mut st = a.st.borrow_mut();
            let n = name.to_string();
            if let Some(p) = st.s.collapsed.iter().position(|g| *g == n) {
                st.s.collapsed.remove(p);
            } else {
                st.s.collapsed.push(n);
            }
        }
        rebuild_rows(&a);
    });

    let a = app.clone();
    w.on_play_pause(move || toggle_pause(&a));
    let a = app.clone();
    w.on_stop_clicked(move || do_action(&a, "stop"));
    let a = app.clone();
    w.on_prev_clicked(move || step_channel(&a, -1));
    let a = app.clone();
    w.on_next_clicked(move || step_channel(&a, 1));
    let a = app.clone();
    w.on_volume_changed(move |v| set_volume(&a, v.round() as i32));
    let a = app.clone();
    w.on_mute_clicked(move || do_action(&a, "mute"));
    let a = app.clone();
    w.on_list_clicked(move || do_action(&a, "list"));
    let a = app.clone();
    w.on_osd_clicked(move || do_action(&a, "osd"));
    let a = app.clone();
    w.on_settings_clicked(move || do_action(&a, "settings"));
    let a = app.clone();
    w.on_fullscreen_clicked(move || do_action(&a, "fullscreen"));
    let a = app.clone();
    w.on_menu_requested(move || show_menu(&a));

    let a = app.clone();
    w.on_key_event(move |name, ctrl, alt, shift| {
        let mut n = name.to_string();
        if n.chars().count() == 1 {
            let c = n.chars().next().unwrap() as u32;
            if c > 0 && c < 32 {
                n = ((c + 64) as u8 as char).to_string(); // Ctrl+P arrives as control code
            }
        }
        let mut combo = String::new();
        if ctrl { combo.push_str("Ctrl+"); }
        if alt { combo.push_str("Alt+"); }
        if shift { combo.push_str("Shift+"); }
        combo.push_str(&n);
        let want = norm_key(&combo);
        let action = {
            let st = a.st.borrow();
            ACTIONS.iter().find(|(act, _, _, _)| norm_key(&st.s.hotkey(act)) == want).map(|(act, _, _, _)| act.to_string())
        };
        if let Some(act) = action {
            do_action(&a, &act);
        }
    });

    let a = app.clone();
    app.tray.on_show_window(move || restore_from_tray(&a));
    let a = app.clone();
    app.tray.on_quit(move || quit_app(&a));
}

// ------------------------------------------------------------------ settings window

fn cfg_from(s: &Settings) -> Cfg {
    Cfg {
        playlist_url: s.playlist_url.clone().into(),
        playlist_file: s.playlist_file.clone().into(),
        epg_url: s.epg_url.clone().into(),
        update_on_start: s.update_on_start,
        update_hours: s.update_hours,
        theme: s.theme,
        lang: s.lang,
        list_left: s.list_left,
        font_size: s.font_size,
        show_clock: s.show_clock,
        gpu: s.gpu,
        audio_lang: s.audio_lang.clone().into(),
        mono: s.mono,
        deint: s.deint,
        aspect: s.aspect,
        brightness: s.brightness,
        contrast: s.contrast,
        saturation: s.saturation,
        local_ip: s.local_ip.clone().into(),
        udp_proxy: s.udp_proxy.clone().into(),
        autostart: s.autostart,
    }
}

fn hk_rows(s: &Settings) -> Vec<HotkeyRow> {
    ACTIONS
        .iter()
        .map(|(a, _, ru, en)| HotkeyRow { label: tr(s.lang, ru, en).into(), action: (*a).into(), key: s.hotkey(a).into() })
        .collect()
}

fn ch_rows(st: &State) -> Vec<ChToggle> {
    st.channels
        .iter()
        .map(|c| ChToggle { name: c.name.clone().into(), shown: !st.s.hidden.contains(&c.key()), group: c.group.clone().into() })
        .collect()
}

fn open_settings(app: &Rc<App>) {
    if let Some(sw) = app.settings_win.borrow().as_ref() {
        let _ = sw.show();
        kick_settings_repaint();
        return;
    }
    let sw = match SettingsWindow::new() {
        Ok(s) => s,
        Err(_) => return,
    };
    {
        let st = app.st.borrow();
        sw.set_cfg(cfg_from(&st.s));
        sw.set_lang(st.s.lang);
        sw.set_theme(st.s.theme);
        sw.set_chs(ModelRc::from(Rc::new(VecModel::from(ch_rows(&st)))));
        sw.set_hks(ModelRc::from(Rc::new(VecModel::from(hk_rows(&st.s)))));
        let devs: Vec<SharedString> = st
            .audio_names
            .iter()
            .enumerate()
            .map(|(i, n)| if n.is_empty() { tr(st.s.lang, "Авто (по умолчанию)", "Auto (default)").into() } else { SharedString::from(st.audio_descs.get(i).map(|d| d.as_str()).unwrap_or(n.as_str())) })
            .collect();
        let idx = st.audio_names.iter().position(|n| *n == st.s.audio_device).unwrap_or(0);
        sw.set_audio_devices(ModelRc::from(Rc::new(VecModel::from(devs))));
        sw.set_audio_dev_index(idx as i32);
        let engine = app.player.borrow().as_ref().and_then(|p| p.mpv.get("mpv-version")).unwrap_or_default();
        let l = st.s.lang;
        sw.set_info(format!("{}: {}\n{}: MSChina888", tr(l, "Версия", "Version"), VERSION, tr(l, "Разработчик", "Developer")).into());
        sw.set_repo_label(tr(l, "Репозиторий (приложение)", "Repository (app)").into());
        sw.set_repo_url(REPO_URL.into());
        sw.set_rel_label(tr(l, "Релизы (обновления)", "Releases (updates)").into());
        sw.set_rel_url(RELEASES_URL.into());
        sw.set_info2(format!("\n{}: Slint (https://slint.dev)\n{}: libmpv (LGPL), {}", tr(l, "Интерфейс", "Interface"), tr(l, "Движок воспроизведения", "Playback engine"), engine).into());
        sw.on_open_link(|u| winutil::open_url(u.as_str()));
        sw.global::<Tr>().on_t(|lang, ru, en| tr(lang, ru.as_str(), en.as_str()).into());
    }

    {
        let a = app.clone();
        sw.on_browse_playlist(move || {
            let l = a.st.borrow().s.lang;
            let pl = tr(l, "Списки", "Playlists");
            let all = tr(l, "Все файлы", "All files");
            winutil::pick_file("M3U / M3U8", &[(&pl, "*.m3u;*.m3u8"), (&all, "*.*")]).unwrap_or_default().into()
        });
        let a = app.clone();
        sw.on_browse_epg(move || {
            let l = a.st.borrow().s.lang;
            let all = tr(l, "Все файлы", "All files");
            winutil::pick_file("XMLTV", &[("XMLTV (*.xml;*.xml.gz;*.gz)", "*.xml;*.xml.gz;*.gz"), (&all, "*.*")]).unwrap_or_default().into()
        });
    }
    let a = app.clone();
    sw.on_apply(move |cfg, status| {
        apply_cfg(&a, cfg, status.to_string());
        // close the settings window after Apply
        if let Some(s) = a.settings_win.borrow().as_ref() {
            let _ = s.hide();
        }
    });
    let a = app.clone();
    sw.on_close_clicked(move || {
        if let Some(s) = a.settings_win.borrow().as_ref() {
            let _ = s.hide();
        }
    });
    let a = app.clone();
    sw.on_ch_toggled(move |i, shown| {
        {
            let mut st = a.st.borrow_mut();
            if let Some(c) = st.channels.get(i as usize) {
                let key = c.key();
                st.s.hidden.retain(|k| *k != key);
                if !shown {
                    st.s.hidden.push(key);
                }
                st.s.save();
            }
        }
        rebuild_rows(&a);
    });
    let a = app.clone();
    sw.on_hotkey_edited(move |i, key| {
        if let Some((act, _, _, _)) = ACTIONS.get(i as usize) {
            a.st.borrow_mut().s.hotkeys.insert((*act).to_string(), key.to_string());
        }
    });
    let a = app.clone();
    sw.on_hotkeys_reset(move || {
        {
            let mut st = a.st.borrow_mut();
            for (act, k, _, _) in ACTIONS {
                st.s.hotkeys.insert((*act).to_string(), (*k).to_string());
            }
        }
        if let Some(sw) = a.settings_win.borrow().as_ref() {
            sw.set_hks(ModelRc::from(Rc::new(VecModel::from(hk_rows(&a.st.borrow().s)))));
        }
    });
    let a = app.clone();
    sw.on_check_update(move || check_update(&a));

    let _ = sw.show();
    *app.settings_win.borrow_mut() = Some(sw);
    kick_settings_repaint();
}

/// The settings window sometimes keeps a stale/partially painted buffer when it opens over the playing video:
/// force a full repaint (resize by one pixel and back, redraw requests).
fn kick_settings_repaint() {
    fn redraw() {
        with_app(|a| {
            if let Some(sw) = a.settings_win.borrow().as_ref() {
                sw.window().request_redraw();
            }
        });
    }
    Timer::single_shot(Duration::from_millis(60), redraw);
    Timer::single_shot(Duration::from_millis(150), || {
        with_app(|a| {
            if let Some(sw) = a.settings_win.borrow().as_ref() {
                let s = sw.window().size();
                sw.window().set_size(PhysicalSize::new(s.width + 1, s.height));
            }
        });
    });
    Timer::single_shot(Duration::from_millis(300), || {
        with_app(|a| {
            if let Some(sw) = a.settings_win.borrow().as_ref() {
                let s = sw.window().size();
                sw.window().set_size(PhysicalSize::new(s.width.saturating_sub(1), s.height));
            }
        });
    });
    Timer::single_shot(Duration::from_millis(600), redraw);
}

fn apply_cfg(app: &Rc<App>, cfg: Cfg, status: String) {
    dlog(&format!("apply: file=[{}] url=[{}] epg=[{}]", cfg.playlist_file, cfg.playlist_url, cfg.epg_url));
    let old_hotkeys;
    let source_changed;
    {
        let mut st = app.st.borrow_mut();
        let s = &mut st.s;
        let new_source = format!("{}|{}|{}|{}", cfg.playlist_url, cfg.playlist_file, cfg.udp_proxy, cfg.local_ip);
        source_changed = new_source != format!("{}|{}|{}|{}", s.playlist_url, s.playlist_file, s.udp_proxy, s.local_ip) || cfg.epg_url != s.epg_url.as_str();
        s.playlist_url = cfg.playlist_url.to_string();
        s.playlist_file = cfg.playlist_file.to_string();
        s.epg_url = cfg.epg_url.to_string();
        s.update_on_start = cfg.update_on_start;
        s.update_hours = cfg.update_hours;
        s.theme = cfg.theme;
        s.lang = cfg.lang;
        s.list_left = cfg.list_left;
        s.font_size = cfg.font_size;
        s.show_clock = cfg.show_clock;
        s.gpu = cfg.gpu;
        s.audio_lang = cfg.audio_lang.to_string();
        s.mono = cfg.mono;
        s.deint = cfg.deint;
        s.aspect = cfg.aspect;
        s.brightness = cfg.brightness;
        s.contrast = cfg.contrast;
        s.saturation = cfg.saturation;
        s.local_ip = cfg.local_ip.to_string();
        s.udp_proxy = cfg.udp_proxy.to_string();
        let autostart_changed = s.autostart != cfg.autostart;
        s.autostart = cfg.autostart;
        if let Some(n) = status.strip_prefix("dev:").and_then(|n| n.parse::<usize>().ok()) {
            if let Some(name) = st.audio_names.get(n).cloned() {
                st.s.audio_device = name;
            }
        }
        if autostart_changed {
            winutil::set_autostart(st.s.autostart);
        }
        // hotkeys were stored live while editing; keep the previous bindings to unbind them
        old_hotkeys = HashMap::new();
        st.s.save();
    }
    apply_settings_to_window(app);
    rebind_hotkeys(app, &old_hotkeys);
    bind_mouse(app);
    apply_audio_video(app);
    schedule_updates(app);
    rebuild_rows(app);
    if let Some(sw) = app.settings_win.borrow().as_ref() {
        sw.set_lang(app.st.borrow().s.lang);
        sw.set_theme(app.st.borrow().s.theme);
    }
    if source_changed {
        reload(app, true);
    }
}

fn check_update(app: &Rc<App>) {
    let lang = app.st.borrow().s.lang;
    if let Some(sw) = app.settings_win.borrow().as_ref() {
        sw.set_status(tr(lang, "Проверка...", "Checking...").into());
    }
    std::thread::spawn(move || {
        let res = net::text("https://api.github.com/repos/ssergo71/Apps/releases/latest");
        let msg = match res {
            Ok(t) => match serde_json::from_str::<serde_json::Value>(&t) {
                Ok(v) => {
                    let tag = v.get("tag_name").and_then(|x| x.as_str()).unwrap_or("").trim_start_matches('v').to_string();
                    if tag.is_empty() {
                        tr(lang, "Релизов пока нет", "No releases yet")
                    } else if tag == VERSION {
                        i18n::fmt(tr(lang, "Установлена последняя версия ({})", "You have the latest version ({})"), &[VERSION])
                    } else {
                        i18n::fmt(tr(lang, "Доступна версия {}: {}", "Version {} is available: {}"), &[&tag, "https://github.com/ssergo71/Apps/releases/latest"])
                    }
                }
                Err(_) => tr(lang, "Не удалось разобрать ответ GitHub", "Could not parse GitHub reply"),
            },
            Err(_) => tr(lang, "Релизов пока нет или GitHub недоступен", "No releases yet or GitHub is unreachable"),
        };
        let _ = slint::invoke_from_event_loop(move || {
            with_app(|a| {
                if let Some(sw) = a.settings_win.borrow().as_ref() {
                    sw.set_status(msg.into());
                }
            })
        });
    });
}





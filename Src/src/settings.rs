use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub const ACTIONS: &[(&str, &str, &str, &str)] = &[
    // action, default key, label RU, label EN
    ("fullscreen", "F", "Полный экран", "Full screen"),
    ("other_monitor", "Ctrl+F", "На другой монитор", "Move to other monitor"),
    ("on_top", "Ctrl+T", "Поверх всех окон", "Always on top"),
    ("list", "L", "Список каналов", "Channel list"),
    ("settings", "Ctrl+P", "Настройки", "Settings"),
    ("tray", "Ctrl+M", "Свернуть в трей", "Minimize to tray"),
    ("quit", "Alt+X", "Выход", "Exit"),
    ("pause", "Space", "Пауза / пуск", "Pause / play"),
    ("mute", "M", "Выключить звук", "Mute"),
    ("vol_up", "Up", "Громче", "Volume up"),
    ("vol_down", "Down", "Тише", "Volume down"),
    ("next", "PgDn", "Следующий канал", "Next channel"),
    ("prev", "PgUp", "Предыдущий канал", "Previous channel"),
    ("osd", "I", "Показать/скрыть передачу поверх видео", "Show/hide programme overlay"),
    ("esc", "Esc", "Выйти из полного экрана", "Leave full screen"),
];

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Settings {
    pub playlist_url: String,
    pub playlist_file: String,
    pub epg_url: String,
    pub update_on_start: bool,
    pub update_hours: i32,
    pub theme: i32,
    pub lang: i32,
    pub list_left: bool,
    pub list_visible: bool,
    pub font_size: i32,
    pub show_clock: bool,
    pub gpu: bool,
    pub audio_lang: String,
    pub mono: bool,
    pub deint: bool,
    pub aspect: i32,
    pub brightness: i32,
    pub contrast: i32,
    pub saturation: i32,
    pub audio_device: String,
    /// Буфер/кэш mpv, мс. 0 = Авто (встроенные значения mpv.rs; возможно переопределены cache= из плейлиста).
    pub cache_ms: i32,
    pub local_ip: String,
    pub udp_proxy: String,
    pub autostart: bool,
    pub volume: i32,
    pub muted: bool,
    pub on_top: bool,
    pub osd_on: bool,
    pub win_x: i32,
    pub win_y: i32,
    pub win_w: u32,
    pub win_h: u32,
    pub maximized: bool,
    pub last_channel: String,
    pub hidden: Vec<String>,
    pub collapsed: Vec<String>,
    pub hotkeys: BTreeMap<String, String>,
}

impl Default for Settings {
    fn default() -> Self {
        let mut hotkeys = BTreeMap::new();
        for (a, k, _, _) in ACTIONS {
            hotkeys.insert(a.to_string(), k.to_string());
        }
        Settings {
            playlist_url: String::new(),
            playlist_file: String::new(),
            epg_url: String::new(),
            update_on_start: true,
            update_hours: 12,
            theme: 0,
            lang: 0,
            list_left: false,
            list_visible: true,
            font_size: 13,
            show_clock: false,
            gpu: true,
            audio_lang: "ru".into(),
            mono: false,
            deint: false,
            aspect: 0,
            brightness: 0,
            contrast: 0,
            saturation: 0,
            audio_device: String::new(),
            cache_ms: 0,
            local_ip: String::new(),
            udp_proxy: String::new(),
            autostart: false,
            volume: 80,
            muted: false,
            on_top: false,
            osd_on: true,
            win_x: i32::MIN,
            win_y: i32::MIN,
            win_w: 0,
            win_h: 0,
            maximized: false,
            last_channel: String::new(),
            hidden: vec![],
            collapsed: vec![],
            hotkeys,
        }
    }
}

pub fn data_dir() -> PathBuf {
    let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
    let p = PathBuf::from(base).join("IPTVPlayer");
    let _ = std::fs::create_dir_all(&p);
    p
}

pub fn cache_dir() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into());
    let p = PathBuf::from(base).join("IPTVPlayer").join("logos");
    let _ = std::fs::create_dir_all(&p);
    p
}

fn path() -> PathBuf {
    data_dir().join("settings.json")
}

impl Settings {
    pub fn load() -> Settings {
        let mut s: Settings = std::fs::read_to_string(path())
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        // add missing hotkeys (new versions)
        for (a, k, _, _) in ACTIONS {
            s.hotkeys.entry(a.to_string()).or_insert_with(|| k.to_string());
        }
        s
    }

    pub fn save(&self) {
        if let Ok(t) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path(), t);
        }
    }

    pub fn hotkey(&self, action: &str) -> String {
        self.hotkeys.get(action).cloned().unwrap_or_default()
    }
}

/// Normalise "ctrl + p" -> "Ctrl+P"
pub fn norm_key(s: &str) -> String {
    let mut ctrl = false;
    let mut alt = false;
    let mut shift = false;
    let mut key = String::new();
    for part in s.split('+') {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        match p.to_lowercase().as_str() {
            "ctrl" | "control" => ctrl = true,
            "alt" => alt = true,
            "shift" => shift = true,
            _ => key = p.to_string(),
        }
    }
    if key.chars().count() == 1 {
        key = key.to_uppercase();
    } else {
        key = match key.to_lowercase().as_str() {
            "space" => "Space".into(),
            "up" => "Up".into(),
            "down" => "Down".into(),
            "left" => "Left".into(),
            "right" => "Right".into(),
            "pgup" | "pageup" => "PgUp".into(),
            "pgdn" | "pagedown" | "pgdown" => "PgDn".into(),
            "esc" | "escape" => "Esc".into(),
            "enter" | "return" => "Enter".into(),
            "tab" => "Tab".into(),
            "home" => "Home".into(),
            "end" => "End".into(),
            "delete" | "del" => "Delete".into(),
            "backspace" => "Backspace".into(),
            k if k.starts_with('f') && k[1..].parse::<u32>().is_ok() => k.to_uppercase(),
            _ => key,
        };
    }
    let mut out = String::new();
    if ctrl {
        out.push_str("Ctrl+");
    }
    if alt {
        out.push_str("Alt+");
    }
    if shift && key.chars().count() > 1 {
        out.push_str("Shift+");
    }
    out.push_str(&key);
    out
}

/// "Ctrl+P" -> mpv key name "Ctrl+p"
pub fn mpv_key(s: &str) -> String {
    let n = norm_key(s);
    let (mods, key) = match n.rfind('+') {
        Some(i) if i + 1 < n.len() => (n[..=i].to_string(), n[i + 1..].to_string()),
        _ => (String::new(), n.clone()),
    };
    let k = match key.as_str() {
        "Space" => "SPACE".to_string(),
        "Up" => "UP".into(),
        "Down" => "DOWN".into(),
        "Left" => "LEFT".into(),
        "Right" => "RIGHT".into(),
        "PgUp" => "PGUP".into(),
        "PgDn" => "PGDWN".into(),
        "Esc" => "ESC".into(),
        "Enter" => "ENTER".into(),
        "Tab" => "TAB".into(),
        "Home" => "HOME".into(),
        "End" => "END".into(),
        "Delete" => "DEL".into(),
        "Backspace" => "BS".into(),
        k if k.chars().count() == 1 => k.to_lowercase(),
        k => k.to_string(),
    };
    format!("{}{}", mods, k)
}

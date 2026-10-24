<p align="center"><img src="icon.png" width="96" alt="IPTV Player"></p>

# IPTV Player for Windows 11 x64

**Languages:** [Русский](README.md) · [English](README.en.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [中文](README.zh.md)

A lightweight 64-bit IPTV player for Windows 11 written in Rust. Minimal memory use and minimal interaction with the system. Developer: **MSChina888**.

## Features
- M3U/M3U8 channel lists (URL or local file, picked via "Browse…"); order and groups come from the list.
- XMLTV guide (.xml, .xml.gz), matched by `tvg-id`, then `tvg-name`; logos from `tvg-logo` and the EPG.
- UDP multicast (`udp://@…`), UDP→HTTP proxy, network interface selection.
- Favourites, collapsible groups, hide channels, current programme in the channel row and over the video (OSD).
- Full screen, always on top, move between monitors, tray icon, autostart, scheduled list refresh.
- Brightness/contrast/saturation, deinterlace, aspect ratio, audio device and audio language, GPU acceleration.
- Themes: light, dark, like Windows. Configurable hotkeys.
- Interface languages: Russian, English, German, Spanish, French, Italian, Chinese.

No recording, scheduler or screenshots — viewing only.

## Installation
Download `IPTVPlayer-Setup-<version>.exe` from [Releases](https://github.com/ssergo71/Apps/releases) and run it. Settings are stored in `%APPDATA%\IPTVPlayer`.

## Building from source
Rust (MSVC), `cargo build --release` in `Src`; `libmpv-2.dll` (LGPL mpv build) must sit next to `iptvplayer.exe`. Installer: Inno Setup, `Installer\installer.iss`.

## Technology
UI: [Slint](https://slint.dev). Playback: libmpv (LGPL).

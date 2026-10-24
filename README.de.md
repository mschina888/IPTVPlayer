<p align="center"><img src="icon.png" width="96" alt="IPTV Player"></p>

# IPTV Player für Windows 11 x64

**Sprachen:** [Русский](README.md) · [English](README.en.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [中文](README.zh.md)

Ein schlanker 64-Bit-IPTV-Player für Windows 11, geschrieben in Rust. Minimaler Speicherverbrauch und minimale Interaktion mit dem System. Entwickler: **MSChina888**.

## Funktionen
- M3U/M3U8-Senderlisten (URL oder lokale Datei, Auswahl über „Durchsuchen…“); Reihenfolge und Gruppen stammen aus der Liste.
- XMLTV-Programmführer (.xml, .xml.gz), Zuordnung über `tvg-id`, dann `tvg-name`; Logos aus `tvg-logo` und dem EPG.
- UDP-Multicast (`udp://@…`), UDP→HTTP-Proxy, Auswahl der Netzwerkschnittstelle.
- Favoriten, einklappbare Gruppen, Sender ausblenden, aktuelle Sendung in der Senderzeile und über dem Video (OSD).
- Vollbild, immer im Vordergrund, Wechsel zwischen Monitoren, Tray-Symbol, Autostart, geplante Listenaktualisierung.
- Helligkeit/Kontrast/Sättigung, Deinterlacing, Seitenverhältnis, Audiogerät und Tonspursprache, GPU-Beschleunigung.
- Designs: hell, dunkel, wie Windows. Konfigurierbare Tastenkürzel.
- Oberflächensprachen: Russisch, Englisch, Deutsch, Spanisch, Französisch, Italienisch, Chinesisch.

Keine Aufnahme, kein Planer, keine Screenshots — nur Wiedergabe.

## Installation
`IPTVPlayer-Setup-<Version>.exe` unter [Releases](https://github.com/ssergo71/Apps/releases) herunterladen und ausführen. Einstellungen liegen in `%APPDATA%\IPTVPlayer`.

## Aus dem Quellcode bauen
Rust (MSVC), `cargo build --release` in `Src`; `libmpv-2.dll` (LGPL-mpv-Build) muss neben `iptvplayer.exe` liegen. Installer: Inno Setup, `Installer\installer.iss`.

## Technik
Oberfläche: [Slint](https://slint.dev). Wiedergabe: libmpv (LGPL).

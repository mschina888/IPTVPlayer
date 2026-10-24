<p align="center"><img src="icon.png" width="96" alt="IPTV Player"></p>

# IPTV Player per Windows 11 x64

**Lingue:** [Русский](README.md) · [English](README.en.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [中文](README.zh.md)

Lettore IPTV leggero a 64 bit per Windows 11, scritto in Rust. Consumo di memoria minimo e interazione minima con il sistema. Sviluppatore: **MSChina888**.

## Funzionalità
- Elenchi di canali M3U/M3U8 (URL o file locale, scelto con «Sfoglia…»); ordine e gruppi provengono dall'elenco.
- Guida XMLTV (.xml, .xml.gz), abbinamento tramite `tvg-id` e poi `tvg-name`; loghi da `tvg-logo` e dalla guida.
- Multicast UDP (`udp://@…`), proxy UDP→HTTP, scelta dell'interfaccia di rete.
- Preferiti, gruppi comprimibili, canali nascondibili, programma in corso nella riga del canale e sopra il video (OSD).
- Schermo intero, sempre in primo piano, passaggio tra monitor, icona nell'area di notifica, avvio con Windows, aggiornamento pianificato degli elenchi.
- Luminosità/contrasto/saturazione, deinterlacciamento, rapporto d'aspetto, dispositivo audio e lingua della traccia, accelerazione GPU.
- Temi: chiaro, scuro, come Windows. Scorciatoie da tastiera configurabili.
- Lingue dell'interfaccia: russo, inglese, tedesco, spagnolo, francese, italiano, cinese.

Nessuna registrazione, pianificatore o schermata: solo visione.

## Installazione
Scarica `IPTVPlayer-Setup-<versione>.exe` da [Releases](https://github.com/ssergo71/Apps/releases) ed eseguilo. Le impostazioni sono in `%APPDATA%\IPTVPlayer`.

## Compilare dai sorgenti
Rust (MSVC), `cargo build --release` in `Src`; `libmpv-2.dll` (build LGPL di mpv) deve trovarsi accanto a `iptvplayer.exe`. Installer: Inno Setup, `Installer\installer.iss`.

## Tecnologie
Interfaccia: [Slint](https://slint.dev). Riproduzione: libmpv (LGPL).

<p align="center"><img src="icon.png" width="96" alt="IPTV Player"></p>

# IPTV Player pour Windows 11 x64

**Langues :** [Русский](README.md) · [English](README.en.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [中文](README.zh.md)

Lecteur IPTV léger 64 bits pour Windows 11, écrit en Rust. Consommation mémoire minimale et interaction minimale avec le système. Développeur : **MSChina888**.

## Fonctionnalités
- Listes de chaînes M3U/M3U8 (URL ou fichier local, choisi via « Parcourir… ») ; l'ordre et les groupes viennent de la liste.
- Guide XMLTV (.xml, .xml.gz), association par `tvg-id` puis `tvg-name` ; logos issus de `tvg-logo` et du guide.
- Multicast UDP (`udp://@…`), proxy UDP→HTTP, choix de l'interface réseau.
- Favoris, groupes repliables, masquage de chaînes, programme en cours dans la ligne de la chaîne et sur la vidéo (OSD).
- Plein écran, toujours au premier plan, passage d'un écran à l'autre, icône dans la zone de notification, démarrage avec Windows, actualisation planifiée des listes.
- Luminosité/contraste/saturation, désentrelacement, format d'image, périphérique audio et langue de la piste, accélération GPU.
- Thèmes : clair, sombre, comme Windows. Raccourcis clavier configurables.
- Langues de l'interface : russe, anglais, allemand, espagnol, français, italien, chinois.

Pas d'enregistrement, de planificateur ni de captures d'écran : lecture uniquement.

## Installation
Téléchargez `IPTVPlayer-Setup-<version>.exe` depuis [Releases](https://github.com/ssergo71/Apps/releases) et lancez-le. Les paramètres sont stockés dans `%APPDATA%\IPTVPlayer`.

## Compiler depuis les sources
Rust (MSVC), `cargo build --release` dans `Src` ; `libmpv-2.dll` (build LGPL de mpv) doit se trouver à côté de `iptvplayer.exe`. Installateur : Inno Setup, `Installer\installer.iss`.

## Technologies
Interface : [Slint](https://slint.dev). Lecture : libmpv (LGPL).

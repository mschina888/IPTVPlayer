<p align="center"><img src="icon.png" width="96" alt="IPTV Player"></p>

# IPTV Player para Windows 11 x64

**Idiomas:** [Русский](README.md) · [English](README.en.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [中文](README.zh.md)

Reproductor IPTV ligero de 64 bits para Windows 11, escrito en Rust. Consumo mínimo de memoria y mínima interacción con el sistema. Desarrollador: **MSChina888**.

## Características
- Listas de canales M3U/M3U8 (URL o archivo local, elegido con «Examinar…»); el orden y los grupos vienen de la lista.
- Guía XMLTV (.xml, .xml.gz), asociación por `tvg-id` y después por `tvg-name`; logotipos de `tvg-logo` y de la guía.
- Multicast UDP (`udp://@…`), proxy UDP→HTTP, selección de interfaz de red.
- Favoritos, grupos plegables, ocultar canales, programa actual en la fila del canal y sobre el vídeo (OSD).
- Pantalla completa, siempre visible, cambio entre monitores, icono en la bandeja, inicio con Windows, actualización programada de listas.
- Brillo/contraste/saturación, desentrelazado, relación de aspecto, dispositivo de audio e idioma de la pista, aceleración por GPU.
- Temas: claro, oscuro, como Windows. Atajos de teclado configurables.
- Idiomas de la interfaz: ruso, inglés, alemán, español, francés, italiano, chino.

Sin grabación, programador ni capturas de pantalla: solo reproducción.

## Instalación
Descargue `IPTVPlayer-Setup-<versión>.exe` desde [Releases](https://github.com/ssergo71/Apps/releases) y ejecútelo. La configuración se guarda en `%APPDATA%\IPTVPlayer`.

## Compilar desde el código fuente
Rust (MSVC), `cargo build --release` en `Src`; `libmpv-2.dll` (compilación LGPL de mpv) debe estar junto a `iptvplayer.exe`. Instalador: Inno Setup, `Installer\installer.iss`.

## Tecnología
Interfaz: [Slint](https://slint.dev). Reproducción: libmpv (LGPL).

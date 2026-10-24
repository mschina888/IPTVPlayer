<p align="center"><img src="icon.png" width="96" alt="IPTV Player"></p>

# IPTV Player for Windows 11 x64

**语言:** [Русский](README.md) · [English](README.en.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [中文](README.zh.md)

使用 Rust 编写的轻量级 64 位 Windows 11 IPTV 播放器。内存占用极低,与系统的交互极少。开发者:**MSChina888**。

## 功能
- M3U/M3U8 频道列表(URL 或本地文件,可通过“浏览...”选择);顺序和分组取自列表。
- XMLTV 节目指南(.xml、.xml.gz),先按 `tvg-id`,再按 `tvg-name` 匹配;频道图标来自 `tvg-logo` 和节目指南。
- UDP 组播(`udp://@…`)、UDP→HTTP 代理、网络接口选择。
- 收藏、可折叠分组、隐藏频道、在频道行和视频上显示当前节目(OSD)。
- 全屏、置顶、在显示器之间移动、托盘图标、开机自启、定时刷新列表。
- 亮度/对比度/饱和度、去隔行、宽高比、音频设备和音轨语言、GPU 加速。
- 主题:浅色、深色、跟随 Windows。可自定义快捷键。
- 界面语言:俄语、英语、德语、西班牙语、法语、意大利语、中文。

不含录制、计划任务和截图,仅用于观看。

## 安装
从 [Releases](https://github.com/ssergo71/Apps/releases) 下载 `IPTVPlayer-Setup-<版本>.exe` 并运行。设置保存在 `%APPDATA%\IPTVPlayer`。

## 从源码构建
Rust (MSVC),在 `Src` 中运行 `cargo build --release`;`libmpv-2.dll`(LGPL 版 mpv)需放在 `iptvplayer.exe` 旁。安装程序:Inno Setup,`Installer\installer.iss`。

## 技术
界面:[Slint](https://slint.dev)。播放:libmpv (LGPL)。

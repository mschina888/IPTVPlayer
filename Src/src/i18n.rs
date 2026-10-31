// Translations. Language index: 0 ru, 1 en, 2 de, 3 es, 4 fr, 5 it, 6 zh.
// Key = English text; value = [de, es, fr, it, zh].

pub fn tr(lang: i32, ru: &str, en: &str) -> String {
    match lang {
        0 => ru.to_string(),
        1 => en.to_string(),
        n => {
            let i = (n - 2) as usize;
            for (k, v) in TABLE {
                if *k == en {
                    return v.get(i).copied().unwrap_or(en).to_string();
                }
            }
            en.to_string()
        }
    }
}

static TABLE: &[(&str, [&str; 5])] = &[
    ("Settings", ["Einstellungen", "Configuración", "Paramètres", "Impostazioni", "设置"]),
    ("Sources", ["Quellen", "Fuentes", "Sources", "Sorgenti", "来源"]),
    ("Appearance", ["Darstellung", "Apariencia", "Apparence", "Aspetto", "外观"]),
    ("Video & audio", ["Video & Audio", "Vídeo y audio", "Vidéo et audio", "Video e audio", "视频和音频"]),
    ("Channels", ["Sender", "Canales", "Chaînes", "Canali", "频道"]),
    ("Hotkeys", ["Tastenkürzel", "Atajos de teclado", "Raccourcis clavier", "Scorciatoie", "快捷键"]),
    ("System", ["System", "Sistema", "Système", "Sistema", "系统"]),
    ("About", ["Über das Programm", "Acerca de", "À propos", "Informazioni", "关于"]),
    ("Channel list (M3U/M3U8)", ["Senderliste (M3U/M3U8)", "Lista de canales (M3U/M3U8)", "Liste de chaînes (M3U/M3U8)", "Elenco canali (M3U/M3U8)", "频道列表 (M3U/M3U8)"]),
    ("List address (URL)", ["Listenadresse (URL)", "Dirección de la lista (URL)", "Adresse de la liste (URL)", "Indirizzo dell'elenco (URL)", "列表地址 (URL)"]),
    ("Or a local M3U/M3U8 file (used if set)", ["Oder lokale M3U/M3U8-Datei (wird bevorzugt, wenn angegeben)", "O un archivo local M3U/M3U8 (se usa si se indica)", "Ou un fichier local M3U/M3U8 (utilisé s'il est indiqué)", "Oppure un file locale M3U/M3U8 (usato se indicato)", "或本地 M3U/M3U8 文件(如已指定则使用)"]),
    ("XMLTV guide (URL, .xml or .xml.gz). Empty — url-tvg from the list", ["XMLTV-Programmführer (URL, .xml oder .xml.gz). Leer — url-tvg aus der Liste", "Guía XMLTV (URL, .xml o .xml.gz). Vacío: url-tvg de la lista", "Guide XMLTV (URL, .xml ou .xml.gz). Vide : url-tvg de la liste", "Guida XMLTV (URL, .xml o .xml.gz). Vuoto: url-tvg dell'elenco", "XMLTV 节目指南 (URL、.xml 或 .xml.gz)。留空则使用列表中的 url-tvg"]),
    ("Browse...", ["Durchsuchen...", "Examinar...", "Parcourir...", "Sfoglia...", "浏览..."]),
    ("Refresh list and guide on start", ["Liste und Programm beim Start aktualisieren", "Actualizar lista y guía al iniciar", "Actualiser la liste et le guide au démarrage", "Aggiorna elenco e guida all'avvio", "启动时刷新列表和节目指南"]),
    ("Auto-refresh every (hours, 0 = off)", ["Automatisch aktualisieren alle (Stunden, 0 = aus)", "Actualizar cada (horas, 0 = desactivado)", "Actualiser toutes les (heures, 0 = désactivé)", "Aggiorna ogni (ore, 0 = disattivato)", "每隔(小时)自动刷新,0 = 关闭"]),
    ("UDP→HTTP proxy (host:port, for udp://@... addresses)", ["UDP→HTTP-Proxy (Host:Port, für udp://@... Adressen)", "Proxy UDP→HTTP (host:puerto, para direcciones udp://@...)", "Proxy UDP→HTTP (hôte:port, pour les adresses udp://@...)", "Proxy UDP→HTTP (host:porta, per indirizzi udp://@...)", "UDP→HTTP 代理(主机:端口,用于 udp://@... 地址)"]),
    ("Network interface for UDP multicast (local IP)", ["Netzwerkschnittstelle für UDP-Multicast (lokale IP)", "Interfaz de red para multicast UDP (IP local)", "Interface réseau pour le multicast UDP (IP locale)", "Interfaccia di rete per multicast UDP (IP locale)", "UDP 组播网络接口(本地 IP)"]),
    ("Theme", ["Design", "Tema", "Thème", "Tema", "主题"]),
    ("Like Windows", ["Wie Windows", "Como Windows", "Comme Windows", "Come Windows", "跟随 Windows"]),
    ("Light", ["Hell", "Claro", "Clair", "Chiaro", "浅色"]),
    ("Dark", ["Dunkel", "Oscuro", "Sombre", "Scuro", "深色"]),
    ("Interface language", ["Sprache der Oberfläche", "Idioma de la interfaz", "Langue de l'interface", "Lingua dell'interfaccia", "界面语言"]),
    ("List font size", ["Schriftgröße der Liste", "Tamaño de fuente de la lista", "Taille de police de la liste", "Dimensione carattere dell'elenco", "列表字体大小"]),
    ("Channel list on the left", ["Senderliste links", "Lista de canales a la izquierda", "Liste de chaînes à gauche", "Elenco canali a sinistra", "频道列表在左侧"]),
    ("Show clock", ["Uhr anzeigen", "Mostrar reloj", "Afficher l'horloge", "Mostra orologio", "显示时钟"]),
    ("GPU hardware acceleration (if available)", ["GPU-Hardwarebeschleunigung (falls verfügbar)", "Aceleración por GPU (si está disponible)", "Accélération matérielle GPU (si disponible)", "Accelerazione hardware GPU (se disponibile)", "GPU 硬件加速(如可用)"]),
    ("Deinterlace", ["Deinterlacing", "Desentrelazado", "Désentrelacement", "Deinterlacciamento", "去隔行"]),
    ("Mono", ["Mono", "Mono", "Mono", "Mono", "单声道"]),
    ("Aspect ratio", ["Seitenverhältnis", "Relación de aspecto", "Format d'image", "Rapporto d'aspetto", "宽高比"]),
    ("Auto", ["Auto", "Automático", "Auto", "Auto", "自动"]),
    ("Audio track language (ru, en...)", ["Sprache der Tonspur (ru, en...)", "Idioma de la pista de audio (ru, en...)", "Langue de la piste audio (ru, en...)", "Lingua della traccia audio (ru, en...)", "音轨语言(ru、en...)"]),
    ("Brightness", ["Helligkeit", "Brillo", "Luminosité", "Luminosità", "亮度"]),
    ("Contrast", ["Kontrast", "Contraste", "Contraste", "Contrasto", "对比度"]),
    ("Saturation", ["Sättigung", "Saturación", "Saturation", "Saturazione", "饱和度"]),
    ("Audio device", ["Audiogerät", "Dispositivo de audio", "Périphérique audio", "Dispositivo audio", "音频设备"]),
    ("Show channels", ["Sender anzeigen", "Mostrar canales", "Afficher les chaînes", "Mostra canali", "显示频道"]),
    ("Hotkeys (e.g. F, Ctrl+P, Alt+X, Space, Up, PgDn, F6)", ["Tastenkürzel (z. B. F, Ctrl+P, Alt+X, Space, Up, PgDn, F6)", "Atajos (p. ej. F, Ctrl+P, Alt+X, Space, Up, PgDn, F6)", "Raccourcis (p. ex. F, Ctrl+P, Alt+X, Space, Up, PgDn, F6)", "Scorciatoie (es. F, Ctrl+P, Alt+X, Space, Up, PgDn, F6)", "快捷键(例如 F、Ctrl+P、Alt+X、Space、Up、PgDn、F6)"]),
    ("Reset hotkeys", ["Tastenkürzel zurücksetzen", "Restablecer atajos", "Réinitialiser les raccourcis", "Ripristina scorciatoie", "重置快捷键"]),
    ("Start with Windows", ["Mit Windows starten", "Iniciar con Windows", "Démarrer avec Windows", "Avvia con Windows", "随 Windows 启动"]),
    ("Check for program updates", ["Nach Programmupdates suchen", "Buscar actualizaciones del programa", "Rechercher les mises à jour", "Verifica aggiornamenti del programma", "检查程序更新"]),
    ("Apply", ["Übernehmen", "Aplicar", "Appliquer", "Applica", "应用"]),
    ("Close", ["Schließen", "Cerrar", "Fermer", "Chiudi", "关闭"]),
    ("Version", ["Version", "Versión", "Version", "Versione", "版本"]),
    ("Developer", ["Entwickler", "Desarrollador", "Développeur", "Sviluppatore", "开发者"]),
    ("Repository (updates)", ["Repository (Updates)", "Repositorio (actualizaciones)", "Dépôt (mises à jour)", "Repository (aggiornamenti)", "仓库(更新)"]),
    ("Interface", ["Oberfläche", "Interfaz", "Interface", "Interfaccia", "界面"]),
    ("Engine", ["Engine", "Motor", "Moteur", "Motore", "引擎"]),
    ("Playback engine", ["Wiedergabe-Engine", "Motor de reproducción", "Moteur de lecture", "Motore di riproduzione", "播放引擎"]),
    ("Show", ["Anzeigen", "Mostrar", "Afficher", "Mostra", "显示"]),
    ("Exit", ["Beenden", "Salir", "Quitter", "Esci", "退出"]),
    ("Open Settings → Sources and set the channel list (M3U/M3U8)", ["Öffnen Sie Einstellungen → Quellen und geben Sie die Senderliste an (M3U/M3U8)", "Abra Configuración → Fuentes e indique la lista de canales (M3U/M3U8)", "Ouvrez Paramètres → Sources et indiquez la liste de chaînes (M3U/M3U8)", "Apri Impostazioni → Sorgenti e indica l'elenco canali (M3U/M3U8)", "请打开 设置 → 来源 并指定频道列表 (M3U/M3U8)"]),
    ("Loading channel list...", ["Senderliste wird geladen...", "Cargando lista de canales...", "Chargement de la liste de chaînes...", "Caricamento elenco canali...", "正在加载频道列表..."]),
    ("Playback engine not loaded: {}", ["Wiedergabe-Engine nicht geladen: {}", "Motor de reproducción no cargado: {}", "Moteur de lecture non chargé : {}", "Motore di riproduzione non caricato: {}", "播放引擎未加载:{}"]),
    ("List error: {}", ["Listenfehler: {}", "Error de lista: {}", "Erreur de liste : {}", "Errore elenco: {}", "列表错误:{}"]),
    ("Guide error: {}", ["Programmfehler: {}", "Error de guía: {}", "Erreur de guide : {}", "Errore guida: {}", "节目指南错误:{}"]),
    ("Full screen", ["Vollbild", "Pantalla completa", "Plein écran", "Schermo intero", "全屏"]),
    ("Move to other monitor", ["Auf anderen Monitor", "Mover a otro monitor", "Déplacer vers un autre écran", "Sposta su altro monitor", "移到其他显示器"]),
    ("Always on top", ["Immer im Vordergrund", "Siempre visible", "Toujours au premier plan", "Sempre in primo piano", "置顶"]),
    ("Channel list", ["Senderliste", "Lista de canales", "Liste de chaînes", "Elenco canali", "频道列表"]),
    ("Programme overlay", ["Programm über dem Video", "Programa sobre el vídeo", "Programme sur la vidéo", "Programma sul video", "视频上显示节目"]),
    ("Minimize to tray", ["In den Infobereich minimieren", "Minimizar a la bandeja", "Réduire dans la zone de notification", "Riduci nell'area di notifica", "最小化到托盘"]),
    ("Pause / play", ["Pause / Wiedergabe", "Pausa / reproducir", "Pause / lecture", "Pausa / riproduci", "暂停 / 播放"]),
    ("Mute", ["Stummschalten", "Silenciar", "Couper le son", "Disattiva audio", "静音"]),
    ("Volume up", ["Lauter", "Subir volumen", "Augmenter le volume", "Alza volume", "增大音量"]),
    ("Volume down", ["Leiser", "Bajar volumen", "Baisser le volume", "Abbassa volume", "减小音量"]),
    ("Next channel", ["Nächster Sender", "Canal siguiente", "Chaîne suivante", "Canale successivo", "下一频道"]),
    ("Previous channel", ["Vorheriger Sender", "Canal anterior", "Chaîne précédente", "Canale precedente", "上一频道"]),
    ("Show/hide programme overlay", ["Programm über dem Video ein-/ausblenden", "Mostrar/ocultar programa sobre el vídeo", "Afficher/masquer le programme sur la vidéo", "Mostra/nascondi programma sul video", "显示/隐藏视频上的节目"]),
    ("Leave full screen", ["Vollbild beenden", "Salir de pantalla completa", "Quitter le plein écran", "Esci da schermo intero", "退出全屏"]),
    ("Auto (default)", ["Auto (Standard)", "Automático (predeterminado)", "Auto (par défaut)", "Auto (predefinito)", "自动(默认)"]),
    ("Checking...", ["Prüfung...", "Comprobando...", "Vérification...", "Verifica...", "正在检查..."]),
    ("No releases yet", ["Noch keine Releases", "Aún no hay versiones", "Pas encore de version", "Nessuna versione disponibile", "暂无发布版本"]),
    ("You have the latest version ({})", ["Die neueste Version ist installiert ({})", "Tiene la última versión ({})", "Vous avez la dernière version ({})", "Hai l'ultima versione ({})", "已是最新版本 ({})"]),
    ("Version {} is available: {}", ["Version {} ist verfügbar: {}", "Versión {} disponible: {}", "La version {} est disponible : {}", "Versione {} disponibile: {}", "有新版本 {}:{}"]),
    ("Could not parse GitHub reply", ["GitHub-Antwort konnte nicht gelesen werden", "No se pudo interpretar la respuesta de GitHub", "Réponse de GitHub illisible", "Impossibile leggere la risposta di GitHub", "无法解析 GitHub 的响应"]),
    ("No releases yet or GitHub is unreachable", ["Noch keine Releases oder GitHub nicht erreichbar", "Aún no hay versiones o GitHub no responde", "Pas encore de version ou GitHub inaccessible", "Nessuna versione o GitHub non raggiungibile", "暂无发布版本或无法连接 GitHub"]),
    ("until", ["bis", "hasta", "jusqu'à", "fino alle", "至"]),
    ("Repository (app)", ["Repository (Anwendung)", "Repositorio (aplicación)", "Dépôt (application)", "Repository (applicazione)", "仓库(应用)"]),
    ("Releases (updates)", ["Releases (Updates)", "Versiones (actualizaciones)", "Versions (mises à jour)", "Release (aggiornamenti)", "发布版本(更新)"]),
    ("Playlists", ["Playlists", "Listas", "Listes", "Playlist", "播放列表"]),
    ("All files", ["Alle Dateien", "Todos los archivos", "Tous les fichiers", "Tutti i file", "所有文件"]),
];

/// Substitute `{}` placeholders in order.
pub fn fmt(template: String, args: &[&str]) -> String {
    let mut out = String::new();
    let mut it = args.iter();
    let mut rest = template.as_str();
    while let Some(p) = rest.find("{}") {
        out.push_str(&rest[..p]);
        out.push_str(it.next().copied().unwrap_or(""));
        rest = &rest[p + 2..];
    }
    out.push_str(rest);
    out
}



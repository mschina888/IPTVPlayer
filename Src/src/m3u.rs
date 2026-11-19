#[derive(Clone, Debug, Default)]
pub struct Channel {
    pub name: String,
    pub url: String,
    pub group: String,
    pub tvg_id: String,
    pub tvg_name: String,
    pub logo: String,
    /// сдвиг времени EPG в часах (tvg-shift=, может быть дробным), 0.0 если не задан
    pub tvg_shift: f64,
}

impl Channel {
    pub fn key(&self) -> String {
        if !self.tvg_id.is_empty() {
            self.tvg_id.clone()
        } else {
            self.name.clone()
        }
    }
}

#[derive(Default)]
pub struct Playlist {
    pub channels: Vec<Channel>,
    pub url_tvg: Vec<String>,
    /// cache= из заголовка #EXTM3U, мс (default-значение для режима «Авто» буфера, если провайдер его указал)
    pub cache_ms: Option<i64>,
}

fn attr(s: &str, key: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut from = 0;
    while let Some(pos) = s[from..].find(key) {
        let at = from + pos;
        let before_ok = at == 0 || !(bytes[at - 1].is_ascii_alphanumeric() || bytes[at - 1] == b'-' || bytes[at - 1] == b'_');
        let after = at + key.len();
        if before_ok && after < s.len() && bytes[after] == b'=' {
            let rest = &s[after + 1..];
            if let Some(r) = rest.strip_prefix('"') {
                return r.find('"').map(|e| r[..e].to_string());
            }
            let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            return Some(rest[..end].to_string());
        }
        from = at + key.len();
    }
    None
}

fn split_extinf(rest: &str) -> (String, String) {
    // attrs , name   (comma outside quotes)
    let mut inq = false;
    for (i, c) in rest.char_indices() {
        match c {
            '"' => inq = !inq,
            ',' if !inq => return (rest[..i].to_string(), rest[i + 1..].trim().to_string()),
            _ => {}
        }
    }
    (rest.to_string(), String::new())
}

pub fn parse(text: &str) -> Playlist {
    let mut pl = Playlist::default();
    let mut cur: Option<Channel> = None;
    let mut pending_group = String::new();
    for raw in text.lines() {
        let line = raw.trim().trim_start_matches('\u{feff}');
        if line.is_empty() {
            continue;
        }
        if line.starts_with("#EXTM3U") {
            if let Some(u) = attr(line, "url-tvg").or_else(|| attr(line, "x-tvg-url")) {
                pl.url_tvg = u.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
            }
            pl.cache_ms = attr(line, "cache").and_then(|v| v.parse::<i64>().ok());
            continue;
        }
        if let Some(rest) = line.strip_prefix("#EXTINF:") {
            let (attrs, name) = split_extinf(rest);
            let mut c = Channel::default();
            c.name = name;
            c.tvg_id = attr(&attrs, "tvg-id").unwrap_or_default();
            c.tvg_name = attr(&attrs, "tvg-name").unwrap_or_default();
            c.logo = attr(&attrs, "tvg-logo").unwrap_or_default();
            c.group = attr(&attrs, "group-title").unwrap_or_default();
            c.tvg_shift = attr(&attrs, "tvg-shift").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
            if c.group.is_empty() {
                c.group = std::mem::take(&mut pending_group);
            }
            cur = Some(c);
            continue;
        }
        if let Some(g) = line.strip_prefix("#EXTGRP:") {
            let g = g.trim().to_string();
            match cur.as_mut() {
                Some(c) => {
                    if c.group.is_empty() {
                        c.group = g;
                    }
                }
                None => pending_group = g,
            }
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        if let Some(mut c) = cur.take() {
            c.url = line.to_string();
            if c.name.is_empty() {
                c.name = c.url.clone();
            }
            pl.channels.push(c);
        }
    }
    pl
}

/// One quality variant of a channel's own HLS stream, from a master playlist's
/// #EXT-X-STREAM-INF (BANDWIDTH/RESOLUTION) line + the URI that follows it.
#[derive(Clone, Debug)]
pub struct Variant {
    pub bandwidth: i64,
    pub width: i32,
    pub height: i32,
    pub url: String,
}

/// Splits a comma-separated attribute list (as used by #EXT-X-STREAM-INF) into its parts,
/// without splitting inside a quoted value (so CODECS="a,b" stays one part).
fn split_csv_attrs(s: &str) -> Vec<&str> {
    let mut out = vec![];
    let mut start = 0;
    let mut in_quotes = false;
    for (i, b) in s.bytes().enumerate() {
        match b {
            b'"' => in_quotes = !in_quotes,
            b',' if !in_quotes => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&s[start..]);
    out
}

/// Like `attr()`, but for comma-separated attribute lists (#EXT-X-STREAM-INF), not the
/// space-separated lists used by #EXTM3U/#EXTINF. `attr()` cannot be reused here: it ends
/// an unquoted value at the first whitespace, which never occurs before the next comma.
fn attr_csv(s: &str, key: &str) -> Option<String> {
    for part in split_csv_attrs(s) {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix(key).and_then(|r| r.strip_prefix('=')) {
            let v = rest.trim();
            if let Some(stripped) = v.strip_prefix('"') {
                return Some(stripped.strip_suffix('"').unwrap_or(stripped).to_string());
            }
            return Some(v.to_string());
        }
    }
    None
}

/// Resolve a possibly-relative URI against the base playlist URL (same directory).
fn resolve_url(base: &str, rel: &str) -> String {
    if rel.starts_with("http://") || rel.starts_with("https://") {
        return rel.to_string();
    }
    match base.rfind('/') {
        Some(pos) => format!("{}/{}", &base[..pos], rel),
        None => rel.to_string(),
    }
}

/// Parse a channel's OWN stream playlist (fetched separately from the main channel
/// list). If it is a master playlist (contains #EXT-X-STREAM-INF), returns its
/// variants sorted by bandwidth, highest first. If it is a plain media playlist
/// (direct segments, no STREAM-INF) — today's case for most providers — returns
/// an empty list, meaning: nothing to switch between, play `base_url` as-is.
pub fn parse_variants(text: &str, base_url: &str) -> Vec<Variant> {
    let mut out = vec![];
    let mut pending: Option<(i64, i32, i32)> = None;
    for raw in text.lines() {
        let line = raw.trim();
        if let Some(rest) = line.strip_prefix("#EXT-X-STREAM-INF:") {
            let bw = attr_csv(rest, "BANDWIDTH").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
            let (mut w, mut h) = (0, 0);
            if let Some(res) = attr_csv(rest, "RESOLUTION") {
                if let Some((a, b)) = res.split_once('x') {
                    w = a.trim().parse().unwrap_or(0);
                    h = b.trim().parse().unwrap_or(0);
                }
            }
            pending = Some((bw, w, h));
            continue;
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((bw, w, h)) = pending.take() {
            out.push(Variant { bandwidth: bw, width: w, height: h, url: resolve_url(base_url, line) });
        }
    }
    out.sort_by(|a, b| b.bandwidth.cmp(&a.bandwidth));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_master_playlist_variants() {
        let m = "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=6000000,RESOLUTION=1920x1080\nhd/index.m3u8\n#EXT-X-STREAM-INF:BANDWIDTH=1500000,RESOLUTION=1280x720\nsd/index.m3u8\n";
        let v = parse_variants(m, "http://host/iptv/ch1/index.m3u8");
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].bandwidth, 6000000);
        assert_eq!(v[0].url, "http://host/iptv/ch1/hd/index.m3u8");
        assert_eq!(v[1].bandwidth, 1500000);
    }

    #[test]
    fn plain_media_playlist_has_no_variants() {
        let m = "#EXTM3U\n#EXT-X-TARGETDURATION:11\n#EXTINF:10.4,\nseg1.ts\n";
        assert!(parse_variants(m, "http://host/ch1/index.m3u8").is_empty());
    }

    #[test]
    fn parses_cache_and_tvg_shift() {
        let m = "#EXTM3U url-tvg=\"http://x/epg.xml\" cache=1050\n#EXTINF:0 tvg-shift=\"-1.5\",Ch\nhttp://x/ch.m3u8\n";
        let pl = parse(m);
        assert_eq!(pl.cache_ms, Some(1050));
        assert_eq!(pl.channels[0].tvg_shift, -1.5);
    }
}

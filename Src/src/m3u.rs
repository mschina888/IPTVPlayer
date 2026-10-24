#[derive(Clone, Debug, Default)]
pub struct Channel {
    pub name: String,
    pub url: String,
    pub group: String,
    pub tvg_id: String,
    pub tvg_name: String,
    pub logo: String,
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

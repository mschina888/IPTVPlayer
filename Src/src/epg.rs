use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Read};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct Prog {
    pub start: i64,
    pub stop: i64,
    pub title: String,
    pub desc: String,
}

#[derive(Default)]
pub struct Epg {
    pub progs: HashMap<String, Vec<Prog>>,
    /// normalised display-name -> channel id (matched channels only)
    pub names: HashMap<String, String>,
    pub ids: HashSet<String>,
    /// channel id -> icon url from <icon src=...>
    pub icons: HashMap<String, String>,
}

pub struct Wanted {
    pub ids: HashSet<String>,
    pub names: HashSet<String>,
}

pub fn norm(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).flat_map(|c| c.to_lowercase()).collect()
}

pub fn now_unix() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// seconds to add to UTC to get local time
pub fn local_offset_secs() -> i64 {
    use windows_sys::Win32::System::Time::{GetTimeZoneInformation, TIME_ZONE_INFORMATION};
    unsafe {
        let mut tz: TIME_ZONE_INFORMATION = std::mem::zeroed();
        let id = GetTimeZoneInformation(&mut tz);
        let extra = if id == 2 { tz.DaylightBias } else { tz.StandardBias };
        -((tz.Bias + extra) as i64) * 60
    }
}

pub fn hhmm(t: i64) -> String {
    let l = (t + local_offset_secs()).rem_euclid(86400);
    format!("{:02}:{:02}", l / 3600, (l % 3600) / 60)
}

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// "20261024143000 +0300"
pub fn parse_time(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.len() < 12 {
        return None;
    }
    let num = |a: usize, b: usize| s.get(a..b).and_then(|x| x.parse::<i64>().ok());
    let (y, mo, d, h, mi) = (num(0, 4)?, num(4, 6)?, num(6, 8)?, num(8, 10)?, num(10, 12)?);
    let sec = num(12, 14).unwrap_or(0);
    let mut t = days_from_civil(y, mo, d) * 86400 + h * 3600 + mi * 60 + sec;
    let rest = s.get(14..).unwrap_or("").trim();
    if rest.len() >= 5 {
        let sign = if rest.starts_with('-') { -1 } else { 1 };
        if let (Some(th), Some(tm)) = (rest.get(1..3).and_then(|x| x.parse::<i64>().ok()), rest.get(3..5).and_then(|x| x.parse::<i64>().ok())) {
            t -= sign * (th * 3600 + tm * 60);
        }
    }
    Some(t)
}

fn attr_val(e: &quick_xml::events::BytesStart, name: &[u8]) -> Option<String> {
    for a in e.attributes().flatten() {
        if a.key.as_ref() == name {
            return a.unescape_value().ok().map(|c| c.into_owned());
        }
    }
    None
}

/// Streaming parse of XMLTV. Keeps only programmes of channels that match `wanted`.
pub fn load(url: &str, wanted: &Wanted) -> Result<Epg, String> {
    let body = crate::net::reader(url)?;
    let mut br = BufReader::new(body);
    let gz = { br.fill_buf().map(|b| b.len() >= 2 && b[0] == 0x1f && b[1] == 0x8b).unwrap_or(false) };
    let reader: Box<dyn Read> = if gz { Box::new(flate2::read::MultiGzDecoder::new(br)) } else { Box::new(br) };
    let mut rdr = Reader::from_reader(BufReader::new(reader));
    let mut buf = Vec::new();

    let now = now_unix();
    let (keep_from, keep_to) = (now - 3 * 3600, now + 48 * 3600);

    let mut epg = Epg::default();
    let mut chan_names: HashMap<String, Vec<String>> = HashMap::new();
    let mut matched: Option<HashSet<String>> = None;

    let mut cur_chan: Option<String> = None;
    let mut in_name = false;
    let mut cur_prog: Option<(String, Prog)> = None;
    let mut in_title = false;
    let mut in_desc = false;

    loop {
        match rdr.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"channel" => {
                    cur_chan = attr_val(&e, b"id");
                    if let Some(id) = &cur_chan {
                        epg.ids.insert(id.clone());
                    }
                }
                b"display-name" => in_name = true,
                b"programme" => {
                    if matched.is_none() {
                        let mut m = HashSet::new();
                        for (id, names) in &chan_names {
                            if wanted.ids.contains(id) || names.iter().any(|n| wanted.names.contains(n)) {
                                m.insert(id.clone());
                                for n in names {
                                    epg.names.entry(n.clone()).or_insert_with(|| id.clone());
                                }
                            }
                        }
                        matched = Some(m);
                    }
                    let ch = attr_val(&e, b"channel").unwrap_or_default();
                    let start = attr_val(&e, b"start").and_then(|s| parse_time(&s));
                    let stop = attr_val(&e, b"stop").and_then(|s| parse_time(&s));
                    cur_prog = None;
                    if let (Some(start), Some(m)) = (start, matched.as_ref()) {
                        let stop = stop.unwrap_or(start + 3600);
                        if m.contains(&ch) && stop > keep_from && start < keep_to {
                            cur_prog = Some((ch, Prog { start, stop, title: String::new(), desc: String::new() }));
                        }
                    }
                }
                b"title" => in_title = true,
                b"desc" => in_desc = true,
                _ => {}
            },
            Ok(Event::Empty(e)) => {
                if e.name().as_ref() == b"icon" {
                    if let (Some(id), Some(src)) = (&cur_chan, attr_val(&e, b"src")) {
                        epg.icons.entry(id.clone()).or_insert(src);
                    }
                }
            }
            Ok(Event::Text(t)) => {
                if let Ok(txt) = t.unescape() {
                    if in_name {
                        if let Some(id) = &cur_chan {
                            chan_names.entry(id.clone()).or_default().push(norm(&txt));
                        }
                    } else if let Some((_, p)) = cur_prog.as_mut() {
                        if in_title && p.title.is_empty() {
                            p.title = txt.trim().to_string();
                        } else if in_desc && p.desc.is_empty() {
                            p.desc = txt.trim().to_string();
                        }
                    }
                }
            }
            Ok(Event::End(e)) => match e.name().as_ref() {
                b"channel" => cur_chan = None,
                b"display-name" => in_name = false,
                b"title" => in_title = false,
                b"desc" => in_desc = false,
                b"programme" => {
                    if let Some((ch, p)) = cur_prog.take() {
                        epg.progs.entry(ch).or_default().push(p);
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => {
                if epg.progs.is_empty() && chan_names.is_empty() {
                    return Err(e.to_string());
                }
                break;
            }
            _ => {}
        }
        buf.clear();
    }
    for v in epg.progs.values_mut() {
        v.sort_by_key(|p| p.start);
    }
    Ok(epg)
}

impl Epg {
    pub fn resolve(&self, tvg_id: &str, tvg_name: &str, name: &str) -> Option<String> {
        if !tvg_id.is_empty() && self.progs.contains_key(tvg_id) {
            return Some(tvg_id.to_string());
        }
        if !tvg_name.is_empty() {
            if let Some(id) = self.names.get(&norm(tvg_name)) {
                return Some(id.clone());
            }
        }
        self.names.get(&norm(name)).cloned()
    }

    pub fn now(&self, id: &str, t: i64) -> Option<&Prog> {
        self.progs.get(id)?.iter().find(|p| p.start <= t && t < p.stop)
    }
}


use std::io::Read;
use std::time::Duration;

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(40))
        .user_agent("IPTVPlayer/0.1 (MSChina888)")
        .build()
}

pub fn reader(url: &str) -> Result<Box<dyn Read + Send + Sync>, String> {
    let resp = agent().get(url).call().map_err(|e| e.to_string())?;
    Ok(resp.into_reader())
}

pub fn bytes(url: &str, max: u64) -> Result<Vec<u8>, String> {
    let mut r = reader(url)?;
    let mut out = Vec::new();
    r.by_ref().take(max).read_to_end(&mut out).map_err(|e| e.to_string())?;
    Ok(out)
}

pub fn text(url: &str) -> Result<String, String> {
    let b = bytes(url, 64 * 1024 * 1024)?;
    Ok(String::from_utf8_lossy(&b).into_owned())
}

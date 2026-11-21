//! Search a compact index and decompress only the requested offline shards.
use std::io::Read;
use serde::Deserialize;
use serde_json::{Value, json};

const INDEX: &str = include_str!("../profiles/library/index.tsv");
const DATA: &[u8] = include_bytes!("../profiles/library/catalog.bin");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    protocol: u32,
    #[serde(default)] query: String,
    #[serde(default = "all")] group: String,
    #[serde(default)] offset: usize,
    #[serde(default = "page_size")] limit: usize,
    #[serde(default)] ids: Vec<String>,
}
fn all() -> String { "all".into() }
fn page_size() -> usize { 32 }

pub fn search(request: Request) -> Result<Value, String> {
    if request.protocol != 2 || request.query.len() > 160 || request.group.len() > 80
        || request.offset > 100_000 || !(1..=64).contains(&request.limit) || request.ids.len() > 128
        || request.ids.iter().any(|id| id.len() > 80) {
        return Err("Invalid DSP catalog query.".into());
    }
    let base: Value = serde_json::from_str(include_str!("../presets.json")).map_err(|_| "Invalid DSP catalog.")?;
    let mut records = Vec::new();
    let mut total = 0;
    let query = request.query.to_lowercase();
    let compact: String = query.chars().filter(|c| c.is_alphanumeric()).collect();
    let matches = |id: &str, text: &str, normalized: &str, group: &str| {
        (request.ids.is_empty() || request.ids.iter().any(|wanted| wanted == id))
            && (request.group == "all" || request.group == group)
            && (query.split_whitespace().all(|part| text.contains(part)) || (!compact.is_empty() && normalized.contains(&compact)))
    };
    let mut accept = |preset: Value| {
        if total >= request.offset && records.len() < request.limit { records.push(preset); }
        total += 1;
    };
    for preset in base["presets"].as_array().ok_or("Invalid DSP catalog.")? {
        // The measured profiles are indexed once, with their original IDs.
        if preset["source_url"].as_str().is_some_and(|url| !url.is_empty()) { continue; }
        let text = format!("{} {} {}", preset["name"].as_str().unwrap_or(""), preset["category"].as_str().unwrap_or(""), preset["description"].as_str().unwrap_or("")).to_lowercase();
        let normalized: String = text.chars().filter(|c| c.is_alphanumeric()).collect();
        if matches(preset["id"].as_str().unwrap_or(""), &text, &normalized, preset["category"].as_str().unwrap_or("")) { accept(preset.clone()); }
    }
    let mut shard = None;
    for line in INDEX.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 5 { return Err("Invalid DSP search index.".into()); }
        if !matches(fields[1], &format!("{} {}", fields[2], fields[3]), fields[4], "Headphones") { continue; }
        // Count without decompressing records outside the requested page.
        if total >= request.offset && records.len() < request.limit {
            let index: usize = fields[0].parse().map_err(|_| "Invalid DSP search index.")?;
            let page = index / 64;
            if shard.as_ref().is_none_or(|(held, _)| *held != page) { shard = Some((page, decompress(page)?)); }
            let text = &shard.as_ref().unwrap().1;
            let row = text.lines().nth(index % 64).ok_or("Invalid DSP catalog shard.")?;
            let preset: Value = serde_json::from_str(row).map_err(|_| "Invalid DSP profile.")?;
            records.push(preset);
        }
        total += 1;
    }
    let next = (request.offset + records.len() < total).then_some(request.offset + records.len());
    let mut localizations = base["localizations"].clone();
    if let Some(spanish) = localizations["es"].as_object_mut() {
        for preset in &records {
            if preset["source_url"].as_str().is_some_and(|url| !url.is_empty()) {
                let id = preset["id"].as_str().unwrap_or("");
                spanish.entry(id.to_string()).or_insert_with(|| json!({"name":preset["name"], "category":"Audífonos",
                    "description":"Corrección medida. El sellado, las almohadillas, las puntas y los efectos del dispositivo cambian el resultado.", "source":preset["source"]}));
            }
        }
    }
    Ok(json!({"protocol":2, "presets":records, "total":total, "next":next,
        "localizations":localizations, "revision":"7ae0f56d53074872b028649617a22bbb4232feb7"}))
}
fn decompress(page: usize) -> Result<String, String> {
    let number = |start: usize| DATA.get(start..start+4).and_then(|word| word.try_into().ok()).map(u32::from_le_bytes).map(|value| value as usize).ok_or("Invalid DSP catalog shard.");
    if page >= number(0)? { return Err("Invalid DSP catalog shard.".into()); }
    let offset = number(4 + page * 8)?;
    let length = number(8 + page * 8)?;
    let bytes = DATA.get(offset..offset+length).ok_or("Invalid DSP catalog shard.")?;
    let mut text = String::new();
    flate2::read::ZlibDecoder::new(bytes).take(512 * 1024).read_to_string(&mut text).map_err(|_| "Invalid compressed DSP catalog.")?;
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request(query: &str) -> Request { Request { protocol: 2, query: query.into(), group: "all".into(), offset: 0, limit: 32, ids: Vec::new() } }
    #[test]
    fn search_is_bounded_offline_and_keeps_measured_variants() {
        let all = search(request("")).unwrap();
        assert!(all["total"].as_u64().unwrap() > 8000);
        assert_eq!(all["presets"].as_array().unwrap().len(), 32);
        let result = search(request("WH1000XM4")).unwrap();
        assert!(!result["presets"].as_array().unwrap().is_empty());
        assert!(result["presets"].as_array().unwrap().iter().any(|p| p["id"] == "sony-wh1000xm4"));
        let mut query = request(""); query.offset = 32;
        let next = search(query).unwrap();
        assert!(all["presets"].as_array().unwrap().iter().all(|p| !next["presets"].as_array().unwrap().iter().any(|n| n["id"] == p["id"])));
        assert!(search(request("no such headphone model 38398503298")).unwrap()["presets"].as_array().unwrap().is_empty());
    }
    #[test]
    fn invalid_limits_and_protocol_are_rejected() {
        let mut bad = request(""); bad.limit = 65; assert!(search(bad).is_err());
        let mut bad = request(""); bad.protocol = 1; assert!(search(bad).is_err());
    }
}

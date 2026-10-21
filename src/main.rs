use serde_json::Value;
use std::env;
use std::error::Error;
use std::time::Duration;

const MAX_BODY_BYTES: u64 = 10 * 1024 * 1024;
const MAX_ITEMS: usize = 20;

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let Some(flag) = args.next() else {
        print_usage();
        return Ok(());
    };
    if flag == "--help" || flag == "-h" {
        print_usage();
        return Ok(());
    }
    if flag != "--url" {
        return Err("expected --url <catalog-url>; use --help for usage".into());
    }
    let url = args.next().ok_or("missing catalog URL after --url")?;
    if args.next().is_some() {
        return Err("unexpected extra argument; use --help for usage".into());
    }
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err("catalog URL must start with https:// or http://".into());
    }

    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(15)))
        .build()
        .into();
    let mut request = agent
        .get(&url)
        .header("Accept", "application/json, audio/x-mpegurl, */*");
    if let Ok(authorization) = env::var("XUPER_AUTHORIZATION") {
        if !authorization.is_empty() {
            request = request.header("Authorization", &authorization);
        }
    }

    let mut response = request.call()?;
    let status = response.status();
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("unknown")
        .to_owned();
    let body = response
        .body_mut()
        .with_config()
        .limit(MAX_BODY_BYTES)
        .read_to_string()?;

    println!("HTTP {status} | Content-Type: {content_type}");
    if let Ok(json) = serde_json::from_str::<Value>(&body) {
        print_json_media(&json);
    } else if body.trim_start().starts_with("#EXTM3U") {
        print_m3u_media(&body);
    } else {
        println!("Response is not JSON or M3U; no media items could be identified.");
    }
    Ok(())
}

fn print_usage() {
    println!("Usage: xuper-cli --url <catalog-url>");
    println!("Optional: set XUPER_AUTHORIZATION to the complete Authorization header value.");
    println!("Fetches a JSON or M3U catalog and prints up to {MAX_ITEMS} media names.");
}

fn print_json_media(json: &Value) {
    let Some(items) = find_media_items(json, 0) else {
        println!("Valid JSON, but no recognized media list was found.");
        if let Some(object) = json.as_object() {
            let keys = object.keys().cloned().collect::<Vec<_>>().join(", ");
            println!("Top-level fields: {keys}");
        }
        return;
    };
    println!("Media items found: {}", items.len());
    for (index, item) in items.iter().take(MAX_ITEMS).enumerate() {
        let title = media_title(item).unwrap_or("(untitled)");
        let id = item
            .get("id")
            .or_else(|| item.get("vod_id"))
            .or_else(|| item.get("stream_id"))
            .map(value_text);
        match id {
            Some(id) => println!("{:>2}. {title} [id: {id}]", index + 1),
            None => println!("{:>2}. {title}", index + 1),
        }
    }
}

fn find_media_items(json: &Value, depth: usize) -> Option<&Vec<Value>> {
    if depth > 4 {
        return None;
    }
    if let Some(items) = json.as_array() {
        if items.iter().any(|item| media_title(item).is_some()) {
            return Some(items);
        }
        for item in items {
            if let Some(found) = find_media_items(item, depth + 1) {
                return Some(found);
            }
        }
    }
    if let Some(object) = json.as_object() {
        for key in [
            "data", "items", "list", "results", "channels", "movies", "series", "vod",
        ] {
            if let Some(value) = object.get(key) {
                if let Some(found) = find_media_items(value, depth + 1) {
                    return Some(found);
                }
            }
        }
    }
    None
}

fn media_title(item: &Value) -> Option<&str> {
    ["title", "name", "vod_name", "channel_name"]
        .into_iter()
        .find_map(|key| item.get(key).and_then(Value::as_str))
}

fn value_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

fn print_m3u_media(body: &str) {
    let names: Vec<&str> = body
        .lines()
        .filter(|line| line.starts_with("#EXTINF:"))
        .filter_map(|line| line.rsplit_once(',').map(|(_, name)| name.trim()))
        .collect();
    println!("Media items found: {}", names.len());
    for (index, name) in names.iter().take(MAX_ITEMS).enumerate() {
        println!("{:>2}. {name}", index + 1);
    }
}

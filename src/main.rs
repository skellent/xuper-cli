use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute, queue,
    style::Print,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use serde_json::Value;
use std::{
    env,
    error::Error,
    fs,
    io::{self, IsTerminal, Write},
    process::Command,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const MAX_BODY_BYTES: u64 = 10 * 1024 * 1024;
const MAX_LISTED_ITEMS: usize = 20;

struct Media {
    title: String,
    stream_url: Option<String>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [] | [_]
            if args
                .first()
                .is_none_or(|arg| arg == "--help" || arg == "-h") =>
        {
            print_usage();
            Ok(())
        }
        [mode, url] if mode == "--play" => play(url),
        [source, path] if source == "--url" || source == "--file" => {
            let media = load_media(source, path)?;
            print_media(&media);
            Ok(())
        }
        [source, path, action]
            if (source == "--url" || source == "--file")
                && (action == "--random" || action == "--search") =>
        {
            let media = load_media(source, path)?;
            let playable: Vec<&Media> = media
                .iter()
                .filter(|item| item.stream_url.is_some())
                .collect();
            if playable.is_empty() {
                return Err("catalog has no direct stream URLs to play".into());
            }
            let selected = if action == "--random" {
                let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
                Some((nanos % playable.len() as u128) as usize)
            } else {
                search_media(&playable)?
            };
            if let Some(index) = selected {
                let item = playable[index];
                println!("Playing: {}", item.title);
                play(item.stream_url.as_deref().expect("filtered playable items"))?;
            }
            Ok(())
        }
        _ => Err("invalid arguments; use --help for usage".into()),
    }
}

fn print_usage() {
    println!("Usage:");
    println!("  xuper-cli --url <catalog-url> [--search | --random]");
    println!("  xuper-cli --file <playlist.m3u> [--search | --random]");
    println!("  xuper-cli --play <stream-url>");
    println!("Optional: set XUPER_AUTHORIZATION to the complete Authorization header value.");
    println!("Search uses arrows, Enter and Esc. Playback requires mpv on PATH.");
}

fn is_stream_url(url: &str) -> bool {
    ["https://", "http://", "rtmp://", "rtsp://"]
        .iter()
        .any(|prefix| url.starts_with(prefix))
}

fn load_media(source: &str, path: &str) -> Result<Vec<Media>, Box<dyn Error>> {
    let body = if source == "--url" {
        if !path.starts_with("https://") && !path.starts_with("http://") {
            return Err("catalog URL must start with https:// or http://".into());
        }
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(15)))
            .build()
            .into();
        let mut request = agent
            .get(path)
            .header("Accept", "application/json, audio/x-mpegurl, */*");
        if let Ok(authorization) = env::var("XUPER_AUTHORIZATION") {
            if !authorization.is_empty() {
                request = request.header("Authorization", &authorization);
            }
        }
        let mut response = request.call()?;
        println!("HTTP {}", response.status());
        response
            .body_mut()
            .with_config()
            .limit(MAX_BODY_BYTES)
            .read_to_string()?
    } else {
        fs::read_to_string(path)?
    };

    let media = if let Ok(json) = serde_json::from_str::<Value>(&body) {
        let mut found = Vec::new();
        collect_json_media(&json, None, &mut found, 0);
        found
    } else if body.trim_start().starts_with("#EXTM3U") {
        parse_m3u(&body)
    } else {
        return Err("response is not a JSON catalog or M3U playlist".into());
    };
    Ok(media)
}

fn collect_json_media(value: &Value, parent: Option<&str>, out: &mut Vec<Media>, depth: usize) {
    if depth > 8 {
        return;
    }
    if let Some(array) = value.as_array() {
        for item in array {
            collect_json_media(item, parent, out, depth + 1);
        }
        return;
    }
    let Some(object) = value.as_object() else {
        return;
    };
    let own_title = media_title(value);
    let title = own_title.or(parent);
    let stream_url = [
        "stream_url",
        "play_url",
        "video_url",
        "direct_source",
        "url",
        "src",
        "file",
    ]
    .into_iter()
    .filter_map(|key| object.get(key).and_then(Value::as_str))
    .find(|url| is_stream_url(url))
    .map(str::to_owned);
    if let Some(title) = title {
        if own_title.is_some() || stream_url.is_some() {
            out.push(Media {
                title: clean_title(title),
                stream_url,
            });
        }
    }
    for key in [
        "data", "items", "list", "results", "channels", "movies", "series", "vod", "videos",
        "seasons", "episodes", "sources", "streams",
    ] {
        if let Some(child) = object.get(key) {
            collect_json_media(child, title, out, depth + 1);
        }
    }
}

fn media_title(value: &Value) -> Option<&str> {
    ["title", "name", "vod_name", "channel_name"]
        .into_iter()
        .find_map(|key| value.get(key).and_then(Value::as_str))
}

fn clean_title(title: &str) -> String {
    title.chars().filter(|ch| !ch.is_control()).collect()
}

fn parse_m3u(body: &str) -> Vec<Media> {
    let mut found = Vec::new();
    let mut pending_title = None;
    for line in body.lines().map(str::trim) {
        if let Some((_, title)) = line
            .strip_prefix("#EXTINF:")
            .and_then(|info| info.rsplit_once(','))
        {
            pending_title = Some(clean_title(title.trim()));
        } else if is_stream_url(line) {
            if let Some(title) = pending_title.take() {
                found.push(Media {
                    title,
                    stream_url: Some(line.to_owned()),
                });
            }
        }
    }
    found
}

fn print_media(media: &[Media]) {
    let playable = media
        .iter()
        .filter(|item| item.stream_url.is_some())
        .count();
    println!("Media items: {} | playable URLs: {playable}", media.len());
    for (index, item) in media.iter().take(MAX_LISTED_ITEMS).enumerate() {
        let marker = if item.stream_url.is_some() {
            "▶"
        } else {
            " "
        };
        println!("{:>2}. {marker} {}", index + 1, item.title);
    }
}

fn play(url: &str) -> Result<(), Box<dyn Error>> {
    if !is_stream_url(url) {
        return Err("stream URL must use HTTP, HTTPS, RTMP or RTSP".into());
    }
    let mut command = Command::new("mpv");
    if env::var("XUPER_MPV_HEADLESS").as_deref() == Ok("1") {
        command.args([
            "--no-config",
            "--vo=null",
            "--ao=null",
            "--frames=30",
            "--msg-level=all=warn",
        ]);
    }
    let status = command.arg("--").arg(url).status()?;
    if !status.success() {
        return Err(format!("mpv exited with {status}").into());
    }
    Ok(())
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Result<Self, Box<dyn Error>> {
        terminal::enable_raw_mode()?;
        if let Err(error) = execute!(io::stdout(), EnterAlternateScreen, Hide) {
            terminal::disable_raw_mode()?;
            return Err(error.into());
        }
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

fn search_media(media: &[&Media]) -> Result<Option<usize>, Box<dyn Error>> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err("--search needs an interactive terminal".into());
    }
    let _guard = TerminalGuard::enter()?;
    let mut query = String::new();
    let mut selected = 0usize;
    loop {
        let needle = query.to_lowercase();
        let matches: Vec<usize> = media
            .iter()
            .enumerate()
            .filter(|(_, item)| item.title.to_lowercase().contains(&needle))
            .map(|(index, _)| index)
            .collect();
        if selected >= matches.len() {
            selected = matches.len().saturating_sub(1);
        }
        draw_search(media, &matches, selected, &query)?;
        if let Event::Key(key) = event::read()? {
            if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
                continue;
            }
            match key.code {
                KeyCode::Esc => return Ok(None),
                KeyCode::Enter if !matches.is_empty() => return Ok(Some(matches[selected])),
                KeyCode::Up => selected = selected.saturating_sub(1),
                KeyCode::Down if selected + 1 < matches.len() => selected += 1,
                KeyCode::Backspace => {
                    query.pop();
                    selected = 0;
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(None);
                }
                KeyCode::Char(ch) if !ch.is_control() => {
                    query.push(ch);
                    selected = 0;
                }
                _ => {}
            }
        }
    }
}

fn draw_search(
    media: &[&Media],
    matches: &[usize],
    selected: usize,
    query: &str,
) -> Result<(), Box<dyn Error>> {
    let (width, height) = terminal::size()?;
    let mut output = io::stdout();
    queue!(
        output,
        MoveTo(0, 0),
        Clear(ClearType::All),
        Print("Search playable media  |  ↑/↓ select  Enter play  Esc quit"),
        MoveTo(0, 1),
        Print(format!("Search: {query}")),
        MoveTo(0, 2),
        Print(format!("{} matches", matches.len()))
    )?;
    let rows = height.saturating_sub(3) as usize;
    let first = selected.saturating_sub(rows.saturating_sub(1));
    for (row, index) in matches.iter().skip(first).take(rows).enumerate() {
        let marker = if first + row == selected { "> " } else { "  " };
        let label = format!("{marker}{}", media[*index].title);
        let visible: String = label.chars().take(width as usize).collect();
        queue!(output, MoveTo(0, row as u16 + 3), Print(visible))?;
    }
    output.flush()?;
    Ok(())
}

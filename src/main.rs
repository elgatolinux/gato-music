use clap::Parser;
use lofty::{read_from_path, AudioFile, TaggedFileExt, Accessor, ItemKey, ItemValue, Tag, TagItem, TagType};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use std::io::{BufRead, BufReader};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL or Search term
    query: String,

    /// Output format (flac, mp3, m4a, wav)
    #[arg(short, long, default_value = "flac")]
    format: String,

    /// Treat query as an album search term
    #[arg(short, long)]
    album: bool,

    /// Output directory (default: current directory)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Serialize)]
struct YtClient {
    #[serde(rename = "clientName")]
    client_name: String,
    #[serde(rename = "clientVersion")]
    client_version: String,
}

#[derive(Serialize)]
struct YtContext {
    client: YtClient,
}

#[derive(Serialize)]
struct YtSearchPayload {
    context: YtContext,
    query: String,
    params: String,
}

#[derive(Deserialize, Debug)]
struct LrcMatch {
    #[serde(rename = "syncedLyrics")]
    synced_lyrics: Option<String>,
    #[serde(rename = "plainLyrics")]
    plain_lyrics: Option<String>,
}

async fn download_ytdlp(dest_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("🐾 \x1b[35m[Gato-Music]\x1b[0m ¡Miau! No encontré yt-dlp. Descargándolo para ti... 📥");
    let url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux";
    let res = reqwest::get(url).await?;
    if !res.status().is_success() {
        return Err(format!("Failed to download yt-dlp: HTTP {}", res.status()).into());
    }
    let mut file = tokio::fs::File::create(dest_path).await?;
    let mut stream = res.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
    }

    // Set executable permissions
    let mut perms = fs::metadata(dest_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(dest_path, perms)?;

    println!("🐾 \x1b[35m[Gato-Music]\x1b[0m ¡Listo! yt-dlp descargado correctamente. 😺");
    Ok(())
}

async fn ensure_ytdlp() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let data_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    let app_dir = data_dir.join("gato-music");
    fs::create_dir_all(&app_dir)?;

    let ytdlp_path = app_dir.join("yt-dlp");
    if !ytdlp_path.exists() {
        download_ytdlp(&ytdlp_path).await?;
    } else {
        // Run yt-dlp -U to self-update in background (ignoring errors for speed)
        let _ = Command::new(&ytdlp_path).arg("-U").output();
    }

    Ok(ytdlp_path)
}

async fn search_album(query: &str) -> Result<String, Box<dyn std::error::Error>> {
    let payload = YtSearchPayload {
        context: YtContext {
            client: YtClient {
                client_name: "WEB_REMIX".to_string(),
                client_version: "1.20230712.01.00".to_string(),
            },
        },
        query: query.to_string(),
        params: "EgWKAQIYAWoMEAMQBBAJEAoQBRAREBAQ".to_string(),
    };

    let client = reqwest::Client::new();
    let res = client
        .post("https://music.youtube.com/youtubei/v1/search")
        .json(&payload)
        .send()
        .await?
        .text()
        .await?;

    let re = regex::Regex::new(r#"OLAK5uy_[a-zA-Z0-9_-]+"#).unwrap();
    if let Some(captures) = re.captures(&res) {
        let playlist_id = &captures[0];
        println!("🐾 \x1b[35m[Gato-Music]\x1b[0m ¡Encontré el álbum! 🎵 ID de lista: \x1b[36m{}\x1b[0m", playlist_id);
        Ok(format!("https://music.youtube.com/playlist?list={}", playlist_id))
    } else {
        Err("😿 No se pudo encontrar el álbum en YouTube Music.".into())
    }
}

async fn fetch_lyrics(artist: &str, track: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://lrclib.net/api/search?track_name={}&artist_name={}",
        urlencoding::encode(track),
        urlencoding::encode(artist)
    );
    let res = client.get(&url).send().await?;
    let matches: Vec<LrcMatch> = res.json().await?;

    if let Some(first) = matches.into_iter().next() {
        if let Some(lyrics) = first.synced_lyrics.or(first.plain_lyrics) {
            return Ok(Some(lyrics));
        }
    }
    Ok(None)
}

fn embed_lyrics(filepath: &str, lyrics: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut tagged_file = read_from_path(filepath)?;
    
    // Lofty v0.17 approach
    let tag = match tagged_file.primary_tag_mut() {
        Some(primary_tag) => primary_tag,
        None => {
            let tag_type = tagged_file.primary_tag_type();
            tagged_file.insert_tag(Tag::new(tag_type));
            tagged_file.primary_tag_mut().unwrap()
        }
    };

    tag.insert(TagItem::new(ItemKey::Lyrics, ItemValue::Text(lyrics.to_string())));
    
    tagged_file.save_to_path(filepath)?;
    Ok(())
}

fn extract_info(filepath: &str) -> Result<(Option<String>, Option<String>), Box<dyn std::error::Error>> {
    let tagged_file = read_from_path(filepath)?;
    if let Some(tag) = tagged_file.primary_tag() {
        let artist = tag.artist().map(|s| s.into_owned());
        let title = tag.title().map(|s| s.into_owned());
        return Ok((artist, title));
    }
    Ok((None, None))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    let ytdlp_path = ensure_ytdlp().await?;

    let mut target_url = args.query.clone();
    let is_album = args.album || target_url.contains("list=");

    if args.album && !target_url.starts_with("http") {
        println!("🐾 \x1b[35m[Gato-Music]\x1b[0m Buscando el álbum en YouTube Music... 💤");
        target_url = match search_album(&target_url).await {
            Ok(url) => url,
            Err(e) => {
                println!("🙀 \x1b[31m[Gato-Music]\x1b[0m ¡Oh no! {}", e);
                std::process::exit(1);
            }
        };
    } else if !target_url.starts_with("http") {
        println!("🐾 \x1b[35m[Gato-Music]\x1b[0m Preparando búsqueda para: \x1b[36m{}\x1b[0m... 🐾", target_url);
        target_url = format!("ytsearch:{}", target_url);
    }

    let mut outtmpl = if is_album {
        "%(playlist_title|Album)s/%(playlist_index)s. %(title)s.%(ext)s".to_string()
    } else {
        "%(title)s.%(ext)s".to_string()
    };

    if let Some(ref out_dir) = args.output {
        fs::create_dir_all(out_dir)?;
        let out_dir_str = out_dir.to_string_lossy();
        outtmpl = format!("{}/{}", out_dir_str, outtmpl);
    }

    println!("🐾 \x1b[35m[Gato-Music]\x1b[0m Iniciando descarga con yt-dlp... 🐱🚀");
    let mut command = Command::new(&ytdlp_path);
    command.args(&[
        "--format", "bestaudio/best",
        "--extract-audio",
        "--audio-format", &args.format,
        "--audio-quality", "192",
        "--add-metadata",
        "--embed-thumbnail",
        "-o", &outtmpl,
        // yt-dlp 2023+ supports printing the final filename
        "--exec", "echo AFTER_MOVE:{}", 
    ]);
    
    if !is_album {
        command.arg("--no-playlist");
    }

    command.arg(&target_url);

    let mut child = command
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    
    let mut downloaded_files = Vec::new();

    for line in reader.lines() {
        if let Ok(l) = line {
            println!("{}", l);
            if l.starts_with("AFTER_MOVE:") {
                let file = l.replace("AFTER_MOVE:", "");
                if std::path::Path::new(&file).exists() {
                    downloaded_files.push(file);
                }
            }
        }
    }

    let status = child.wait()?;
    if !status.success() {
        println!("🙀 \x1b[31m[Gato-Music]\x1b[0m ¡Oh no! yt-dlp encontró un error en su cacería. 😿");
        return Ok(());
    }

    if downloaded_files.is_empty() {
        println!("🐾 \x1b[35m[Gato-Music]\x1b[0m No se capturaron archivos descargados. 🙀");
        return Ok(());
    }

    println!("🐾 \x1b[35m[Gato-Music]\x1b[0m Buscando letras en LRCLIB para el festín... 🐟");
    for file in downloaded_files {
        if let Ok((Some(artist), Some(title))) = extract_info(&file) {
            println!("🐾 \x1b[35m[Gato-Music]\x1b[0m Olfateando letras para: \x1b[32m{} - {}\x1b[0m 🐱", artist, title);
            match fetch_lyrics(&artist, &title).await {
                Ok(Some(lyrics)) => {
                    println!("🐾 \x1b[35m[Gato-Music]\x1b[0m ¡Letras encontradas! Ronroneando e incrustándolas... ✍️✨");
                    if let Err(e) = embed_lyrics(&file, &lyrics) {
                        println!("🙀 [Error] No pude guardar las letras: {}", e);
                    }
                }
                Ok(None) => println!("🐾 \x1b[35m[Gato-Music]\x1b[0m No encontré letras para esta canción. 😿"),
                Err(e) => println!("🙀 [Error] Error al conectar con LRCLIB: {}", e),
            }
        } else {
            println!("🐾 \x1b[35m[Gato-Music]\x1b[0m Faltan metadatos, me salto la búsqueda de letras. 😿");
        }
    }

    println!("🐾 \x1b[35m[Gato-Music]\x1b[0m ¡Todo listo! Tu música está servida. ¡Miau! 🐟🐱🎉");
    Ok(())
}

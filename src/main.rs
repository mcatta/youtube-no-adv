use std::io::{Read, Write};
use std::net::TcpListener;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: youtube-no-adv <youtube-url>");
        eprintln!("Example: youtube-no-adv \"https://www.youtube.com/watch?v=SMhZJ3_wIUk\"");
        process::exit(1);
    }

    let video_id = match extract_video_id(&args[1]) {
        Some(id) => id,
        None => {
            eprintln!("Error: could not extract video ID from '{}'", args[1]);
            eprintln!("Expected a URL like https://www.youtube.com/watch?v=VIDEO_ID");
            process::exit(1);
        }
    };

    // Bind on a random available port.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap_or_else(|e| {
        eprintln!("Error: could not bind local server: {}", e);
        process::exit(1);
    });
    let port = listener.local_addr().unwrap().port();
    let url = format!("http://127.0.0.1:{}", port);

    open::that(&url).unwrap_or_else(|e| {
        eprintln!("Error: failed to open browser: {}", e);
        process::exit(1);
    });

    // Serve one request then exit.
    if let Ok((mut stream, _)) = listener.accept() {
        // Drain the request so the browser doesn't see a connection reset.
        let mut buf = [0u8; 4096];
        let _ = stream.read(&mut buf);

        let html = build_player_html(&video_id);
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            html.len(),
            html
        );
        let _ = stream.write_all(response.as_bytes());
    }
}

fn build_player_html(video_id: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>YouTube (no cookies)</title>
  <style>
    * {{ margin: 0; padding: 0; box-sizing: border-box; }}
    body {{ background: #000; display: flex; justify-content: center; align-items: center; height: 100vh; }}
    iframe {{ width: 100vw; height: 100vh; border: none; }}
  </style>
</head>
<body>
  <iframe
    src="https://www.youtube-nocookie.com/embed/{video_id}?autoplay=1"
    allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
    referrerpolicy="strict-origin-when-cross-origin"
    allowfullscreen>
  </iframe>
</body>
</html>
"#
    )
}

fn extract_video_id(input: &str) -> Option<String> {
    let parsed = url::Url::parse(input).ok()?;

    let host = parsed.host_str()?;

    // Standard watch URL: youtube.com/watch?v=ID
    if host.contains("youtube.com") {
        if parsed.path() == "/watch" {
            return parsed
                .query_pairs()
                .find(|(k, _)| k == "v")
                .map(|(_, v)| v.into_owned());
        }
        // Embed URL: youtube.com/embed/ID
        if let Some(mut segments) = parsed.path_segments() {
            if segments.next() == Some("embed") {
                if let Some(id) = segments.next() {
                    if !id.is_empty() {
                        return Some(id.to_owned());
                    }
                }
            }
        }
    }

    // Short URL: youtu.be/ID
    if host == "youtu.be" {
        return parsed
            .path_segments()
            .and_then(|mut s| s.next())
            .map(|s| s.to_owned());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_watch_url() {
        let id = extract_video_id("https://www.youtube.com/watch?v=SMhZJ3_wIUk");
        assert_eq!(id, Some("SMhZJ3_wIUk".to_owned()));
    }

    #[test]
    fn test_watch_url_with_extra_params() {
        let id = extract_video_id(
            "https://www.youtube.com/watch?v=SMhZJ3_wIUk&list=PLxxx&index=3",
        );
        assert_eq!(id, Some("SMhZJ3_wIUk".to_owned()));
    }

    #[test]
    fn test_short_url() {
        let id = extract_video_id("https://youtu.be/SMhZJ3_wIUk");
        assert_eq!(id, Some("SMhZJ3_wIUk".to_owned()));
    }

    #[test]
    fn test_embed_url() {
        let id = extract_video_id("https://www.youtube.com/embed/SMhZJ3_wIUk");
        assert_eq!(id, Some("SMhZJ3_wIUk".to_owned()));
    }

    #[test]
    fn test_invalid_url() {
        let id = extract_video_id("not-a-url");
        assert_eq!(id, None);
    }

    #[test]
    fn test_non_youtube_url() {
        let id = extract_video_id("https://www.example.com/watch?v=something");
        assert_eq!(id, None);
    }

    #[test]
    fn test_youtube_url_without_video_id() {
        let id = extract_video_id("https://www.youtube.com/feed/subscriptions");
        assert_eq!(id, None);
    }

    #[test]
    fn test_player_html_contains_video_id() {
        let html = build_player_html("SMhZJ3_wIUk");
        assert!(html.contains("https://www.youtube-nocookie.com/embed/SMhZJ3_wIUk"));
        assert!(html.contains("autoplay=1"));
        assert!(html.contains("allowfullscreen"));
        assert!(html.contains("referrerpolicy"));
    }

    #[test]
    fn test_player_html_is_valid_structure() {
        let html = build_player_html("abc123");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<iframe"));
        assert!(html.contains("</iframe>"));
    }
}

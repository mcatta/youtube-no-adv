# youtube-no-adv

Open any YouTube video in your browser using the privacy-enhanced `youtube-nocookie.com` embed, which avoids tracking cookies on page load.

## How it works

The app starts a local HTTP server, opens your browser to `http://127.0.0.1:<port>`, and serves a minimal HTML page containing a full-screen `youtube-nocookie.com` iframe. Using a real HTTP origin (instead of a `file://` URL) is required for YouTube to allow the embed.

## Installation

Requires [Rust](https://rustup.rs).

```bash
cargo install --path .
```

## Usage

```bash
youtube-no-adv "<youtube-url>"
```

Supported URL formats:

```bash
youtube-no-adv "https://www.youtube.com/watch?v=SMhZJ3_wIUk"
youtube-no-adv "https://youtu.be/SMhZJ3_wIUk"
youtube-no-adv "https://www.youtube.com/embed/SMhZJ3_wIUk"
```

> **Note:** always quote the URL in your shell to prevent `?` from being interpreted as a glob character.

## Running tests

```bash
cargo test
```

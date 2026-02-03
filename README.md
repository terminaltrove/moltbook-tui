<div align="center">
  <h1>moltbook-tui</h1>
  <p>A TUI client for Moltbook, the social network for AI Agents.</p>
  <img src="media/moltbook-tui-feed-new.png" width="800" alt="moltbook-tui preview">
  <br><br>
  <video alt="moltbook-tui demo" src="https://github.com/user-attachments/assets/1f92c463-9ae4-45ac-be31-70507a245294" width="800" autoplay loop muted playsinline></video>
  <p>
    <img src="https://img.shields.io/badge/License-MIT-brightgreen.svg" alt="License: MIT">
    <img src="https://img.shields.io/badge/Rust-1.70%2B-orange" alt="Rust">
    <img src="https://ratatui.rs/built-with-ratatui/badge.svg" alt="Built With Ratatui">
  </p>
</div>

## Description

Browse [Moltbook](https://www.moltbook.com), the social network for AI agents, from your terminal. View posts, comments, leaderboards, community stats, and real-time updates.

**Features at a glance:**
- Vim-style keyboard navigation (`j`/`k`, `Enter`, `Esc`)
- Auto-refreshing feed with background updates
- Threaded comment views
- Agent leaderboards and statistics
- Mouse support
- Works without authentication (API key optional)

This TUI application is built using the [Ratatui](https://github.com/ratatui/ratatui) framework.

## Installation

### From GitHub Releases

You can download latest binary rom GitHub releases [for your platform here](https://github.com/terminaltrove/moltbook-tui/releases).

### Using Homebrew

```bash
brew install terminaltrove/tap/moltbook-tui
```

### Using Cargo

```bash
cargo install moltbook-tui
```

### Using eget

```bash
eget terminaltrove/moltbook-tui
```

### Build from Source

Requirements: Rust 1.70 or later

```bash
git clone https://github.com/terminaltrove/moltbook-tui.git
cd moltbook-tui
cargo build --release
# binary: target/release/moltbook
```

## Features

- **Real-time Feed**: Browse the latest new posts from AI agents with auto-refresh.
- **Post Details**: View full posts with threaded comments
- **Agent Leaderboard**: See the most active and top-scoring agents
- **Submolts**: Browse different communities (subreddits for AI)
- **Statistics**: View network-wide stats and metrics
- **Settings**: Configure API key and preferences
- **Mouse Support**: Click to navigate and scroll
- **Theming**: Moltbook-themed interface with teal and red accents

## Demo Screenshots

<table>
  <tr>
    <td align="center"><img src="media/moltbook-tui-feed-new.png" width="300"><br><sub>Main Feed</sub></td>
    <td align="center"><img src="media/moltbook-tui-post-comments.png" width="300"><br><sub>Post Comments</sub></td>
    <td align="center"><img src="media/moltbook-tui-leaderboard.png" width="300"><br><sub>Leaderboard</sub></td>
  </tr>
  <tr>
    <td align="center"><img src="media/moltbook-tui-submolts.png" width="300"><br><sub>Submolts</sub></td>
    <td align="center"><img src="media/moltbook-tui-stats.png" width="300"><br><sub>Statistics</sub></td>
    <td align="center"><img src="media/moltbook-tui-about.png" width="300"><br><sub>About</sub></td>
  </tr>
</table>

## Usage

Launch the TUI:

```bash
moltbook
```

### TUI Controls

**Global**
- `q` or `Ctrl+C` - Quit
- `Esc` - Go back / Cancel
- `?` - Help

**Feed View**
- `j` / `k` or `↓` / `↑` - Navigate posts
- `Enter` - View post details
- `l` - View leaderboard
- `s` - View submolts
- `t` - View statistics
- `r` - Refresh feed
- `,` - Settings

**Post Detail**
- `j` / `k` or `↓` / `↑` - Scroll content
- `Esc` or `Backspace` - Back to feed

**Leaderboard / Lists**
- `j` / `k` or `↓` / `↑` - Navigate items
- `Tab` - Switch between tabs/views
- `Esc` or `Backspace` - Back to feed

## Configuration

moltbook-tui works without configuration. Optionally, provide an API key for authenticated features.

### API Key

Set via environment variable:

```bash
export MOLTBOOK_API_KEY="your_api_key_here"
```

Or config file (`~/.moltbook-tui/config.toml`):

```toml
api_key = "your_api_key_here"
```

## Development

```bash
cargo run                    # Run in dev mode
cargo test                   # Run tests
cargo clippy                 # Lint
cargo fmt                    # Format
```

## License

MIT

## About Moltbook

[Moltbook](https://www.moltbook.com) is a Reddit-like social network for AI agents. Agents can create posts, comment, vote, and participate in communities called "submolts". This TUI provides a terminal interface for browsing the network.

## Disclaimer

This project is not affiliated with, endorsed by, or connected to Moltbook.

EOF

![Moltbook Screenshot](./media/image.png)

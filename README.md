# val-insta-locker

Valorant instalocker built with Rust + Tauri + React. Automatically selects and locks your agent the moment agent select starts.

## Features

- **Instant lock or select-only mode** — lock immediately or just hover (Valorant auto-locks when the timer runs out, no AFK penalty)
- **Configurable delay** — 0 to 2000ms slider to look less obvious
- **Map macros** — assign a different agent per map, switches automatically
- **Break protection** — re-arms if the match cancels mid agent-select
- **Match abandon** — quit pregame with one click from inside the app
- **Activity log** — real-time feed of every action the tool takes
- **Persistent config** — all settings saved locally as JSON

## Download

Go to [Releases](../../releases/latest) and grab the `.exe` installer.

Requires Windows 10/11. Valorant must be running.

## Build from source

**Requirements:** Rust stable, Node 18+, Windows SDK (for Tauri)

```bash
git clone https://github.com/YOUR_USERNAME/val-insta-locker
cd val-insta-locker
npm install
npm run tauri build
```

Output is in `src-tauri/target/release/bundle/`.

## How it works

Valorant exposes a local HTTP API on a random port. The credentials are stored in a lockfile at:

```
%LOCALAPPDATA%\Riot Games\Riot Client\Config\lockfile
```

The app reads that file on startup, then polls the `/product-session/v1/external-sessions` endpoint every 600ms to detect phase changes. When the phase becomes `pregame`, it fetches the match ID and fires either a `select` or `lock` request to the pregame endpoints.

## Stack

- **Backend** — Rust via Tauri 2, reqwest for HTTP
- **Frontend** — React 19, TypeScript, Tailwind CSS v4, Zustand
- **Build** — Vite 7, GitHub Actions (Windows release)

## Disclaimer

For personal and educational use only. All risks from using this tool — including account bans or restrictions — are your own responsibility. This tool is not affiliated with or endorsed by Riot Games.

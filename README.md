<div align="center">

<img src="src-tauri/icons/128x128.png" width="80" alt="logo" />

# val-insta-locker

<p>
  <a href="https://github.com/daniilsys/val-insta-locker/releases/latest">
    <img src="https://img.shields.io/github/v/release/daniilsys/val-insta-locker?style=flat-square&color=FF4655&label=release" alt="Latest Release" />
  </a>
  <a href="https://github.com/daniilsys/val-insta-locker/releases/latest">
    <img src="https://img.shields.io/github/downloads/daniilsys/val-insta-locker/total?style=flat-square&color=FF4655&label=downloads" alt="Downloads" />
  </a>
  <a href="https://github.com/daniilsys/val-insta-locker/actions/workflows/release.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/daniilsys/val-insta-locker/release.yml?style=flat-square&color=FF4655&label=build" alt="Build Status" />
  </a>
  <img src="https://img.shields.io/badge/platform-Windows%2010%2F11-FF4655?style=flat-square" alt="Platform" />
  <img src="https://img.shields.io/badge/stack-Rust%20%2B%20Tauri%20%2B%20React-FF4655?style=flat-square" alt="Stack" />
</p>

<p>Valorant instalocker built with Rust + Tauri + React.<br/>Locks your agent the instant agent select starts.</p>

</div>

---

## Download

<a href="https://github.com/daniilsys/val-insta-locker/releases/latest">
  <img src="https://img.shields.io/badge/Download%20Installer-FF4655?style=for-the-badge&logo=windows&logoColor=white" alt="Download" />
</a>

> Requires **Windows 10/11 x64**. Valorant must be running when you launch the app.

---

## Features

| | |
|---|---|
| **Lock / Select mode** | Lock the agent instantly, or just hover it — Valorant auto-locks when the timer runs out (no AFK penalty in select-only) |
| **Configurable delay** | Slider from 0 to 2000ms so the lock doesn't look instant |
| **Map macros** | Assign a different agent per map — the app detects the map and picks automatically |
| **Break protection** | Re-arms if the match cancels mid agent-select |
| **Match abandon** | Quit pregame with one click from inside the app |
| **Activity log** | Real-time feed of every action the tool performs |
| **Persistent config** | All settings saved locally as JSON |

---

## How it works

Valorant runs a local HTTP API on a random port. Credentials are written to a lockfile at:

```
%LOCALAPPDATA%\Riot Games\Riot Client\Config\lockfile
```

The app reads that file on startup and polls `/product-session/v1/external-sessions` every 600ms. When the phase becomes `pregame`, it grabs the match ID and fires a `select` or `lock` request to the pregame endpoints.

---

## Build from source

**Requirements:** Rust stable · Node 18+ · Windows SDK

```bash
git clone https://github.com/daniilsys/val-insta-locker
cd val-insta-locker
npm install
npm run tauri build
```

Output: `src-tauri/target/release/bundle/`

---

## Stack

<p>
  <img src="https://img.shields.io/badge/Rust-000000?style=flat-square&logo=rust&logoColor=white" />
  <img src="https://img.shields.io/badge/Tauri%202-24C8D8?style=flat-square&logo=tauri&logoColor=white" />
  <img src="https://img.shields.io/badge/React%2019-61DAFB?style=flat-square&logo=react&logoColor=black" />
  <img src="https://img.shields.io/badge/TypeScript-3178C6?style=flat-square&logo=typescript&logoColor=white" />
  <img src="https://img.shields.io/badge/Tailwind%20CSS%20v4-06B6D4?style=flat-square&logo=tailwindcss&logoColor=white" />
  <img src="https://img.shields.io/badge/Vite%207-646CFF?style=flat-square&logo=vite&logoColor=white" />
</p>

---

## Disclaimer

For personal and educational use only. All risks — including account bans or restrictions — are your own responsibility. Not affiliated with or endorsed by Riot Games.

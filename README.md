<p align="center">
  <img src="src-tauri/icons/icon.png" alt="LibreRTC logo" width="128" />
</p>

<h1 align="center">LibreRTC Client</h1>

<p align="center">
  A compact Windows desktop client for LibreRTC with Proxy and TUN modes.
</p>

<table align="center">
  <tr>
    <td align="center" width="760">
      <h3>Keep LibreRTC independent</h3>
      <p>If this project helps you, support further development, testing, and network tooling.</p>
      <a href="https://t.me/tribute/app?startapp=dK9j">
        <img alt="Support LibreRTC via Tribute" src="https://img.shields.io/badge/Support%20LibreRTC-Tribute-0EA5E9?style=for-the-badge&logo=telegram&logoColor=white&labelColor=111827">
      </a>
      <a href="https://nowpayments.io/donation/svllvsx">
        <img alt="Support with crypto via NOWPayments" src="https://img.shields.io/badge/Support%20LibreRTC-NOWPayments-22C55E?style=for-the-badge&logo=bitcoin&logoColor=white&labelColor=111827">
      </a>
    </td>
  </tr>
</table>

<table align="center">
  <tr>
    <td align="center" width="370">
      <a href="https://t.me/svllvsxprod">
        <img alt="svllvsxprod Telegram" src="https://img.shields.io/badge/Telegram-svllvsxprod-26A5E4?style=for-the-badge&logo=telegram&logoColor=white&labelColor=111827">
      </a>
      <p>Project updates, builds, and release notes.</p>
    </td>
    <td align="center" width="370">
      <a href="https://t.me/openlibrecommunity">
        <img alt="Open Libre Community Telegram" src="https://img.shields.io/badge/Telegram-Open%20Libre%20Community-26A5E4?style=for-the-badge&logo=telegram&logoColor=white&labelColor=111827">
      </a>
      <p>Community chat, feedback, and testing.</p>
    </td>
  </tr>
</table>

<p align="center">
  <img alt="Windows" src="https://img.shields.io/badge/Windows-Tauri%20v2-0078D4?style=for-the-badge&logo=windows&logoColor=white">
  <img alt="Frontend" src="https://img.shields.io/badge/UI-TypeScript%20%2B%20Vite-646CFF?style=for-the-badge&logo=vite&logoColor=white">
  <img alt="Backend" src="https://img.shields.io/badge/Backend-Rust-000000?style=for-the-badge&logo=rust&logoColor=white">
  <img alt="TUN" src="https://img.shields.io/badge/TUN-Embedded%20sing--box-22C55E?style=for-the-badge">
</p>

<p align="center">
  <a href="#screenshots">Screenshots</a> ·
  <a href="#what-it-does">What It Does</a> ·
  <a href="#features">Features</a> ·
  <a href="#build">Build</a> ·
  <a href="#security-model">Security</a>
</p>

## Screenshots

<p align="center">
  <img src="screens/1.png" width="150" alt="Connection screen" />
  <img src="screens/2.png" width="150" alt="Profiles" />
  <img src="screens/3.png" width="150" alt="Logs" />
  <img src="screens/4.png" width="150" alt="Settings" />
  <img src="screens/5.png" width="150" alt="Tray flow" />
</p>

## What It Does

LibreRTC Client is a Windows desktop app for importing `olcrtc://` profiles and running a local LibreRTC connection without exposing runtime details to the user.

It supports two operating modes:

- Proxy mode: starts `olcrtc.exe` and exposes a local SOCKS proxy.
- TUN mode: starts `olcrtc.exe`, then routes system traffic through `librertc-net-service.exe` with an embedded sing-box engine.

## Features

- Import and store multiple LibreRTC server profiles.
- Proxy mode with automatic local port selection.
- TUN mode through a Windows service and embedded sing-box.
- Real-time traffic counters from `olcrtc` runtime stats.
- RU/EN interface language.
- Fixed-size frameless Windows UI with tray controls.
- External community/support links opened through the system browser.
- Local Zector font bundle; no web font linking.

## Architecture

```text
Windows desktop app
  -> Tauri UI
  -> Rust backend commands
  -> olcrtc.exe local SOCKS runtime
  -> optional LibreRTCNetService TUN bridge
  -> LibreRTC node
```

TUN mode keeps the Windows service installed after disconnect. Disconnect stops only the embedded tunnel engine.

## Requirements

- Windows 10/11.
- WebView2 Runtime.
- Rust stable toolchain.
- Node.js and npm.
- Go toolchain for rebuilding `librertc-net-service.exe`.
- `olcrtc.exe` placed next to the released client executable at runtime.

## Build

Install frontend dependencies:

```bash
npm install
```

Build the frontend:

```bash
npm run build
```

Build the Tauri client:

```bash
npm run tauri -- build
```

Build the TUN service:

```bash
cd librertc-net-service
go build -tags with_gvisor -ldflags "-H=windowsgui" -o ../src-tauri/target/release/librertc-net-service.exe ./cmd/librertc-net-service
```

Runtime files expected next to `librertc-client.exe`:

```text
librertc-client.exe
librertc-net-service.exe
olcrtc.exe
```

## Project Layout

```text
src/                    TypeScript UI, styles, local assets
src-tauri/              Rust Tauri backend and Windows shell integration
librertc-net-service/   Go Windows service with embedded sing-box
screens/                Public screenshots for documentation
```

## Security Model

- Live `olcrtc://` profiles are not committed.
- Runtime profile data is stored under `%LOCALAPPDATA%\LibreRTC\client`.
- Build outputs, binaries, local imports, and profile files are ignored by git.
- External links are opened only through a strict backend allowlist.
- TUN mode blocks IPv6 in the current MVP to avoid leaks.

## Notes

This repository contains the desktop client only. The core runtime and server node are maintained separately under the LibreRTC project.

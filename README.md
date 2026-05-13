<p align="center">
  <img src="src-tauri/icons/icon.png" alt="LibreRTC logo" width="128" />
</p>

<h1 align="center">LibreRTC Client</h1>

<p align="center">
  Лёгкий Windows-клиент для LibreRTC с Proxy и TUN режимами, системным tray, реальной телеметрией трафика и встроенным TUN-сервисом на sing-box.
</p>

<p align="center">
  <img alt="Windows" src="https://img.shields.io/badge/Windows-Tauri%20v2-0078D4?style=for-the-badge&logo=windows&logoColor=white">
  <img alt="Frontend" src="https://img.shields.io/badge/UI-TypeScript%20%2B%20Vite-646CFF?style=for-the-badge&logo=vite&logoColor=white">
  <img alt="Backend" src="https://img.shields.io/badge/Backend-Rust-000000?style=for-the-badge&logo=rust&logoColor=white">
  <img alt="TUN" src="https://img.shields.io/badge/TUN-Embedded%20sing--box-22C55E?style=for-the-badge">
</p>

<p align="center">
  <a href="#скриншоты">Скриншоты</a> ·
  <a href="#что-это">Что это</a> ·
  <a href="#возможности">Возможности</a> ·
  <a href="#сборка">Сборка</a> ·
  <a href="#безопасность">Безопасность</a>
</p>

<table align="center">
  <tr>
    <td align="center" width="760">
      <h3>Поддержка и сообщество</h3>
      <p>Поддержать разработку, следить за обновлениями или присоединиться к обсуждению LibreRTC.</p>
      <table align="center">
        <tr>
          <td align="center" width="350"><a href="https://t.me/tribute/app?startapp=dK9j"><img src="screens/donate-tribute-v2.svg" width="340" alt="Support LibreRTC via Tribute" /></a></td>
          <td align="center" width="350"><a href="https://nowpayments.io/donation/svllvsx"><img src="screens/donate-nowpayments-v2.svg" width="340" alt="Donate with NOWPayments" /></a></td>
        </tr>
        <tr>
          <td align="center" width="350"><a href="https://t.me/svllvsxprod"><img src="screens/telegram-updates-v2.svg" width="340" alt="svllvsxprod Telegram" /></a></td>
          <td align="center" width="350"><a href="https://t.me/openlibrecommunity"><img src="screens/telegram-community-v2.svg" width="340" alt="Open Libre Community Telegram" /></a></td>
        </tr>
      </table>
    </td>
  </tr>
</table>

## Скриншоты

<p align="center">
  <img src="screens/1.png" width="150" alt="Экран подключения" />
  <img src="screens/2.png" width="150" alt="Профили" />
  <img src="screens/3.png" width="150" alt="Логи" />
  <img src="screens/4.png" width="150" alt="Настройки" />
  <img src="screens/5.png" width="150" alt="Tray" />
</p>

## Что это

LibreRTC Client импортирует `olcrtc://` профили и запускает локальное подключение к LibreRTC node без ручной настройки runtime-компонентов пользователем.

Клиент поддерживает два режима:

- Proxy: запускает `olcrtc.exe` и поднимает локальный SOCKS proxy.
- TUN: запускает `olcrtc.exe`, затем направляет системный трафик через `librertc-net-service.exe` со встроенным sing-box engine.

## Возможности

- Импорт и хранение нескольких LibreRTC профилей.
- Proxy режим с автоматическим выбором свободного локального порта.
- TUN режим через Windows service и embedded sing-box.
- Реальная телеметрия download/upload из runtime-статистики `olcrtc`.
- RU/EN интерфейс.
- Fixed-size frameless UI, custom titlebar и tray controls.
- Внешние ссылки открываются только через системный браузер.
- Локальный Inter Regular font bundle, без web-font ссылок.

## Архитектура

```text
Windows desktop app
  -> Tauri UI
  -> Rust backend commands
  -> olcrtc.exe local SOCKS runtime
  -> optional LibreRTCNetService TUN bridge
  -> LibreRTC node
```

TUN service остаётся установленным после отключения. Disconnect останавливает только embedded tunnel engine.

## Требования

- Windows 10/11.
- WebView2 Runtime.
- Rust stable toolchain.
- Node.js и npm.
- Go toolchain для пересборки `librertc-net-service.exe`.
- `olcrtc.exe` рядом с release `librertc-client.exe` во время запуска.

## Сборка

Установить frontend dependencies:

```bash
npm install
```

Собрать frontend:

```bash
npm run build
```

Собрать Tauri-клиент:

```bash
npm run tauri -- build
```

Собрать TUN service:

```bash
cd librertc-net-service
go build -tags with_gvisor -ldflags "-H=windowsgui" -o ../src-tauri/target/release/librertc-net-service.exe ./cmd/librertc-net-service
```

Runtime-файлы рядом с `librertc-client.exe`:

```text
librertc-client.exe
librertc-net-service.exe
olcrtc.exe
```

## Структура

```text
src/                    TypeScript UI, styles, local assets
src-tauri/              Rust Tauri backend and Windows shell integration
librertc-net-service/   Go Windows service with embedded sing-box
screens/                Screenshots and README assets
```

## Безопасность

- Live `olcrtc://` профили не коммитятся.
- Runtime profile хранится в `%LOCALAPPDATA%\LibreRTC\client`.
- Build outputs, binaries, local imports и profile files исключены из git.
- Внешние ссылки проходят через strict backend allowlist.
- В текущем MVP TUN режим блокирует IPv6, чтобы избежать leak.

## Теги

`librertc` `vpn-client` `tauri` `rust` `typescript` `windows` `tun` `sing-box` `socks-proxy` `webrtc`

## Примечание

Этот репозиторий содержит только desktop client. Core runtime и server node ведутся отдельно в рамках LibreRTC.

# LibreRTC Net Service

Privileged Windows service for LibreRTC networking.

The desktop UI should not own TUN lifecycle directly. This service is the long-lived privileged component that will host the sing-box-based TUN engine and expose a small localhost control API to the Tauri client.

Initial control API:

- `GET /health`
- `GET /status`
- `POST /start`
- `POST /stop`

The current skeleton is intentionally conservative: it provides service install/uninstall/run plumbing and the control API, while the sing-box embedded engine is added behind the `Engine` interface next.

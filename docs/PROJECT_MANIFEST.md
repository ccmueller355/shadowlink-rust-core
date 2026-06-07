# ShadowLink — Family Matrix App Manifest

## Project Name

**ShadowLink** — Privacy-first family communications

## Core Purpose

Privacy-first family app: E2EE chat, media/picture sharing, and location sharing on Matrix protocol. Lightweight custom client. One-time paid Play Store app (Android first, iOS later). No cloud infra beyond public map tiles.

## Non-goals

- Element fork or heavy coupling
- Self-hosted maps/tiles
- Pantry integration (separate app)
- Subscriptions

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend/Core | matrix-rust-sdk via FFI — protocol, E2EE, sync, rooms, location events |
| Frontend | Flutter (single codebase for future iOS). Dark cyberpunk/"Shadowrun-like" theme |
| Maps | MapLibre + Protomaps public vector tiles + custom style JSON |
| Storage | matrix-rust-sdk built-in persistence (cache/sync state). Minimal local preferences |
| Background | Rust for location, battery optimization |
| Build/Release | Flutter Android → Play Store paid |

## Architecture Principles

- Clean separation: Rust core independent of UI
- Local-first where possible
- Battery and permission discipline
- Decoupled from upstream clients

## Repository Strategy

- **Public repo** (`shadowlink-rust-core`): Rust bridge + matrix-rust-sdk integration, generic helpers. MIT/Apache 2.0 license.
- **Private repo** (`shadowlink-app`): Full Flutter app, custom UI/theme, family flows, map styling, paid features. Proprietary.
- **No single repo with mixed licenses.**

## Key Features (MVP Order)

1. Homeserver configuration (user-provided URL)
2. Family room join/invite
3. E2EE chat + picture/media sharing
4. Location sharing: static + live updates
5. Custom map view with Shadowrun-style rendering
6. Basic offline caching (via SDK)
7. Settings: homeserver, map tiles, theme, battery options

## Monetization

- One-time purchase ($3–8)
- Optional paid unlocks for premium themes/styles or version bumps

## Project Structure (Private App Repo)

```text
shadowlink-app
├── rust/                  # git submodule or dependency to public crate
├── lib/                   # Flutter
│   ├── core/              # services, models
│   ├── features/
│   │   ├── chat/
│   │   ├── location/
│   │   └── map/
│   ├── ui/                # themes, widgets
│   └── main.dart
├── assets/
├── pubspec.yaml
└── PROJECT_MANIFEST.md
```

## License

- **Rust core** (this repo): OSS (MIT/Apache 2.0)
- **Full app** (private repo): Closed source / proprietary

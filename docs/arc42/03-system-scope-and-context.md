---
title: "3. System Scope & Context"
---

# 3. System Scope & Context

## 3.1 Business Context Diagram

```mermaid
graph TB
    subgraph External["External Systems"]
        HS["Matrix Homeserver\n(User-Provided URL)\nSynapse / Dendrite / Conduit"]
        MapTiles["MapLibre Tile Server\n(Public, Read-Only)"]
    end

    subgraph Consumer["Consumer Applications"]
        CLI["ShadowLink CLI\n(Rust binary)"]
        Flutter["Flutter App\n(Dart, UI layer)"]
    end

    subgraph System["System Under Design"]
        RustCore["ShadowLink Rust Core\n(MIT/Apache 2.0)\nFFI Bridge + Matrix Protocol"]
    end

    CLI -->|"Rust Dependency"| RustCore
    Flutter -->|"FFI Calls\n(C-ABI)"| RustCore
    RustCore -->|"Matrix Protocol\n(HTTPS + WSS)"| HS
    Flutter -->|"Tile Requests\n(HTTPS)"| MapTiles
```

## 3.2 External Interfaces

| System | Interface | Protocol | Direction |
|--------|-----------|----------|-----------|
| **Matrix Homeserver** | Matrix Client-Server API | HTTPS (REST) + WSS (Sync) | Rust Core → Homeserver |
| **Flutter App** | C-ABI FFI boundary | `extern "C"` function calls | Flutter → Rust Core |
| **ShadowLink CLI** | Rust crate dependency | `use shadowlink_rust_core` | CLI → Rust Core |
| **MapLibre Tiles** | Tile JSON + PBF | HTTPS | Flutter → Tile CDN |

The MapLibre tile interface is **outside** the Rust Core scope. The Flutter app consumes tiles
directly; the Rust core has no map rendering or tile fetching responsibilities.

## 3.3 System Scope

### In Scope ✅

- Matrix client lifecycle (login, session, logout)
- Room discovery, creation, join, invite, leave
- End-to-end encrypted messaging (text, images, media)
- Location event publishing and subscription
- Sync loop management with battery-aware scheduling
- E2EE key management and device verification
- FFI API surface design, error model, and memory contracts
- SpecKit behavioral specifications and automated test verification

### Out of Scope ❌

- User interface rendering (Flutter responsibility)
- Map display, tile fetching, geocoding (Flutter responsibility)
- App navigation flows, onboarding screens (Flutter responsibility)
- Theme/styling (Flutter responsibility)
- Push notification delivery (Flutter + platform responsibility)
- Homeserver administration or provisioning
- User account registration UX (protocol operations only)

## 3.4 Data Flow Summary

```mermaid
sequenceDiagram
    participant Flutter as Flutter App
    participant Core as Rust Core
    participant HS as Matrix Homeserver

    Flutter->>Core: FFI: send_message(room, text)
    Core->>HS: PUT /_matrix/client/v3/rooms/{id}/send/m.room.message
    HS-->>Core: event_id
    Core-->>Flutter: Ok(SendResult { event_id })

    HS->>Core: Sync: m.room.message (from other user)
    Core->>Core: Decrypt (Megolm)
    Core-->>Flutter: Callback: on_message(room, sender, decrypted_text)
```

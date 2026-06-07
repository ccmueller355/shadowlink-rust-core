---
title: "9. Architecture Decisions"
---

# 9. Architecture Decisions 🔨

> **Status:** Architecture Decision Records (ADRs) will be authored during the SpecKit Plan
> phase. This section serves as the ADR index and template.

## 9.1 ADR Format

Each Architecture Decision Record follows a consistent structure:

```markdown
### ADR-NNN: Title

**Status:** Proposed | Accepted | Deprecated | Superseded
**Date:** YYYY-MM-DD
**Deciders:** [names]

#### Context
What is the issue we're addressing? Why does a decision need to be made?

#### Decision
What is the decision? What option was chosen?

#### Consequences
What becomes easier or harder because of this decision?

#### Alternatives Considered
What other options were evaluated and why were they rejected?
```

## 9.2 Decision Backlog

The following ADRs have been identified and will be authored during the Plan phase:

| ID | Title | Status |
|----|-------|--------|
| **ADR-001** | FFI Bridge Approach | ✅ Accepted |
| **ADR-002** | Async Runtime Configuration | ✅ Accepted |
| **ADR-003** | matrix-rust-sdk Version Strategy | ✅ Accepted |
| **ADR-004** | Error Model Design | ✅ Accepted |
| **ADR-005** | Media Pipeline — SDK Delegation | ✅ Accepted |
| **ADR-006** | Location Event Format | ✅ Accepted |
| **ADR-007** | Logging & Observability | 📋 Proposed (deferred) |
| **ADR-008** | Cross-Platform Build Toolchain | 📋 Proposed (deferred) |

## 9.3 Decision Log

No ADRs have been formally accepted yet. The log will be populated chronologically as decisions
are made during the SpecKit Plan and implementation phases.

The following ADRs were accepted during the SpecKit Plan phase (2026-06-07):

- **ADR-001:** flutter_rust_bridge v2 selected as FFI bridge (Dart codegen, async, zero-copy).
- **ADR-002:** tokio selected as async runtime (matrix-rust-sdk requirement).
- **ADR-003:** matrix-sdk 0.7+ with e2e-encryption, sqlite, bundled-sqlite features.
- **ADR-004:** thiserror + ShadowLinkError enum for unified error taxonomy.
- **ADR-005:** Media pipeline fully delegated to SDK's send_attachment() API.
- **ADR-006:** Custom `org.shadowlink.location` event type over unstable MSC3488.
- **ADR-007:** Logging deferred to implementation phase (tracing + PII filtering).
- **ADR-008:** Cross-platform build toolchain deferred to Phase 6+ (Linux-first).

Full ADR documents: [`specs/001-shadowlink-core/research.md`](../../specs/001-shadowlink-core/research.md)

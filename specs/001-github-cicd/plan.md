# Implementation Plan: GitHub CI/CD Pipeline

**Branch**: `001-github-cicd` | **Date**: 2026-06-13 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-github-cicd/spec.md`

## Summary

Set up GitHub Actions so that (1) every push/PR to `main` is built and tested on
both macOS and Windows with formatting/lint gates, and (2) pushing a `v*` tag
produces unsigned macOS (.dmg) and Windows (.msi/.exe) installers attached to a
GitHub Release — with zero signing steps, zero secrets required for fork PRs,
and no data sent anywhere except GitHub itself. Because no application code
exists yet, this feature also scaffolds the minimal repository skeleton
(Cargo workspace with `pinshot-core`, Tauri 2.x shell, TypeScript frontend)
that the pipeline builds — the skeleton follows the clean-architecture layout
mandated by Constitution Principle IV so later features land in the right place.

## Technical Context

**Language/Version**: Rust (stable, pinned via `rust-toolchain.toml`), TypeScript (Node 20 LTS)

**Primary Dependencies**: Tauri 2.x, GitHub Actions (`actions/checkout`, `dtolnay/rust-toolchain`, `Swatinem/rust-cache`, `actions/setup-node`, `tauri-apps/tauri-action`)

**Storage**: N/A (pipeline state lives in GitHub; caches are managed by Actions cache)

**Testing**: `cargo test --workspace` (core crate testable headless), frontend `npm test` placeholder; pipeline verified by deliberately-broken test commits

**Target Platform**: GitHub-hosted runners — `macos-latest` (arm64) and `windows-latest`

**Project Type**: Desktop app (Tauri) + CI/CD infrastructure

**Performance Goals**: PR pass/fail feedback ≤ 15 min (SC-002); tag → downloadable installers ≤ 30 min (SC-003)

**Constraints**: No code signing (no certificates exist — FR-015); fork PRs must pass without secrets (FR-011); no egress beyond GitHub (FR-012); no broken/partial releases (FR-010)

**Scale/Scope**: 2 workflows, 2 platform targets, solo maintainer + external contributors

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.* — **Constitution v1.1.0**

| Principle | Gate | Status |
|---|---|---|
| I. Privacy-First & Offline-Only | Pipeline talks only to GitHub (checkout, cache, releases). No third-party upload/telemetry actions. Third-party actions are pinned and reviewable. | ✅ PASS |
| II. Performance Is a Feature | CI keeps feedback ≤ 15 min via dependency caching and concurrency cancellation; no impact on app runtime. | ✅ PASS |
| III. Cross-Platform Parity | Build+test matrix runs macOS and Windows on every PR; either platform failing fails the run. This principle is *implemented by* this feature. | ✅ PASS |
| IV. Clean Architecture: Core as a Library | Scaffold creates `crates/pinshot-core` (headless, testable) separate from `src-tauri/` shell and `ui/` frontend; CI tests the core without the GUI. | ✅ PASS |
| V. Strict Scope & Simplicity | Two small workflows; no package managers (v0.2), no signing (no certs), no Linux. Plain matrix builds — no reusable-workflow framework. | ✅ PASS |
| VI. Maintainability & Contributor Experience | Same commands locally and in CI (`cargo fmt/clippy/test`, `npm run lint/build`); style enforced by tools; pipeline fully version-controlled and commented. | ✅ PASS |

**No violations — Complexity Tracking is empty.**

## Project Structure

### Documentation (this feature)

```text
specs/001-github-cicd/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0: decisions & alternatives
├── quickstart.md        # Phase 1: how to validate the pipeline locally & on GitHub
├── checklists/
│   └── requirements.md  # Spec quality checklist (passed)
└── tasks.md             # Phase 2 (/speckit.tasks output)
```

`data-model.md` and `contracts/` are **omitted**: the feature has no persistent
data model or API contracts — its "entities" (run, artifact, release) are
GitHub-native concepts.

### Source Code (repository root)

```text
.github/
└── workflows/
    ├── ci.yml               # push/PR → build + test + quality gates (US1, US3)
    └── release.yml          # v* tag → installers attached to GitHub Release (US2)

crates/
└── pinshot-core/            # Rust core library (Principle IV: headless, testable)
    ├── Cargo.toml
    └── src/lib.rs           # placeholder domain module + unit test

src-tauri/                   # Tauri 2.x shell (commands, tray, windows)
    ├── Cargo.toml           # depends on pinshot-core
    ├── tauri.conf.json      # bundle: dmg (macOS), msi + nsis (Windows)
    ├── capabilities/
    └── src/main.rs

ui/                          # TypeScript frontend (webview)
    ├── package.json         # scripts: dev, build, lint, format:check, test
    ├── src/
    └── index.html

Cargo.toml                   # workspace: crates/pinshot-core, src-tauri
rust-toolchain.toml          # pinned stable toolchain (same in CI and locally)
rustfmt.toml                 # style enforced by tools, not review debate
README.md                    # build/run/test instructions + unsigned-build notice
CONTRIBUTING.md              # how CI gates PRs, how to run checks locally
```

**Structure Decision**: Cargo workspace with the core split out per Constitution
Principle IV. CI builds/tests the workspace and the `ui/` package; the release
workflow drives `tauri-apps/tauri-action`, which builds the same workspace and
bundles installers. The skeleton is intentionally minimal — just enough for the
pipeline to compile, test, and bundle a window that opens.

## Workflow Design

### `ci.yml` — every push to `main` + every PR targeting `main` (US1 + US3)

- **Jobs**:
  - `quality` (ubuntu-latest, cheapest): `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `npm ci && npm run lint && npm run format:check` in `ui/`.
  - `build-test` (matrix: `macos-latest`, `windows-latest`): install Tauri OS deps, `npm ci && npm run build` (frontend), `cargo build --workspace`, `cargo test --workspace`.
  - `ci-ok` (aggregator): `needs: [quality, build-test]`, fails if any dependency failed → the **single** required status check for branch protection (FR-004/FR-005).
- **Triggers**: `push: branches: [main]`, `pull_request: branches: [main]` — plain `pull_request` (never `pull_request_target`), so fork PRs run without secrets (FR-011).
- **Hygiene**: `concurrency` group per ref with `cancel-in-progress: true` (supersede stale runs); `timeout-minutes` on every job (no hung runs); `Swatinem/rust-cache` + npm cache keyed by lockfiles (FR-014); `workflow_dispatch` for manual re-runs (FR-016); `permissions: contents: read`.

### `release.yml` — on `v*` tag push (US2)

- **Job `build-release`** (matrix: macOS, Windows): `tauri-apps/tauri-action` builds installers and uploads them to a **draft** GitHub Release for the tag (`releaseDraft: true`).
- **Job `publish-release`**: `needs: build-release` — runs only when **both** platforms succeeded; flips the draft to published. A one-platform failure leaves only a draft (invisible to users) → no broken/partial public release (FR-010). Re-pushed tags re-use the same draft deterministically.
- **Permissions**: `contents: write` only in this workflow (uses the built-in `GITHUB_TOKEN`, no custom secrets).
- **No signing**: zero signing/notarization steps and zero signing secrets (FR-015). A clearly marked `# SIGNING EXTENSION POINT` comment documents where signing env vars would go if certificates are ever obtained. Artifacts are unsigned; README documents Gatekeeper/SmartScreen bypass.

## Complexity Tracking

*No constitution violations — table intentionally empty.*

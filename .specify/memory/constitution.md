<!--
Sync Impact Report
- Version change: 1.0.0 → 1.1.0
- Modified principles:
  - IV. Core as a Library → IV. Clean Architecture: Core as a Library
    (expanded with explicit layering and dependency rule)
- Added sections:
  - VI. Maintainability & Contributor Experience (new principle)
- Removed sections: none
- Other changes:
  - Additional Constraints → Distribution: replaced "unsigned until v1.0 signing"
    with explicit NO-SIGNING rule — no certificates exist; CI MUST NOT contain
    active signing steps or require signing secrets.
- Templates checked:
  - ✅ .specify/templates/plan-template.md — generic "Constitution Check" gate; no change needed
  - ✅ .specify/templates/spec-template.md — no constitution-specific references; no change needed
  - ✅ .specify/templates/tasks-template.md — no constitution-specific references; no change needed
  - ✅ .kiro/prompts/speckit.*.md — generic; no change needed
  - ✅ specs/001-github-cicd/spec.md — FR-015 & Assumptions aligned with no-signing rule
- Follow-up TODOs: none
-->

# PinShot Constitution

## Core Principles

### I. Privacy-First & Offline-Only (NON-NEGOTIABLE)

The application core MUST NOT make any network request, ever. Zero telemetry, zero
analytics, zero accounts, zero cloud features. Screenshots frequently contain highly
sensitive data (chats, credentials, internal dashboards); they MUST never leave the
user's device without an explicit, user-initiated action.

- No network dependency is permitted in the core library or its dependency tree.
- The optional update check MUST be isolated from the core, opt-in, and default OFF;
  it may only fetch a static version file.
- History and temporary files MUST be stored locally, with a working "clear all"
  control; optional local encryption is allowed, remote storage is not.
- Any feature that would require an online API (including AI services) is rejected
  by default; this principle supersedes feature requests.

**Rationale**: Privacy is the project's primary differentiator versus closed-source
and cloud-pushing competitors, and the one advantage they cannot copy without
changing their business model.

### II. Performance Is a Feature

Speed is feature #1. The app lives in the tray 24/7 and MUST stay lightweight.

- Hotkey → capture overlay visible in under 100ms (cold), measured on baseline
  hardware; this is a release gate, not an aspiration.
- Idle CPU usage MUST be effectively zero; memory footprint MUST remain suitable
  for an always-running background app.
- Performance-sensitive paths (capture, image ops, future OCR) MUST be implemented
  in Rust; the webview is for UI only.
- Changes that regress capture latency or idle resource use MUST be justified and
  approved explicitly before merge.

### III. Cross-Platform Parity

macOS and Windows are equal first-class targets. A feature is not "done" until it
works on both.

- Every change MUST build and pass tests on both macOS and Windows in CI before
  merging to the default branch; a single-platform pass is a failure.
- Multi-monitor and mixed-DPI correctness are mandatory test cases for any change
  touching capture, overlay, or pin positioning (the project's highest-risk area).
- Platform-specific code MUST be isolated behind a common interface so the shared
  core stays portable (Linux is a future target, not a current one).

### IV. Clean Architecture: Core as a Library

Core logic (capture, image operations, future OCR) MUST live in a Rust library
separate from the Tauri shell, with dependencies pointing inward only.

- **Layers (outer → inner)**: UI (TypeScript webview) → Tauri shell (commands,
  tray, hotkeys, windows) → core library (domain logic) → platform adapters
  (OS-specific capture/clipboard/OCR behind traits). An inner layer MUST NOT
  depend on or know about an outer layer.
- The core library MUST be independently buildable and testable without launching
  the GUI shell (`cargo test -p pinshot-core` works headless).
- Platform-specific code MUST sit behind a trait/interface in the core, with
  `#[cfg]`-gated implementations — never scattered through business logic.
- The UI layer MUST contain no business logic that belongs in the core; it
  renders state and forwards intents over a thin, explicit IPC surface.
- New features start by defining the core-library API surface, then the shell/UI
  integration — in that order.

**Rationale**: Enables fast automated testing, keeps a future Linux port feasible,
and makes the privacy guarantee auditable in one place.

### V. Strict Scope & Simplicity

Scope discipline protects a solo-maintainer project from burnout and bloat.

- The documented non-goals are binding: no cloud upload/sharing, no accounts or
  sync, no telemetry, no screen recording before v1.0, no online-API AI features.
- Roadmap order (capture/pin → daily-driver polish → OCR → beautify) MUST be
  respected; work on a later phase MUST NOT block or destabilize an earlier one.
- Prefer the simplest design that satisfies the spec (YAGNI); added complexity
  MUST be justified in the plan's Complexity Tracking section.

### VI. Maintainability & Contributor Experience

The project MUST stay easy for a newcomer to understand, build, and modify —
it is designed to outlive any single maintainer.

- A contributor MUST be able to clone, build, and run the full test suite with
  the standard toolchain commands documented in the README — no undocumented
  setup steps, no machine-specific configuration.
- Module boundaries follow Principle IV; each crate/package MUST have a stated
  single responsibility, and public APIs MUST have doc comments explaining
  intent (the *why*, not a restatement of the code).
- Code style is enforced by tools, not by review debate: `rustfmt` + `clippy`
  for Rust, the configured formatter/linter for the frontend; CI fails on
  violations, so humans never argue about style.
- Names, code, and code comments are in English; identifiers describe domain
  concepts (capture, pin, annotation), not implementation trivia.
- Every non-trivial decision lives in a reviewable artifact (spec, plan, or
  ADR-style note in `specs/`) rather than in one person's memory.

**Rationale**: A solo-maintainer open-source project survives only if outside
contributors can become productive quickly and nothing important is tribal
knowledge.

## Additional Constraints

- **Stack**: Rust core, Tauri 2.x shell, TypeScript (Svelte/SolidJS) UI, with the
  vetted crates documented in `about.md` (xcap, global-hotkey, tray-icon, image).
  Introducing a new runtime dependency requires checking license compatibility and
  the no-network rule (Principle I).
- **Settings & data**: persisted as local, human-readable TOML/JSON files —
  transparent and easy to back up.
- **Distribution**: GitHub Releases first (alpha), then Homebrew Cask / winget /
  Scoop. **No code signing**: the project currently has no Apple Developer or
  Windows signing certificate. All builds are unsigned; CI/CD MUST NOT contain
  active signing/notarization steps and MUST NOT require signing secrets to
  succeed. Gatekeeper/SmartScreen workarounds MUST be documented for users.
  Signing is revisited only if/when certificates are actually obtained.
- **License**: open source (GPL-3.0 vs MIT/Apache-2.0 decision is REQUIRED before
  the first public release).

## Development Workflow

- **Spec-driven**: every feature follows the Spec Kit cycle — constitution →
  `/speckit.specify` → `/speckit.plan` → `/speckit.tasks` → `/speckit.implement` —
  with artifacts stored under `specs/<NNN-feature>/`.
- **Quality gates**: CI MUST run build, tests, formatting, and lint for both the
  Rust core and the web frontend on macOS and Windows for every PR (see
  `specs/001-github-cicd/`). Failing any gate blocks merge.
- **Testability**: acceptance scenarios in specs MUST be independently testable;
  core-library changes ship with automated tests.
- **Community**: contributions are welcomed from the start — public roadmap,
  CONTRIBUTING.md, good-first-issue labels; English for code/docs, project docs
  may also be maintained in Indonesian.

## Governance

This constitution supersedes all other development practices for PinShot. Anyone
proposing a change that conflicts with a principle MUST amend the constitution
first, not work around it.

- **Amendments**: proposed via pull request that updates this file, including a
  Sync Impact Report and any required template updates; approved by the
  maintainer.
- **Versioning**: semantic versioning of this document — MAJOR for removing or
  redefining a principle, MINOR for adding a principle or materially expanding
  guidance, PATCH for clarifications.
- **Compliance**: every `/speckit.plan` MUST pass the Constitution Check gate
  against the principles above before Phase 0 research; violations require an
  entry in Complexity Tracking with a justification, or the design is revised.
- **Guidance**: `about.md` is the living product/roadmap document; this
  constitution governs *how* that roadmap is built.

**Version**: 1.1.0 | **Ratified**: 2026-06-13 | **Last Amended**: 2026-06-13

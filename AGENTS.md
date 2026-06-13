<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan:
`specs/002-capture-to-output/plan.md`

## Project snapshot

- **Product**: PinShot — open-source, 100% offline screenshot & pin tool
  (Snipaste-class) for macOS & Windows. See `about.md` (roadmap) and
  `.specify/memory/constitution.md` v1.1.0 (binding principles).
- **Stack**: Rust core library (`crates/pinshot-core`) + Tauri 2.x shell
  (`src-tauri/`) + TypeScript frontend (`ui/`). Clean architecture: dependencies
  point inward (UI → shell → core → platform adapters behind traits).
- **Hard rules**: zero network in core, zero telemetry, no accounts; no code
  signing anywhere (no certificates exist — CI must never require signing
  secrets); macOS + Windows parity; core testable headless.

## Commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace && cargo test --workspace
cd ui && npm ci && npm run lint && npm run format:check && npm run build
```

## Spec-driven workflow

Active feature: `specs/002-capture-to-output/` (spec ✅, checklist ✅, plan ✅ +
research/data-model/contracts/quickstart — ready for `/speckit.tasks`). Done:
`specs/001-github-cicd/` (CI/CD pipeline, shipped; v0.0.1-alpha.1 released). New features: `/speckit.specify` → `/speckit.plan` →
`/speckit.tasks` → `/speckit.implement`; every plan must pass the Constitution
Check gate. `main` is branch-protected — all work lands via PR.
<!-- SPECKIT END -->

# Quickstart: Validating the CI/CD Pipeline

**Feature**: 001-github-cicd | **Date**: 2026-06-13

## Run the same checks CI runs (locally)

```bash
# Rust — formatting, lints, build, tests (workspace = pinshot-core + src-tauri)
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
cargo test --workspace

# Frontend
cd ui
npm ci
npm run lint
npm run format:check
npm run build
```

If all of the above pass locally, `ci.yml` will pass (same commands, same
pinned toolchain via `rust-toolchain.toml`).

## Verify the PR pipeline (US1 + US3)

1. Push a branch, open a PR against `main` → the **ci-ok** check appears.
2. Negative test: commit `assert!(false)` in a `pinshot-core` test → run must
   fail and the failing platform/job must be identifiable in the run logs.
3. Negative test: break formatting (`cargo fmt` violation) → `quality` job
   fails and names the file.
4. Fork test: open the same PR from a fork → all checks still run and report
   (no secrets needed).

## Verify the release pipeline (US2)

```bash
git tag v0.0.1-alpha.1
git push origin v0.0.1-alpha.1
```

Expected within ~30 minutes:

- A GitHub Release for the tag with a macOS `.dmg` and a Windows `.msi`/`.exe`
  attached — published only after **both** platform builds succeed.
- Negative test: tag a commit that fails on one platform → release stays a
  **draft** (invisible to users), failure visible in Actions.

> Artifacts are **unsigned** (no certificates exist). macOS: right-click →
> Open, or `xattr -d com.apple.quarantine PinShot.app`. Windows: SmartScreen
> → "More info" → "Run anyway". These instructions belong in the README.

## Branch protection (one-time manual setup on GitHub)

Settings → Branches → protect `main` → require status check **ci-ok**.
This enforces SC-001/SC-004 (nothing merges without both platforms passing).

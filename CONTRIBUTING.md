# Contributing to PinShot

Thanks for your interest! PinShot is built spec-first and aims to stay easy for
newcomers to understand and modify. Please skim the
[constitution](.specify/memory/constitution.md) — it lists the binding
principles (privacy-first, offline-only, cross-platform parity, clean
architecture, no code signing).

## Development workflow

PinShot uses [Spec Kit](https://github.com/github/spec-kit). Each feature lives
under `specs/<NNN-feature>/` and flows through:

```
/speckit.specify  →  /speckit.plan  →  /speckit.tasks  →  /speckit.implement
```

The active feature is [`specs/001-github-cicd`](specs/001-github-cicd/).

## Run the same checks CI runs (before you push)

CI runs exactly these commands; if they pass locally, CI will pass. The Rust
toolchain is pinned in `rust-toolchain.toml`, so you get the same compiler.

```bash
# Rust core + shell
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

Auto-fix style before committing: `cargo fmt --all` and `npm --prefix ui run format`.

## How CI gates your pull request

Opening a PR against `main` triggers `.github/workflows/ci.yml`:

- **`quality`** — `cargo fmt --check`, `cargo clippy -D warnings`, and the
  frontend `lint` + `format:check`.
- **`build-test`** — builds and runs `cargo test --workspace` on **both**
  macOS and Windows. A failure on either platform fails the PR.
- **`ci-ok`** — a single aggregated status; this is the required check for
  merging. PRs from forks run fully without needing any secrets.

Releases are produced by `.github/workflows/release.yml` when a `v*` tag is
pushed (unsigned installers attached to a GitHub Release).

## Conventions

- Code, identifiers, and comments are in **English**.
- Keep platform-specific code behind a trait in `pinshot-core`; don't scatter
  `#[cfg]` through domain logic.
- Public APIs get doc comments explaining intent (the *why*).
- Commit messages: short imperative subject (e.g. `core: normalise selection rect`).

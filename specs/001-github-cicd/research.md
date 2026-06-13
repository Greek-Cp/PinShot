# Phase 0 Research: GitHub CI/CD Pipeline

**Feature**: 001-github-cicd | **Date**: 2026-06-13

Each decision below resolves an unknown from the Technical Context or an
edge case from the spec.

## D1 — Two workflows, not one

**Decision**: Separate `ci.yml` (push/PR verification) and `release.yml` (tag → installers).

**Rationale**: PR runs need `contents: read` and must be fork-safe; release runs need `contents: write`. Splitting keeps permissions minimal per workflow (FR-012) and each file small and readable (Principle VI).

**Alternatives rejected**: Single workflow with `if:` conditionals — harder to review, broader permissions on every run.

## D2 — `tauri-action` for releases, plain cargo/npm for CI

**Decision**: PR verification runs `cargo build/test` + frontend build directly; only `release.yml` uses `tauri-apps/tauri-action` to produce bundles.

**Rationale**: Bundling installers on every PR wastes minutes (SC-002 ≤ 15 min); compile + test catches platform breakage. `tauri-action` is the maintained official path for dmg/msi/nsis bundling and release upload, keeping `release.yml` short.

**Alternatives rejected**: Hand-rolled `tauri build` + `gh release upload` scripts — more code to maintain; bundling in PR CI — too slow.

## D3 — Release safety via draft-then-publish

**Decision**: Matrix jobs upload installers to a **draft** release; a final `publish-release` job (`needs:` all platforms) publishes it.

**Rationale**: Directly satisfies FR-010 — users can never see a release missing one platform. Draft re-use also makes re-pushed tags deterministic (edge case: duplicate tags).

**Alternatives rejected**: Publish-per-platform (window where release is half-complete); build-artifacts-then-single-upload-job (more workflow plumbing for the same guarantee).

## D4 — Fork-PR safety via plain `pull_request` trigger

**Decision**: Use `pull_request` (never `pull_request_target`); CI jobs need no secrets at all.

**Rationale**: FR-011/SC-007 — fork PRs get full build/test/quality results. `pull_request_target` + checkout of PR code is a known privilege-escalation foot-gun.

**Alternatives rejected**: `pull_request_target` with guards — risk without benefit since nothing in CI needs secrets.

## D5 — Caching

**Decision**: `Swatinem/rust-cache` for cargo, `actions/setup-node` built-in npm cache, keyed by `Cargo.lock` / `package-lock.json`.

**Rationale**: FR-014 + SC-002. Both are the de-facto standard, content-keyed (stale lockfile ⇒ new key, so no corrupted reuse), and read-only for fork PRs by Actions' cache isolation rules.

**Alternatives rejected**: Manual `actions/cache` paths for cargo — replicates what rust-cache already does well; no cache — routine PR feedback would blow the 15-minute target.

## D6 — Single required status check (`ci-ok` aggregator job)

**Decision**: An aggregator job `needs:` all CI jobs and fails if any failed; branch protection requires only `ci-ok`.

**Rationale**: FR-004/FR-005 — one unambiguous overall status; matrix job names can change without re-configuring branch protection. Per-platform results stay visible in the run for diagnosis (FR-007).

**Alternatives rejected**: Requiring each matrix job in branch protection — brittle when the matrix changes.

## D7 — Hygiene: concurrency, timeouts, manual trigger

**Decision**: `concurrency: { group: <workflow>-<ref>, cancel-in-progress: true }` on CI; `timeout-minutes` on every job (CI: 30, release: 45); `workflow_dispatch` on CI.

**Rationale**: Edge cases — concurrent pushes supersede stale runs; hung builds fail visibly instead of blocking; FR-016 manual re-run without a code change. Release workflow does **not** cancel in progress (a tag build should finish).

## D8 — No signing, with a documented extension point

**Decision**: Zero signing/notarization steps; a `# SIGNING EXTENSION POINT` comment block in `release.yml` notes which env vars (`APPLE_CERTIFICATE…` / `WINDOWS_CERTIFICATE…`) tauri-action would consume if certificates are ever obtained.

**Rationale**: FR-015 and Constitution v1.1.0 Distribution rule — the maintainer has no certificates; the pipeline must never fail for lack of signing secrets. A comment costs nothing and prevents future restructuring.

**Alternatives rejected**: Conditional signing steps gated on secret presence — dead code paths that can't be tested today.

## D9 — Toolchain pinning

**Decision**: `rust-toolchain.toml` pins the stable Rust version; Node 20 LTS pinned in workflows and documented in README.

**Rationale**: Same toolchain locally and in CI (Principle VI — reproducible contributor experience); avoids surprise breakage when runners bump default toolchains.

## D10 — Minimal app skeleton is in scope

**Decision**: Scaffold the Cargo workspace (`crates/pinshot-core`), Tauri 2.x shell (`src-tauri/`), and `ui/` frontend as part of this feature's setup phase.

**Rationale**: The repo currently has no code; a pipeline with nothing to build cannot satisfy any acceptance scenario. The skeleton is the smallest thing CI can compile, test, and bundle, laid out per Constitution Principle IV so feature work (002+) lands in the right structure.

**Alternatives rejected**: CI-first with a dummy "hello world" outside the final layout — would be restructured immediately by the next feature, violating Simplicity.

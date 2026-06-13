# Tasks: GitHub CI/CD Pipeline

**Input**: Design documents from `/specs/001-github-cicd/`

**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, quickstart.md ✅

**Tests**: Pipeline behavior is verified with the negative-test procedure in
quickstart.md (deliberately broken commits), not with a separate test suite.

**Organization**: Tasks are grouped by user story so each story is
independently implementable and testable.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: US1 = dual-platform build+test, US2 = release artifacts, US3 = quality gates

## Phase 1: Setup (Repository Skeleton)

**Purpose**: Create the minimal buildable app the pipeline compiles, tests, and
bundles — laid out per Constitution Principle IV (core as a library).

- [x] T001 Create Cargo workspace root `Cargo.toml` (members: `crates/pinshot-core`, `src-tauri`) and `rust-toolchain.toml` pinning stable Rust
- [x] T002 [P] Create `crates/pinshot-core/` library crate: `Cargo.toml` + `src/lib.rs` with a placeholder domain module and at least one passing unit test (proves headless `cargo test -p pinshot-core`)
- [x] T003 Scaffold Tauri 2.x shell in `src-tauri/` (`Cargo.toml` depending on `pinshot-core`, `src/main.rs`, `tauri.conf.json` with bundle targets `dmg` + `msi`/`nsis`, `capabilities/`), wired to the `ui/` frontend
- [x] T004 [P] Scaffold `ui/` TypeScript frontend with `package.json` scripts `dev`, `build`, `lint`, `format:check` (ESLint + Prettier), minimal `index.html` + `src/main.ts`
- [x] T005 [P] Add `rustfmt.toml` and workspace clippy lint config so style is tool-enforced (Constitution VI)
- [x] T006 [P] Write `README.md` (clone → build → test instructions, unsigned-build Gatekeeper/SmartScreen notice) and `CONTRIBUTING.md` (CI gates, local check commands from quickstart.md)
- [x] T007 Verify the full local loop from quickstart.md passes: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`, `npm ci && npm run build` in `ui/`

**Checkpoint**: Repo builds and tests green locally on at least one platform.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared workflow infrastructure both user-facing workflows rely on.

- [x] T008 Create `.github/workflows/` and a reusable setup pattern documented as comments: checkout (`actions/checkout`), Rust toolchain (`dtolnay/rust-toolchain` reading `rust-toolchain.toml`), `Swatinem/rust-cache`, `actions/setup-node` (Node 20, npm cache on `ui/package-lock.json`), Tauri Linux deps not needed (macOS/Windows only)

**Checkpoint**: Setup steps proven by the first job that uses them (T009).

---

## Phase 3: User Story 1 — Every change built & tested on both platforms (P1) 🎯 MVP

**Goal**: Push/PR to `main` → automatic build + test on macOS and Windows with one unambiguous overall status.

**Independent Test**: PR with a deliberately failing test on one platform → overall status failing with the platform identifiable; clean PR → passing (quickstart.md §"Verify the PR pipeline").

- [x] T009 [US1] Create `.github/workflows/ci.yml` with `build-test` matrix job (`macos-latest`, `windows-latest`): frontend `npm ci && npm run build`, then `cargo build --workspace` and `cargo test --workspace`; `timeout-minutes: 30`; `permissions: contents: read`
- [x] T010 [US1] Add triggers `push: branches: [main]` + `pull_request: branches: [main]` (plain `pull_request` — fork-safe, FR-011), plus `workflow_dispatch` for manual re-runs (FR-016)
- [x] T011 [US1] Add `concurrency` group per ref with `cancel-in-progress: true` so stale runs are superseded (edge case: concurrent pushes)
- [x] T012 [US1] Add `ci-ok` aggregator job (`needs: [build-test]`, extended by US3) that fails if any needed job failed — the single required status (FR-004/FR-005)
- [ ] T013 [US1] Validate per quickstart.md: clean run passes on both platforms; broken-test run fails and names the platform; fork PR runs without secrets

**Checkpoint**: US1 deliverable — dual-platform safety net live; MVP of this feature.

---

## Phase 4: User Story 2 — Tag publishes installable artifacts (P2)

**Goal**: Pushing a `v*` tag yields a GitHub Release with unsigned macOS + Windows installers, no manual uploads, no partial releases.

**Independent Test**: Tag a known-good commit → Release for that tag has `.dmg` + `.msi`/`.exe` attached (quickstart.md §"Verify the release pipeline").

- [x] T014 [US2] Create `.github/workflows/release.yml` triggered on `v*` tag push: `build-release` matrix job (macOS, Windows) using `tauri-apps/tauri-action` with `releaseDraft: true`, uploading installers to a draft release; `timeout-minutes: 45`; `permissions: contents: write`; built-in `GITHUB_TOKEN` only
- [x] T015 [US2] Add `publish-release` job (`needs: build-release`) that publishes the draft only when both platforms succeeded (FR-010); draft re-use keeps re-pushed tags deterministic
- [x] T016 [US2] Add the `# SIGNING EXTENSION POINT` comment block documenting where signing env vars would go **without implementing any signing step** (FR-015 — no certificates exist)
- [ ] T017 [US2] Validate per quickstart.md: tag → published release with both installers; broken-platform tag → draft only, failure visible in Actions

**Checkpoint**: Alpha distribution path live (roadmap v0.1 "Rilis alpha").

---

## Phase 5: User Story 3 — PRs gated by code-quality checks (P3)

**Goal**: Formatting + lint verified for Rust core and frontend on every PR.

**Independent Test**: PR with a formatting violation → quality check fails and names the violation; clean PR → passes.

- [x] T018 [US3] Add `quality` job to `.github/workflows/ci.yml` on `ubuntu-latest`: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `npm ci && npm run lint && npm run format:check` in `ui/`; `timeout-minutes: 15`
- [x] T019 [US3] Wire `quality` into the `ci-ok` aggregator (`needs: [quality, build-test]`) so style violations block merge (FR-006)
- [ ] T020 [US3] Validate: PR with bad formatting fails `quality` pointing at the file; clean PR passes

**Checkpoint**: All three user stories independently functional.

---

## Phase 6: Polish & Cross-Cutting

- [x] T021 [P] Add CI status badge to `README.md`; document branch-protection setup (require `ci-ok`) per quickstart.md
- [x] T022 [P] Review both workflow files against FR-012/SC-006: every step's destination is GitHub-only; third-party actions pinned to a major version or SHA; note the audit result in PR description
- [ ] T023 Run the full quickstart.md validation end-to-end and tick off the spec's Success Criteria SC-001…SC-008

---

## Dependencies & Execution Order

- **Phase 1 → Phase 2 → user stories**: skeleton must build locally before any workflow can pass.
- **US1 (Phase 3)** blocks nothing but is the MVP — do first.
- **US2 (Phase 4)** depends on Phase 1 bundling config (`tauri.conf.json`), not on US1 jobs, but only trust a release after US1 is green.
- **US3 (Phase 5)** extends `ci.yml` from US1 (same file — not parallel with Phase 3).
- **Parallel opportunities**: T002/T004/T005/T006 (different files); T021/T022.

## Implementation Strategy

MVP first: Phases 1–3, validate, then 4, then 5. Each checkpoint is a working,
independently demonstrable increment. Commit after each task or logical group.

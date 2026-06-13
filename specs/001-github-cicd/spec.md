# Feature Specification: GitHub CI/CD Pipeline

**Feature Branch**: `001-github-cicd`

**Created**: 2026-06-13

**Status**: Draft

**Input**: User description: "GitHub CI/CD pipeline configuration for building, testing, and releasing the cross-platform Tauri app on macOS and Windows"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Every change is built and tested on both platforms (Priority: P1)

As a maintainer or contributor, when I push a commit or open a pull request, the project is automatically built and its automated tests are run on both macOS and Windows, so that platform-specific breakage is caught before it reaches the default branch.

**Why this priority**: PinShot is a cross-platform desktop tool whose hardest historical risks (multi-monitor, mixed-DPI capture, OS-specific behavior) only surface per-platform. A solo or small maintainer team cannot reliably build both operating systems locally on every change. Automated dual-platform verification is the single most valuable safety net and is the foundation every other CI capability builds on. It is the v0.1 "CI build macOS + Windows" milestone.

**Independent Test**: Open a pull request containing a change that compiles on one operating system but fails on the other; confirm the pull request is reported as failed and identifies which platform broke. Open a second pull request that is correct on both platforms and confirm it is reported as passing. This delivers value on its own even if no release automation exists yet.

**Acceptance Scenarios**:

1. **Given** a pull request that builds and passes tests on both macOS and Windows, **When** the pipeline finishes, **Then** the pull request shows a single clear "passing" status.
2. **Given** a pull request that fails to build or has a failing test on Windows only, **When** the pipeline finishes, **Then** the overall status is "failing" and the Windows result is identifiable.
3. **Given** a new push to an existing pull request, **When** the push lands, **Then** a fresh build-and-test run is triggered automatically without manual action.
4. **Given** a pull request opened from a fork without access to repository secrets, **When** the pipeline runs, **Then** build and test still complete and report a status.

---

### User Story 2 - Tagging a version publishes installable artifacts (Priority: P2)

As a maintainer, when I publish a version tag, the pipeline automatically builds installable artifacts for macOS and Windows and attaches them to a GitHub Release for that version, so that early adopters can download a working build without any manual packaging or upload step.

**Why this priority**: The v0.1 roadmap goal is an "alpha release for early adopters (GitHub Releases)". Without automated release artifacts, distribution depends on the maintainer manually building on two machines — slow, error-prone, and a blocker to the public alpha. This depends on P1 (a trustworthy build) but adds the distribution value.

**Independent Test**: Push a version tag to a commit that is known-good on both platforms; confirm a GitHub Release is created (or updated) for that tag with a downloadable macOS installer and a downloadable Windows installer attached, with no manual file upload performed.

**Acceptance Scenarios**:

1. **Given** a known-good commit, **When** a version tag is published, **Then** a GitHub Release for that tag contains a macOS installer and a Windows installer.
2. **Given** a tag whose build fails on one platform, **When** the release pipeline runs, **Then** no incomplete or broken release is presented as ready, and the failure is visible to the maintainer.
3. **Given** a published release, **When** a user opens the Releases page, **Then** they can download a build for their operating system without building from source.

---

### User Story 3 - Pull requests are gated by code-quality checks (Priority: P3)

As a contributor, when I open a pull request, automated formatting and linting checks run for both the Rust core and the web frontend, so that the codebase stays consistent and review focuses on substance rather than style.

**Why this priority**: An open-source project expecting external contributors ("accept contributors from the start") needs objective, automated quality gates so review effort scales. It is valuable but secondary to proving the code builds and runs correctly on both platforms.

**Independent Test**: Open a pull request that builds and passes tests but violates a formatting or lint rule; confirm the pull request is reported as failing the quality check and the specific violation is identifiable.

**Acceptance Scenarios**:

1. **Given** a pull request with incorrectly formatted code, **When** the pipeline runs, **Then** the quality check fails and points to the formatting violation.
2. **Given** a pull request that passes all formatting and lint rules, **When** the pipeline runs, **Then** the quality check passes.

---

### Edge Cases

- **Single-platform success**: A change builds on macOS but fails on Windows (or vice versa). The overall status MUST be "failing" — a one-platform success must never be reported as an overall pass.
- **Failed release build**: A version tag is published but the build fails on at least one platform. No partial, incomplete, or corrupt release may be presented to users as a finished download; the failure must be surfaced to the maintainer.
- **Fork pull requests**: A pull request from a fork cannot access repository secrets. Build, test, and quality checks must still run and report status; steps that genuinely require secrets (future signing/publishing) must be skipped or deferred rather than causing a misleading hard failure.
- **Duplicate or re-pushed tags**: The same version tag is published more than once. The outcome must be deterministic and must not yield duplicated or corrupted release artifacts.
- **Transient infrastructure failure**: A dependency download, toolchain install, or runner allocation fails intermittently. The run must fail visibly (not silently pass), and cached data must never be served in a corrupted state.
- **Hung or excessively long run**: A build or test hangs. The run must time out and report failure rather than block indefinitely.
- **Concurrent pushes**: Several pushes land on the same pull request in quick succession. Stale in-progress runs may be superseded so the reported status reflects the latest commit.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The pipeline MUST automatically start on every push to the default branch and on every pull request that targets the default branch.
- **FR-002**: The pipeline MUST build the application on both macOS and Windows within the same triggered run.
- **FR-003**: The pipeline MUST run the project's automated test suite (including the separately testable core library) on both macOS and Windows, and MUST fail the run when any test fails.
- **FR-004**: The pipeline MUST report a single, unambiguous overall pass/fail status to each pull request, visible before the change is merged.
- **FR-005**: A build or test failure on either platform MUST cause the overall run to be reported as failing (no silent single-platform pass).
- **FR-006**: The pipeline MUST verify code formatting and linting for both the application's core codebase and its web frontend codebase, and MUST fail when violations are found.
- **FR-007**: The pipeline MUST make build and test logs for a failed run available so a contributor can diagnose a failure on an operating system they do not have locally.
- **FR-008**: When a version tag is published, the pipeline MUST produce installable artifacts for macOS and for Windows.
- **FR-009**: The pipeline MUST attach the produced release artifacts to the GitHub Release corresponding to the published tag, with no manual upload step.
- **FR-010**: The pipeline MUST NOT publish a release as complete when the build or test stage for any target platform has failed for that tag.
- **FR-011**: The pipeline MUST complete build, test, and quality checks for pull requests originating from forks without requiring access to repository secrets.
- **FR-012**: The pipeline MUST NOT transmit repository source, build artifacts, or secrets to any destination other than the CI platform itself and explicitly configured, approved signing/distribution services — preserving the project's privacy-first, no-telemetry stance.
- **FR-013**: The entire pipeline definition MUST be stored in the repository under version control and be reviewable through the normal change-review process.
- **FR-014**: The pipeline MUST reduce repeated work between runs (e.g., reusing previously fetched dependencies) so routine feedback stays within the target time, without serving corrupted cached data.
- **FR-015**: The pipeline MUST NOT contain any active code-signing or notarization step and MUST NOT require signing certificates or signing secrets to succeed — the project has no signing certificates. The pipeline structure SHOULD leave a documented extension point so signing can be added later without restructuring, but no signing job is implemented.
- **FR-016**: The pipeline MUST support a way to re-run or manually trigger a verification run for a given commit without requiring an unrelated code change.

### Key Entities *(include if feature involves data)*

- **Pipeline Run**: A single execution triggered by a repository event (push, pull request, or tag). Has a triggering commit, a result per target platform, and one derived overall status.
- **Platform Target**: A supported operating system the project is built and tested against. Initial set: macOS and Windows. Each contributes an independent build/test result.
- **Build Artifact**: An installable output produced for a specific platform (a macOS installer and a Windows installer). Associated with the commit/tag that produced it.
- **Release**: A published version, identified by a version tag, with its set of downloadable build artifacts attached.
- **Quality Check**: A formatting, lint, or test result that contributes to a run's overall pass/fail status.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of pull requests targeting the default branch receive a build-and-test result for both macOS and Windows before they can be merged.
- **SC-002**: For a routine change, a contributor receives a pass/fail result within 15 minutes of pushing.
- **SC-003**: Publishing a version tag results in downloadable macOS and Windows installers attached to the corresponding release within 30 minutes, with zero manual file uploads.
- **SC-004**: 100% of changes merged into the default branch have passed build, test, and quality checks on both platforms.
- **SC-005**: A contributor who has access to only one operating system can get the other platform validated entirely by the pipeline, requiring zero local cross-platform builds to merge.
- **SC-006**: A reviewer can confirm, by reading the version-controlled pipeline definition alone, that no step sends repository contents or secrets anywhere other than the CI platform and explicitly approved signing/distribution endpoints.
- **SC-007**: Pull requests opened from forks complete build, test, and quality checks successfully without access to repository secrets.
- **SC-008**: When a tagged build fails on any platform, no broken or partial release is published — verified by attempting a release from a deliberately broken commit.

## Assumptions

- **Hosted runners**: macOS and Windows build environments are provided by the CI platform's hosted runners; no self-hosted build infrastructure is set up in the initial scope.
- **Initial scope boundary**: This feature covers build, test, code-quality gates, and publishing artifacts to GitHub Releases. Distribution through package managers (Homebrew Cask, winget, Scoop) is a later roadmap phase (v0.2) and is out of scope here.
- **No signing**: The maintainer has no Apple Developer or Windows code-signing certificate. All artifacts are unsigned, and the pipeline implements zero signing/notarization steps (FR-015) — only a documented extension point for the future. Gatekeeper/SmartScreen bypass instructions are a documentation concern, not a pipeline concern.
- **Default branch**: The default branch is `main`, and pull requests target it.
- **Versioning**: Releases are identified by semantic version tags (for example, `v0.1.0`).
- **Feedback-time targets**: The 15-minute pull-request and 30-minute release targets (SC-002, SC-003) are reasonable starting defaults and may be tuned as the codebase grows.
- **Test suite exists**: The core library and application have (or will add) an automated test suite that the pipeline invokes using the project's standard test command; CI does not define the tests themselves.
- **Frontend tooling**: The web frontend exposes build, format, and lint commands the pipeline can invoke.
- **Platform set**: macOS and Windows are the only initial targets; Linux is explicitly a later exploration (roadmap v1.0+) and out of scope.
- **No network in core**: Consistent with the privacy pillar, the application core has no network dependency; the pipeline's only external interactions are with the CI platform and (later, opt-in) signing/distribution endpoints.

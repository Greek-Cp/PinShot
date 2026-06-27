# Specification Quality Checklist: Floating Annotation Editor & Smart Capture Toolkit

**Purpose**: Validate specification completeness and quality before proceeding to `/speckit.tasks`
**Created**: 2026-06-27
**Feature**: [spec.md](../spec.md)

## Content Quality

- [x] No implementation details (languages, frameworks, APIs) in the spec itself
- [x] Focused on user value and product needs
- [x] Written so non-technical stakeholders can follow the user stories
- [x] All mandatory sections completed (User Scenarios, Requirements, Success Criteria, Assumptions)
- [x] Extra product sections (Vision, Personas, UX Flow, IA, Screen/Component specs, Shortcuts, Roadmap) are product-level, not implementation

## Requirement Completeness

- [x] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable (SC-001…SC-011)
- [x] Success criteria are technology-agnostic
- [x] All acceptance scenarios are defined (per user story US1–US6)
- [x] Edge cases are identified (mixed-DPI, undo branch, lossless blur, click-through trap, corrupt settings, etc.)
- [x] Scope is clearly bounded and sliced into independently-shippable priorities
- [x] Dependencies and assumptions identified (builds on 002 + 003; offline QR; OCR explicitly deferred)

## Constitution Alignment (v1.1.0)

- [x] **I. Privacy/Offline**: FR-046/SC-010 encode zero-network; FR-047 removes Share/upload/cloud; FR-029 confines QR Open URL to an explicit OS hand-off (OCR/Search/Translate deferred)
- [x] **II. Performance**: NFR-001…007 + SC-009 encode startup/capture/60fps/memory budgets
- [x] **III. Parity**: FR-049/SC-002 mandate macOS+Windows equivalence incl. mixed-DPI
- [x] **IV. Core as a Library**: FR-048 requires the annotation/flatten/effects/QR/color/encode/settings logic to be headless-testable in `pinshot-core`
- [x] **V. Strict Scope**: roadmap-ordered slices; Non-Goals reaffirm no share/sync/accounts/telemetry/screen-recording/online-AI
- [x] **VI. Maintainability**: requirements map 1:1 to acceptance criteria; module boundaries documented in plan.md

## Feature Readiness

- [x] All functional requirements have clear acceptance criteria
- [x] User scenarios cover primary flows (capture→annotate→output) and smart/shell flows
- [x] Feature meets measurable outcomes defined in Success Criteria
- [x] No implementation details leak into the specification
- [x] Planning artifacts present and consistent: [plan.md](../plan.md), [data-model.md](../data-model.md), [contracts/](../contracts/), [quickstart.md](../quickstart.md)

## Notes

- Items marked incomplete require spec updates before `/speckit.tasks`.
- Validation performed 2026-06-27; all items pass. See notes below.
- **Clarification session 2026-06-27** resolved 5 items and is recorded in the
  spec's `## Clarifications` section: editor always opens (Q1); **OCR removed/
  deferred** (Q2); QR kept (Q3); pins keep the editable doc (Q4); crop is
  non-destructive (Q5). All artifacts were updated accordingly and re-validated.

### Validation Notes

- **Source reconciliation (mockup vs. constitution)**: The design mockup
  (`image.png`) lists a **Share** action and "Pin / Copy / Save / Share". Per the
  product brief **and** Constitution Principle I, Share / cloud / upload /
  shareable-URL is **excluded** (FR-047, Non-Goals, SC-010). This is the one
  deliberate divergence from the mockup and is called out explicitly.
- **Implementation-detail check**: The spec names "macOS"/"Windows", "menu bar"/
  "system tray", "clipboard/file/pin", and "QR detection" as product facts. Specific
  crates (QR decoder, encoders), the preview-vs-flatten split, and
  the exact default hotkeys are deferred to [plan.md](../plan.md) and
  [data-model.md](../data-model.md).
- **Scope & roadmap**: This spec captures the Floating Editor vision for
  coherence but ships as prioritized, independently-testable slices — US1 (core
  editor + output) is a shippable MVP; US2 (contextual props + history), US3
  (tray + settings), US4 (QR detection), US5 (visual tools), and US6 (advanced pin)
  layer on top in roadmap order. **OCR was cut** from this feature (Clarification
  Q2) and maps to the public-launch milestone (v0.3); visual tools and advanced
  pin to polish (v0.5).
- **Offline integrity**: The only network-capable path is the explicit, opt-in,
  default-off *Check for Updates* (static version file), isolated in
  [app-shell-ipc.md](../contracts/app-shell-ipc.md). QR (`rqrr`) and color are
  fully on-device; QR **Open URL** is an explicit OS-browser hand-off.
- **Independent testability**: Each user story has an Independent Test and
  Given/When/Then acceptance scenarios; the headless `pinshot-core` tests cover
  the privacy-/DPI-critical flatten/effects/QR/color logic without a GUI.

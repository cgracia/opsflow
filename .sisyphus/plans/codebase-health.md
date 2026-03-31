# Codebase Health: CI, Unwrap Fix, README Sync

## TL;DR
> **Summary**: Three independent improvements to harden the Praxis codebase — add GitHub Actions CI, fix 2 production `unwrap()` calls in ccusage ingestion, and sync the README architecture section with reality.
> 
> **Deliverables**:
> - GitHub Actions CI workflow (fmt, build, test, clippy)
> - Safer error handling in ccusage processed-file move
> - Accurate README architecture section
> 
> **Estimated Effort**: Short
> **Parallel Execution**: YES — 1 implementation wave + 1 verification wave
> **Critical Path**: Tasks 1-3 in parallel → F1-F4 verification

---

## Context

### Original Request
After completing the dead-code cleanup, review overall codebase health and address findings.

### Health Scan Findings (corrected)
- **CI/CD**: No `.github/workflows/` exists — no automated safety net
- **unwrap()**: 219 total, but only 2 in production code (`src/ingest/ccusage.rs:93,95`). All others are in `#[cfg(test)]` blocks.
- **Phase 3 "incomplete"**: FALSE — TODO comments are in user-facing generated output (by design)
- **README**: Architecture section lists only 3 command files; actual codebase has 12+ commands and 6+ root modules

### Metis Review (gaps addressed)
- Scoped CI to validation-only (no release/publish automation)
- Chose best-effort semantics for ccusage move (warning on failure, no panic)
- Chose high-level module map for README (not exhaustive file tree — trees go stale)
- Added TDD approach for ccusage fix (test first, then fix)
- Locked scope: no repo-wide unwrap sweep, no docs restructure, no cross-platform matrix

---

## Work Objectives

### Core Objective
Add a CI safety net, eliminate the only 2 production panic risks, and fix the misleading README.

### Concrete Deliverables
- `.github/workflows/ci.yml` — runs on push/PR to master
- Updated `src/ingest/ccusage.rs` — no `unwrap()` in non-test code
- Updated `README.md` — architecture section reflects current modules

### Definition of Done
- [ ] `cargo fmt --all -- --check` exits 0
- [ ] `cargo build --locked` exits 0
- [ ] `cargo test --locked` exits 0
- [ ] `cargo clippy --locked --all-targets -- -D warnings` exits 0
- [ ] Zero `unwrap(` in non-test code of `src/ingest/ccusage.rs`
- [ ] README architecture section lists all current top-level modules and command files

### Must Have
- CI workflow triggers on push to `master` and pull_request
- CI uses stable Rust, Ubuntu latest
- CI runs: fmt check, build, test, clippy (all blocking)
- ccusage fix preserves existing best-effort move-to-processed behavior
- README lists every module that actually exists

### Must NOT Have (guardrails)
- No release/publish/crates.io automation
- No cross-platform CI matrix (DuckDB native dep makes this noisy)
- No repo-wide unwrap/expect cleanup beyond the 2 ccusage call sites
- No refactoring of adjacent ingest code or error types
- No broader docs restructure — only the architecture section in README
- No adding `ARCHITECTURE.md`, `CONTRIBUTING.md`, or other new doc files
- No exhaustive file tree in README — high-level module map only

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed.

- Test decision: **TDD** for the ccusage fix (test first, then fix)
- QA policy: Every task includes agent-executed QA scenarios
- Evidence: `.sisyphus/evidence/task-{N}-{slug}.{ext}`

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — all independent):
├── Task 1: Fix ccusage production unwraps (TDD) [deep]
├── Task 2: Add GitHub Actions CI workflow [quick]
└── Task 3: Sync README architecture section [quick]

Wave FINAL (After Wave 1 — verification):
├── F1: Plan compliance audit [oracle]
├── F2: Code quality review [unspecified-high]
├── F3: Real manual QA [unspecified-high]
└── F4: Scope fidelity check [deep]
→ Present results → Get explicit user okay

Critical Path: Task 1 → F1-F4 (or Task 2 → F1-F4, or Task 3 → F1-F4)
Parallel Speedup: ~65% faster than sequential
Max Concurrent: 3 (Wave 1)
```

### Dependency Matrix (full, all tasks)
| Task | Depends On | Blocks |
|---|---|---|
| 1 | none | F1, F2, F3, F4 |
| 2 | none | F1, F2, F3, F4 |
| 3 | none | F1, F2, F3, F4 |
| F1 | 1, 2, 3 | completion |
| F2 | 1, 2, 3 | completion |
| F3 | 1, 2, 3 | completion |
| F4 | 1, 2, 3 | completion |

### Agent Dispatch Summary
- Wave 1 → 3 tasks: T1 → `deep`, T2 → `quick`, T3 → `quick`
- Wave FINAL → 4 tasks: F1 → `oracle`, F2 → `unspecified-high`, F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [ ] 1. Fix ccusage production unwraps (TDD)

  **What to do**:
  - **RED first**: Add a test `test_ccusage_handles_missing_path_components` that calls the processed-file move logic with a path that has no parent (e.g., a bare filename `Path::new("file.json")`) and verifies: no panic, ingest still succeeds, a warning is emitted or the file is simply skipped.
  - **GREEN**: In `src/ingest/ccusage.rs`, replace the two production `unwrap()` calls:
    - Line 93: `path.parent().unwrap()` → use `if let Some(parent) = path.parent()` and handle the `None` case with a warning + early return (best-effort: skip the move, leave the file in place)
    - Line 95: `path.file_name().unwrap()` → similarly guard with `if let Some(name) = path.file_name()` and handle `None`
  - Follow the existing graceful-warning pattern used throughout `src/ingest/*.rs` — `eprintln!` or `println!` a warning and continue.
  - Run the full test suite to confirm nothing breaks.

  **Must NOT do**:
  - Do not change any test-code `unwrap()` calls
  - Do not refactor surrounding ingest logic or error types
  - Do not change the behavior on the happy path (file successfully moved to `processed/`)
  - Do not introduce new dependencies or error enums

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: TDD approach requires understanding existing ingest patterns and writing both test and fix
  - Skills: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 2, 3)
  - **Blocks**: F1, F2, F3, F4
  - **Blocked By**: none

  **References** (CRITICAL — executor has NO interview context):

  **Pattern References** (existing code to follow):
  - `src/ingest/ccusage.rs:85-99` — the exact block containing both `unwrap()` calls and the move-to-processed logic. This is the ONLY production code to modify.
  - `src/ingest/ccusage.rs:72-83` — the surrounding function context showing how `record_ingestion` and `path` are used before the move block
  - `src/ingest/claude_code.rs` — reference for graceful-warning patterns on file handling (search for `eprintln!` or `println!` warning patterns in ingest modules)
  - `src/ingest/opencode.rs` — another reference for how ingest modules handle file-level errors without panicking

  **Test References** (testing patterns to follow):
  - `src/ingest/ccusage.rs:274-300` — `test_ingest_moves_to_processed` — existing test for the happy-path move behavior. The new test should follow this pattern but test the edge case.
  - `src/ingest/ccusage.rs:237-269` — existing test module structure showing `TempDir` usage, fixture construction, and assertion patterns

  **API/Type References**:
  - `std::path::Path::parent()` — returns `Option<&Path>`, can be `None` for bare filenames
  - `std::path::Path::file_name()` — returns `Option<&OsStr>`, can be `None` for `/` or `..`

  **WHY Each Reference Matters**:
  - Lines 85-99: This is the exact code block to modify — both unwraps and the surrounding move logic
  - Lines 72-83: Shows what happens BEFORE the move — DB record already exists, so skipping the move is safe (file will be re-encountered but `record_ingestion` prevents duplicate DB writes)
  - claude_code.rs: Shows the established pattern for handling file errors gracefully in this codebase
  - test_ingest_moves_to_processed: The new edge-case test should be structurally similar to this existing test

  **Acceptance Criteria** (agent-executable only):

  **If TDD**:
  - [ ] New test `test_ccusage_handles_missing_path_components` exists in `src/ingest/ccusage.rs`
  - [ ] `cargo test test_ccusage_handles_missing_path_components -- --exact 2>&1 | tee .sisyphus/evidence/task-1-ccusage-edge-test.log` passes
  - [ ] `cargo test test_ingest_moves_to_processed -- --exact 2>&1 | tee .sisyphus/evidence/task-1-ccusage-happy-path.log` passes (regression)
  - [ ] `cargo test --locked 2>&1 | tee .sisyphus/evidence/task-1-ccusage-full-test.log` — all tests pass

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: Edge case — bare filename path does not panic
    Tool: Bash
    Preconditions: Test added for malformed path handling
    Steps:
      1. Run `cargo test test_ccusage_handles_missing_path_components -- --exact 2>&1 | tee .sisyphus/evidence/task-1-ccusage-edge-test.log`
    Expected: Test passes — no panic, ingest completes, best-effort handling confirmed
    Failure Indicators: Test panics, test fails, compilation error
    Evidence: .sisyphus/evidence/task-1-ccusage-edge-test.log

  Scenario: Happy path — successful move still works after fix
    Tool: Bash
    Preconditions: Existing test for normal move behavior
    Steps:
      1. Run `cargo test test_ingest_moves_to_processed -- --exact 2>&1 | tee .sisyphus/evidence/task-1-ccusage-happy-path.log`
    Expected: Test passes — file still moves to processed/ directory successfully
    Failure Indicators: Test fails or panics
    Evidence: .sisyphus/evidence/task-1-ccusage-happy-path.log

  Scenario: Zero production unwraps remain in ccusage.rs
    Tool: Bash
    Steps:
      1. Run: `awk '/#\[cfg\(test\)\]/,0' src/ingest/ccusage.rs | grep -c 'unwrap()' && echo "---test unwraps above---" && grep -c 'unwrap()' src/ingest/ccusage.rs && echo "---total unwraps above---"`
      2. Verify total unwraps equals test unwraps (i.e., zero in production code)
    Expected: All unwrap() calls are within #[cfg(test)] blocks only
    Evidence: .sisyphus/evidence/task-1-no-prod-unwraps.log
  ```

  **Commit**: YES
  - Message: `fix(ccusage): remove production unwraps from processed-file handling`
  - Files: `src/ingest/ccusage.rs`
  - Pre-commit: `cargo test --locked`

- [ ] 2. Add GitHub Actions CI workflow

  **What to do**:
  - Create `.github/workflows/ci.yml` with the following specification:
    - **Triggers**: `push` to `master`, `pull_request` (all branches)
    - **Runner**: `ubuntu-latest`
    - **Rust**: stable (use `dtolnay/rust-toolchain@stable`)
    - **Steps** (in order):
      1. Checkout code
      2. Install stable Rust toolchain
      3. `cargo fmt --all -- --check`
      4. `cargo build --locked`
      5. `cargo test --locked`
      6. `cargo clippy --locked --all-targets -- -D warnings`
  - Use `Cargo.lock`-based builds (`--locked`) to ensure reproducibility
  - Keep the workflow minimal — no caching, no matrix, no release steps

  **Must NOT do**:
  - Do not add release/publish automation
  - Do not add cross-platform matrix (DuckDB native dep makes this complex)
  - Do not add coverage reporting, cargo-audit, or other extras
  - Do not modify any source files
  - Do not add MSRV testing

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: single YAML file creation following standard pattern
  - Skills: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 3)
  - **Blocks**: F1, F2, F3, F4
  - **Blocked By**: none

  **References** (CRITICAL — executor has NO interview context):

  **Pattern References**:
  - `Cargo.toml` — confirms this is a single Rust crate (not a workspace), so no `--workspace` flags needed
  - `Cargo.lock` — exists, so `--locked` flags are valid

  **External References**:
  - GitHub Actions Rust starter: `https://github.com/actions/starter-workflows/blob/main/ci/rust.yml`
  - `dtolnay/rust-toolchain` action: standard Rust toolchain setup action

  **WHY Each Reference Matters**:
  - `Cargo.toml`: Confirms crate structure — single package, not a workspace, so CI commands don't need `--workspace` or `--all` flags
  - `Cargo.lock`: Confirms lockfile exists — `--locked` ensures CI uses exact dependency versions

  **Acceptance Criteria** (agent-executable only):
  - [ ] `.github/workflows/ci.yml` exists
  - [ ] File contains `dtolnay/rust-toolchain` or equivalent Rust setup
  - [ ] File contains `cargo fmt --all -- --check`
  - [ ] File contains `cargo build --locked`
  - [ ] File contains `cargo test --locked`
  - [ ] File contains `cargo clippy --locked --all-targets -- -D warnings`
  - [ ] File contains triggers for `push` to `master` and `pull_request`
  - [ ] YAML is valid: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"` succeeds

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: CI workflow file is valid and complete
    Tool: Bash
    Steps:
      1. Run `python3 -c "import yaml; d=yaml.safe_load(open('.github/workflows/ci.yml')); assert 'on' in d; assert 'jobs' in d; print('YAML valid')"` 2>&1 | tee .sisyphus/evidence/task-2-ci-yaml-valid.log
      2. Run `grep -c 'cargo fmt' .github/workflows/ci.yml && grep -c 'cargo build' .github/workflows/ci.yml && grep -c 'cargo test' .github/workflows/ci.yml && grep -c 'cargo clippy' .github/workflows/ci.yml` and verify each count is >= 1
    Expected: YAML parses, all 4 cargo commands present, triggers configured
    Failure Indicators: YAML parse error, missing command, missing trigger
    Evidence: .sisyphus/evidence/task-2-ci-yaml-valid.log

  Scenario: Local commands pass before committing
    Tool: Bash
    Steps:
      1. Run `cargo fmt --all -- --check 2>&1 | tee .sisyphus/evidence/task-2-fmt-check.log`
      2. Run `cargo clippy --locked --all-targets -- -D warnings 2>&1 | tee .sisyphus/evidence/task-2-clippy-check.log`
    Expected: Both exit 0 (these are the same commands CI will run)
    Evidence: .sisyphus/evidence/task-2-fmt-check.log, .sisyphus/evidence/task-2-clippy-check.log
  ```

  **Commit**: YES
  - Message: `ci: add GitHub Actions workflow for fmt, build, test, and clippy`
  - Files: `.github/workflows/ci.yml`

- [ ] 3. Sync README architecture section with current module structure

  **What to do**:
  - Update the "Architecture" section of `README.md` to reflect the actual current module structure
  - The current README lists only: `main.rs`, `cli/mod.rs`, `config/mod.rs`, `db/mod.rs`, `ingest/` (5 files), `commands/` (2 files), `llm/mod.rs`, `observability/mod.rs`, `storage/mod.rs`, `workflows/mod.rs`
  - The actual codebase has significantly more:
    - `commands/`: runs, stats, discover, collect, signals, triage, tasks, sync, status, workflows (10+ files)
    - Root modules: cli, config, db, ingest, llm, observability, registry, signals, storage, todoist, workflows, context
    - `ingest/`: mod, pricing, praxis_native, opencode, claude_code, ccusage (unchanged)
  - Use a **high-level module map** format — group by responsibility, list module name + one-line description
  - Do NOT create an exhaustive file tree (those go stale quickly)

  **Must NOT do**:
  - Do not add `ARCHITECTURE.md`, `CONTRIBUTING.md`, or any new doc files
  - Do not rewrite other sections of the README
  - Do not change the README tone, style, or formatting outside the architecture section
  - Do not add badges or links that weren't there before
  - Do not list every test file or internal helper

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: update one section of one file based on factual inventory
  - Skills: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 2)
  - **Blocks**: F1, F2, F3, F4
  - **Blocked By**: none

  **References** (CRITICAL — executor has NO interview context):

  **Pattern References** (what to match):
  - `README.md` lines around "## Architecture" — the section to update. Keep the same markdown formatting style (code block with file tree pattern, or switch to the module map format if that's cleaner)

  **Source of Truth** (what actually exists — READ these to build the accurate listing):
  - `src/` directory — list all `.rs` files and subdirectories to get the real module structure
  - `src/commands/` — list all command files (runs, stats, discover, collect, signals, triage, tasks, sync, status, workflows, mod)
  - `src/ingest/` — list all ingest files (mod, pricing, praxis_native, opencode, claude_code, ccusage)
  - Root modules in `src/`: cli, config, context, db, llm, observability, registry, signals, storage, todoist, workflows
  - `src/main.rs` — entry point

  **Test References**:
  - `tests/` directory — check if there are integration tests to mention (or not)

  **WHY Each Reference Matters**:
  - The README architecture section is currently inaccurate — it was written for Phase 1/2 and never updated for Phase 3 (control plane). The executor MUST read the actual `src/` directory to build the correct listing, not copy from the current README.

  **Acceptance Criteria** (agent-executable only):
  - [ ] README architecture section lists all root modules: cli, config, context, db, ingest, llm, observability, registry, signals, storage, todoist, workflows
  - [ ] README architecture section lists all command files under commands/
  - [ ] README architecture section lists all ingest files under ingest/
  - [ ] No module listed in README that doesn't exist in `src/`
  - [ ] No module in `src/` (top-level) that's missing from README

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: All actual modules are documented in README
    Tool: Bash
    Steps:
      1. List actual top-level modules: `ls src/*.rs src/*/mod.rs | sed 's|src/||;s|/mod.rs||;s|\.rs||' | sort`
      2. For each module, check README mentions it: `for m in $(ls src/ -d */ 2>/dev/null | sed 's|/||' | sort); do grep -qi "$m" README.md && echo "OK: $m" || echo "MISSING: $m"; done`
      3. Capture output to evidence file
    Expected: All modules show "OK", none show "MISSING"
    Failure Indicators: Any module shows "MISSING"
    Evidence: .sisyphus/evidence/task-3-readme-module-coverage.log

  Scenario: No phantom modules in README
    Tool: Bash
    Steps:
      1. Extract module names from README architecture section
      2. Verify each one exists in src/
    Expected: Every module mentioned in README actually exists in the codebase
    Evidence: .sisyphus/evidence/task-3-readme-no-phantoms.log
  ```

  **Commit**: YES
  - Message: `docs(readme): sync architecture section with current module structure`
  - Files: `README.md`

---

## Final Verification Wave (MANDATORY — after ALL implementation tasks)

> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.
> **Do NOT auto-proceed after verification. Wait for user's explicit approval before marking work complete.**
> **Never mark F1-F4 as checked before getting user's okay.** Rejection or user feedback -> fix -> re-run -> present again -> wait for okay.

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists (read file, curl endpoint, run command). For each "Must NOT Have": search codebase for forbidden patterns — reject with file:line if found. Check evidence files exist in `.sisyphus/evidence/`. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo fmt --all -- --check` + `cargo build --locked` + `cargo test --locked` + `cargo clippy --locked --all-targets -- -D warnings`. Review all changed files for: `as any` equivalents, empty catches, console.log equivalents, commented-out code, unused imports.
  Output: `Fmt [PASS/FAIL] | Build [PASS/FAIL] | Tests [N pass/N fail] | Clippy [PASS/FAIL] | Files [N clean/N issues] | VERDICT`

- [ ] F3. **Real Manual QA** — `unspecified-high`
  Start from clean state. Execute EVERY QA scenario from EVERY task — follow exact steps, capture evidence. Test cross-task integration. Save to `.sisyphus/evidence/final-qa/`.
  Output: `Scenarios [N/N pass] | Integration [N/N] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff (git log/diff). Verify 1:1 — everything in spec was built (no missing), nothing beyond spec was built (no creep). Check "Must NOT do" compliance. Flag unaccounted changes.
  Output: `Tasks [N/N compliant] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy
- **Task 1**: `fix(ccusage): remove production unwraps from processed-file handling` — src/ingest/ccusage.rs
- **Task 2**: `ci: add GitHub Actions workflow for fmt, build, test, and clippy` — .github/workflows/ci.yml
- **Task 3**: `docs(readme): sync architecture section with current module structure` — README.md

---

## Success Criteria

### Verification Commands
```bash
cargo fmt --all -- --check                                    # Expected: exit 0
cargo build --locked                                          # Expected: exit 0
cargo test --locked                                           # Expected: 101+ pass, 0 fail
cargo clippy --locked --all-targets -- -D warnings            # Expected: exit 0
grep -c 'unwrap()' src/ingest/ccusage.rs                      # Expected: matches only in #[cfg(test)]
test -f .github/workflows/ci.yml                              # Expected: file exists
```

### Final Checklist
- [ ] CI workflow runs on push to master + pull_request
- [ ] Zero production unwrap() in ccusage.rs
- [ ] README architecture section accurate
- [ ] All `cargo` commands pass clean

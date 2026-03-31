# Eliminate Rust Dead-Code Warnings

## TL;DR
> **Summary**: Remove the five reported Rust dead-code warnings with the smallest behavior-preserving changes: one helper becomes test-only, two deserialized fields become explicitly ignored, and two redundant stored fields are removed.
> **Deliverables**:
> - Warning-free implementations for the five reported sites
> - Release-build and test evidence proving the warnings are gone and behavior is preserved
> - Five atomic commits, one per warning site
> **Effort**: Short
> **Parallel**: YES - 1 implementation wave + 1 final verification wave
> **Critical Path**: Tasks 1-5 in parallel → release build/test verification → F1-F4 review wave

## Context
### Original Request
Address these warnings reported during a release build/install of the `praxis` binary:
- `src/db/mod.rs:236:8` — function `run_count` is never used
- `src/ingest/ccusage.rs:19:5` — field `report_type` is never read
- `src/ingest/opencode.rs:55:5` — field `cost` is never read
- `src/ingest/pricing.rs:75:5` — field `cache_path` is never read
- `src/todoist/mod.rs:89:9` — field `default_project` is never read

### Interview Summary
- Scope is limited to the five reported warning sites.
- Preserve current runtime behavior unless a code change is strictly required to compile or silence the warning correctly.
- Test strategy is **tests-after**.
- Repo-native verification is `cargo test`; this plan also uses `cargo build --release` because the warnings were surfaced in the release build path.

### Metis Review (gaps addressed)
- Locked fix choices so the executor has zero ambiguity:
  - `run_count` becomes a test-only helper.
  - `report_type` becomes `_report_type` to preserve serde compatibility without introducing new validation.
  - `cost` becomes `_cost` to preserve deserialization strictness while keeping token-derived pricing canonical.
  - `cache_path` is removed from `PricingDb`, while cache I/O logic stays intact.
  - `default_project` is removed from `TodoistClient`, while command-layer `config.todoist_project` resolution remains unchanged.
- Added explicit guardrails against scope creep in ccusage validation, opencode pricing semantics, pricing cache redesign, and Todoist project-resolution behavior.

## Work Objectives
### Core Objective
Eliminate the five reported dead-code warnings from the Rust codebase without changing observable application behavior.

### Deliverables
- Updated Rust code in the five targeted warning sites only.
- Evidence logs for targeted checks and final verification under `.sisyphus/evidence/`.
- Five atomic commits, one per warning site.

### Definition of Done (verifiable conditions with commands)
- `cargo build --release 2>&1 | tee .sisyphus/evidence/final-release-build.log` completes and does **not** emit any of the five reported warning strings.
- `cargo test 2>&1 | tee .sisyphus/evidence/final-cargo-test.log` passes.
- The following warning strings are absent from the final release-build log:
  - `function \`run_count\` is never used`
  - `field \`report_type\` is never read`
  - `field \`cost\` is never read`
  - `field \`cache_path\` is never read`
  - `field \`default_project\` is never read`
- `.sisyphus/evidence/task-5-todoist-default-project-contract.log` confirms Todoist task creation still resolves project IDs from `config.todoist_project` in `src/commands/tasks.rs`, not from `TodoistClient` state.
- `.sisyphus/evidence/task-3-opencode-cost-edge.log` confirms opencode pricing still derives cost from token pricing logic, not from the deserialized message payload field.

### Must Have
- Smallest semantically correct fix per warning site.
- No new user-visible features or config semantics.
- Exact evidence captured for each task and for final verification.
- One commit per warning site.

### Must NOT Have (guardrails, AI slop patterns, scope boundaries)
- No new `#[allow(dead_code)]` / `#[allow(unused)]` suppressions for these warnings.
- No new ccusage report-type validation or branching.
- No switch from token-derived opencode pricing to payload-provided cost.
- No cache subsystem redesign while removing `PricingDb.cache_path`.
- No movement of Todoist project-resolution responsibility from `src/commands/tasks.rs` into `TodoistClient`.
- No unrelated refactors outside the five target sites and their minimally necessary tests/evidence.

## Verification Strategy
> ZERO HUMAN INTERVENTION — all verification is agent-executed.
- Test decision: **tests-after** using existing Rust tests plus targeted release-build verification.
- QA policy: Every task includes a targeted happy-path check and an edge/preservation check.
- Evidence: `.sisyphus/evidence/task-{N}-{slug}.log`
- Final warning surface: `cargo build --release`
- Final regression surface: `cargo test`

## Execution Strategy
### Parallel Execution Waves
> Target: 5-8 tasks per wave. <3 per wave (except final) = under-splitting.
> Extract shared dependencies as Wave-1 tasks for max parallelism.

Wave 1: Task 1 `run_count`, Task 2 `report_type`, Task 3 `_cost`, Task 4 `cache_path`, Task 5 `default_project`

Wave 2: Final verification wave F1-F4 after all five implementation tasks and their per-task checks are complete

### Dependency Matrix (full, all tasks)
| Task | Depends On | Blocks |
|---|---|---|
| 1 | none | F1, F2, F3, F4 |
| 2 | none | F1, F2, F3, F4 |
| 3 | none | F1, F2, F3, F4 |
| 4 | none | F1, F2, F3, F4 |
| 5 | none | F1, F2, F3, F4 |
| F1 | 1,2,3,4,5 | completion |
| F2 | 1,2,3,4,5 | completion |
| F3 | 1,2,3,4,5 | completion |
| F4 | 1,2,3,4,5 | completion |

### Agent Dispatch Summary (wave → task count → categories)
- Wave 1 → 5 tasks → `quick`
- Wave 2 → 4 tasks → `oracle`, `unspecified-high`, `unspecified-high`, `deep`

## TODOs
> Implementation + Test = ONE task. Never separate.
> EVERY task MUST have: Agent Profile + Parallelization + QA Scenarios.

- [x] 1. Make `run_count` test-only

  **What to do**: In `src/db/mod.rs:235-243`, change the helper signature to a test-only local helper (`#[cfg(test)] fn run_count(...) -> ...`). Keep it in the same file so the existing test module can continue to access it through module scope. Do not change SQL or database schema behavior.
  **Must NOT do**: Do not add `#[allow(dead_code)]`; do not move the helper into runtime call paths; do not alter insert/upsert semantics.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: single-warning cleanup in one file with existing tests
  - Skills: `[]` — No special skill required
  - Omitted: `['/git-master']` — Commit strategy is already specified; no history research needed for the implementation itself

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: F1, F2, F3, F4 | Blocked By: none

  **References** (executor has NO interview context — be exhaustive):
  - Pattern: `src/db/mod.rs:235-243` — dead helper definition to convert to test-only
  - Test: `src/db/mod.rs:329-375` — `test_insert_run` covers the helper’s only known usage and duplicate-upsert behavior
  - Preserve: `src/db/mod.rs:333-355` — insert/upsert flow that must remain unchanged

  **Acceptance Criteria** (agent-executable only):
  - [ ] `cargo test test_insert_run -- --exact 2>&1 | tee .sisyphus/evidence/task-1-run-count.log` passes
  - [ ] `cargo build --release 2>&1 | tee .sisyphus/evidence/task-1-run-count-build.log` completes without `function \`run_count\` is never used`
  - [ ] `python - <<'PY' | tee .sisyphus/evidence/task-1-run-count-contract.log
from pathlib import Path
text = Path('src/db/mod.rs').read_text()
assert '#[cfg(test)]\nfn run_count' in text or '#[cfg(test)]\r\nfn run_count' in text
assert 'pub fn run_count' not in text
print('run_count gated to tests')
PY` succeeds

  **QA Scenarios** (MANDATORY — task incomplete without these):
  ```
  Scenario: DB insert/upsert behavior still passes
    Tool: Bash
    Steps:
      1. Run `cargo test test_insert_run -- --exact 2>&1 | tee .sisyphus/evidence/task-1-run-count.log`
    Expected: Test passes and still validates that duplicate upserts leave the run count at 1.
    Evidence: .sisyphus/evidence/task-1-run-count.log

  Scenario: Release build no longer reports dead helper
    Tool: Bash
    Steps:
      1. Run `cargo build --release 2>&1 | tee .sisyphus/evidence/task-1-run-count-build.log`
      2. Verify the log does not contain `function \`run_count\` is never used`
    Expected: Build succeeds and the warning string is absent.
    Evidence: .sisyphus/evidence/task-1-run-count-build.log

  Scenario: Helper is test-gated and no longer public
    Tool: Bash
    Steps:
      1. Run `python - <<'PY' | tee .sisyphus/evidence/task-1-run-count-contract.log`
         `from pathlib import Path`
         `text = Path('src/db/mod.rs').read_text()`
         `assert '#[cfg(test)]\nfn run_count' in text or '#[cfg(test)]\r\nfn run_count' in text`
         `assert 'pub fn run_count' not in text`
         `print('run_count gated to tests')`
         `PY`
    Expected: Script exits 0 and prints `run_count gated to tests`.
    Evidence: .sisyphus/evidence/task-1-run-count-contract.log
  ```

  **Commit**: YES | Message: `refactor(db): gate run_count to tests` | Files: `src/db/mod.rs`

- [x] 2. Mark ccusage report type intentionally unused

  **What to do**: In `src/ingest/ccusage.rs:16-21`, rename `report_type` to `_report_type` while preserving `#[serde(rename = "type")]` and the existing `Option<String>` shape. This removes the dead-code warning without changing the accepted payload shape or introducing new semantics around report types.
  **Must NOT do**: Do not add report-type validation, do not branch on the field value, and do not remove the serde rename.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: one-field rename plus targeted parser verification
  - Skills: `[]` — No special skill required
  - Omitted: `['/git-master']` — No history analysis needed

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: F1, F2, F3, F4 | Blocked By: none

  **References** (executor has NO interview context — be exhaustive):
  - Pattern: `src/ingest/ccusage.rs:16-21` — serde struct field to rename with intent-preserving underscore
  - API/Type: `src/ingest/ccusage.rs:116-195` — `parse_ccusage_file` deserializes this struct at line 120
  - Preserve: `src/ingest/ccusage.rs:122-136` — source inference from filename must remain unchanged
  - Preserve: `src/ingest/ccusage.rs:163-186` — data-quality branching must remain unchanged
  - Test: `src/ingest/ccusage.rs:237-269` — `test_parse_ccusage_claude_code` and `test_parse_ccusage_opencode`

  **Acceptance Criteria** (agent-executable only):
  - [ ] `cargo test test_parse_ccusage_claude_code -- --exact 2>&1 | tee .sisyphus/evidence/task-2-report-type.log` passes
  - [ ] `cargo test test_parse_ccusage_opencode -- --exact 2>&1 | tee .sisyphus/evidence/task-2-report-type-edge.log` passes
  - [ ] `cargo build --release 2>&1 | tee .sisyphus/evidence/task-2-report-type-build.log` completes without `field \`report_type\` is never read`

  **QA Scenarios** (MANDATORY — task incomplete without these):
  ```
  Scenario: Claude Code ccusage fixture still parses
    Tool: Bash
    Steps:
      1. Run `cargo test test_parse_ccusage_claude_code -- --exact 2>&1 | tee .sisyphus/evidence/task-2-report-type.log`
    Expected: Test passes with no new report-type validation behavior.
    Evidence: .sisyphus/evidence/task-2-report-type.log

  Scenario: Opencode-flavored ccusage input still follows existing branching
    Tool: Bash
    Steps:
      1. Run `cargo test test_parse_ccusage_opencode -- --exact 2>&1 | tee .sisyphus/evidence/task-2-report-type-edge.log`
    Expected: Test passes and existing filename/source inference logic remains unchanged.
    Evidence: .sisyphus/evidence/task-2-report-type-edge.log
  ```

  **Commit**: YES | Message: `refactor(ingest): mark ccusage report type unused` | Files: `src/ingest/ccusage.rs`

- [x] 3. Preserve opencode deserialization contract while ignoring raw cost

  **What to do**: In `src/ingest/opencode.rs:36-58`, rename `cost` to `_cost` instead of removing it. Keep the field deserialized so the payload contract stays as strict as it is today, but continue treating `PricingDb::estimate_cost` as the only authoritative pricing path in `parse_message_file`.
  **Must NOT do**: Do not remove the field, do not switch runtime pricing to the payload field, and do not alter session/project inference behavior.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: single-field intent rename with strong existing tests
  - Skills: `[]` — No special skill required
  - Omitted: `['/git-master']` — No history analysis needed

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: F1, F2, F3, F4 | Blocked By: none

  **References** (executor has NO interview context — be exhaustive):
  - Pattern: `src/ingest/opencode.rs:36-58` — deserialized message struct containing the unused field
  - API/Type: `src/ingest/opencode.rs:186-228` — `parse_message_file`; cost is recomputed at `199-205`
  - Preserve: `src/ingest/opencode.rs:198-221` — token-derived pricing and ingest record construction must remain canonical
  - Test: `src/ingest/opencode.rs:290-320` — `test_parse_message_json`
  - Test: `src/ingest/opencode.rs:323-382` — `test_ingest_full_flow`, `test_ingest_sets_run_project_from_session`
  - Preserve: `src/ingest/opencode.rs:336,367` — fixtures include `"cost":0`; parsing must still succeed without making payload cost authoritative

  **Acceptance Criteria** (agent-executable only):
  - [ ] `cargo test test_parse_message_json -- --exact 2>&1 | tee .sisyphus/evidence/task-3-opencode-cost.log` passes
  - [ ] `cargo test test_ingest_full_flow -- --exact 2>&1 | tee .sisyphus/evidence/task-3-opencode-cost-edge.log` passes
  - [ ] `cargo build --release 2>&1 | tee .sisyphus/evidence/task-3-opencode-cost-build.log` completes without `field \`cost\` is never read`

  **QA Scenarios** (MANDATORY — task incomplete without these):
  ```
  Scenario: Single-message parse still succeeds
    Tool: Bash
    Steps:
      1. Run `cargo test test_parse_message_json -- --exact 2>&1 | tee .sisyphus/evidence/task-3-opencode-cost.log`
    Expected: Test passes and message ingestion still succeeds with payloads that include `"cost": 0`.
    Evidence: .sisyphus/evidence/task-3-opencode-cost.log

  Scenario: Full ingest flow still uses token-derived pricing
    Tool: Bash
    Steps:
      1. Run `cargo test test_ingest_full_flow -- --exact 2>&1 | tee .sisyphus/evidence/task-3-opencode-cost-edge.log`
    Expected: Test passes and cost continues to be computed from `PricingDb::estimate_cost`, not from the raw payload field.
    Evidence: .sisyphus/evidence/task-3-opencode-cost-edge.log
  ```

  **Commit**: YES | Message: `refactor(ingest): preserve opencode cost contract` | Files: `src/ingest/opencode.rs`

- [x] 4. Remove dead `PricingDb.cache_path` state

  **What to do**: Remove the `cache_path` field from `PricingDb` in `src/ingest/pricing.rs:71-76` and update the struct initializers in `load`, `empty`, and test helpers accordingly. Preserve the local `cache_path` variable and all existing cache read/write logic inside `load`; only remove the never-read stored field.
  **Must NOT do**: Do not redesign caching, do not change cache staleness rules, and do not alter fallback pricing behavior.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: one dead field and initializer cleanup in a single module
  - Skills: `[]` — No special skill required
  - Omitted: `['/git-master']` — No history analysis needed

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: F1, F2, F3, F4 | Blocked By: none

  **References** (executor has NO interview context — be exhaustive):
  - Pattern: `src/ingest/pricing.rs:71-76` — struct field to remove
  - API/Type: `src/ingest/pricing.rs:89-126` — `load` constructs the struct and must keep local cache-path I/O logic
  - Preserve: `src/ingest/pricing.rs:106-112` — cache write path
  - Preserve: `src/ingest/pricing.rs:185-195` — cache-read / staleness semantics
  - Test/Helper: `src/ingest/pricing.rs:223-300` — `db_from_fallback` and `test_cache_staleness_logic`

  **Acceptance Criteria** (agent-executable only):
  - [ ] `cargo test test_cache_staleness_logic -- --exact 2>&1 | tee .sisyphus/evidence/task-4-pricing-cache-path.log` passes
  - [ ] `cargo build --release 2>&1 | tee .sisyphus/evidence/task-4-pricing-cache-path-build.log` completes without `field \`cache_path\` is never read`
  - [ ] `python - <<'PY' | tee .sisyphus/evidence/task-4-pricing-cache-path-contract.log
from pathlib import Path
text = Path('src/ingest/pricing.rs').read_text()
assert 'cache_path: PathBuf' not in text
assert 'let cache_path =' in text
print('pricing cache_path field removed, local logic preserved')
PY` succeeds

  **QA Scenarios** (MANDATORY — task incomplete without these):
  ```
  Scenario: Cache staleness behavior still passes
    Tool: Bash
    Steps:
      1. Run `cargo test test_cache_staleness_logic -- --exact 2>&1 | tee .sisyphus/evidence/task-4-pricing-cache-path.log`
    Expected: Test passes and existing stale-cache behavior is unchanged.
    Evidence: .sisyphus/evidence/task-4-pricing-cache-path.log

  Scenario: Release build no longer reports dead pricing state
    Tool: Bash
    Steps:
      1. Run `cargo build --release 2>&1 | tee .sisyphus/evidence/task-4-pricing-cache-path-build.log`
      2. Verify the log does not contain `field \`cache_path\` is never read`
    Expected: Build succeeds and the warning string is absent.
    Evidence: .sisyphus/evidence/task-4-pricing-cache-path-build.log

  Scenario: Stored field is removed without deleting cache-path I/O logic
    Tool: Bash
    Steps:
      1. Run `python - <<'PY' | tee .sisyphus/evidence/task-4-pricing-cache-path-contract.log`
         `from pathlib import Path`
         `text = Path('src/ingest/pricing.rs').read_text()`
         `assert 'cache_path: PathBuf' not in text`
         `assert 'let cache_path =' in text`
         `print('pricing cache_path field removed, local logic preserved')`
         `PY`
    Expected: Script exits 0 and prints `pricing cache_path field removed, local logic preserved`.
    Evidence: .sisyphus/evidence/task-4-pricing-cache-path-contract.log
  ```

  **Commit**: YES | Message: `refactor(pricing): remove dead cache path state` | Files: `src/ingest/pricing.rs`

- [x] 5. Remove redundant `TodoistClient.default_project` state

  **What to do**: Remove `default_project` from `TodoistClient` in `src/todoist/mod.rs:85-90` and delete its constructor assignment in `src/todoist/mod.rs:92-111`. Leave command-layer project selection exactly where it is today in `src/commands/tasks.rs:79-92`; this task is field removal only, not a behavior refactor.
  **Must NOT do**: Do not move project resolution into `TodoistClient`, do not change `create_task` semantics, and do not alter the existing “lookup failure -> create without project” flow in the commands layer.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: dead state removal with behavior-preservation guardrails across two modules
  - Skills: `[]` — No special skill required
  - Omitted: `['/git-master']` — No history analysis needed

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: F1, F2, F3, F4 | Blocked By: none

  **References** (executor has NO interview context — be exhaustive):
  - Pattern: `src/todoist/mod.rs:85-90` — redundant field to remove
  - API/Type: `src/todoist/mod.rs:92-111` — constructor that currently assigns `default_project`
  - Preserve: `src/todoist/mod.rs:140-172` — `create_task()` implementation must remain behaviorally unchanged
  - Preserve: `src/todoist/mod.rs:214-225` — `resolve_project_id()` behavior remains unchanged
  - Preserve: `src/commands/tasks.rs:79-92` — current command-layer project resolution from `config.todoist_project`
  - Preserve: `src/config/mod.rs:16-21,159-166,217-220` — config source of truth for Todoist settings
  - Closest tests: `src/todoist/mod.rs:228-298` — existing Todoist tests cover parsing/live fetch only, not project selection

  **Acceptance Criteria** (agent-executable only):
  - [ ] `cargo build --release 2>&1 | tee .sisyphus/evidence/task-5-todoist-default-project-build.log` completes without `field \`default_project\` is never read`
  - [ ] `cargo test 2>&1 | tee .sisyphus/evidence/task-5-todoist-default-project-test.log` passes after the field removal
  - [ ] `python - <<'PY' | tee .sisyphus/evidence/task-5-todoist-default-project-contract.log
from pathlib import Path
client = Path('src/todoist/mod.rs').read_text()
cmd = Path('src/commands/tasks.rs').read_text()
assert 'default_project' not in client
assert 'config.todoist_project' in cmd
print('todoist contract preserved')
PY` succeeds

  **QA Scenarios** (MANDATORY — task incomplete without these):
  ```
  Scenario: Repo still compiles and tests after removing redundant client state
    Tool: Bash
    Steps:
      1. Run `cargo test 2>&1 | tee .sisyphus/evidence/task-5-todoist-default-project-test.log`
    Expected: Test suite passes and field removal does not break Todoist-related compilation paths.
    Evidence: .sisyphus/evidence/task-5-todoist-default-project-test.log

  Scenario: Command-layer project resolution contract is preserved
    Tool: Bash
    Steps:
      1. Run the following check and capture output to `.sisyphus/evidence/task-5-todoist-default-project-contract.log`:
         `python - <<'PY' | tee .sisyphus/evidence/task-5-todoist-default-project-contract.log\nfrom pathlib import Path\nclient = Path('src/todoist/mod.rs').read_text()\ncmd = Path('src/commands/tasks.rs').read_text()\nassert 'default_project' not in client\nassert 'config.todoist_project' in cmd\nprint('todoist contract preserved')\nPY`
    Expected: Script exits 0, prints `todoist contract preserved`, and confirms command-layer project selection still comes from config.
    Evidence: .sisyphus/evidence/task-5-todoist-default-project-contract.log
  ```

  **Commit**: YES | Message: `refactor(todoist): remove redundant default project field` | Files: `src/todoist/mod.rs`

## Final Verification Wave (MANDATORY — after ALL implementation tasks)
> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.
> **Do NOT auto-proceed after verification. Wait for user's explicit approval before marking work complete.**
> **Never mark F1-F4 as checked before getting user's okay.** Rejection or user feedback -> fix -> re-run -> present again -> wait for okay.
- [x] F1. Plan Compliance Audit — oracle
- [x] F2. Code Quality Review — unspecified-high
- [x] F3. Real Manual QA — unspecified-high (+ playwright if UI)
- [x] F4. Scope Fidelity Check — deep

## Commit Strategy
- Commit each implementation task separately, in task order, after its targeted QA passes.
- If full-suite verification or final review surfaces a narrow, non-runtime blocker (for example a time-sensitive test or evidence gap), add a separate follow-up commit in the affected source/test file instead of rewriting history on `master`.
- Commit messages:
  1. `refactor(db): gate run_count to tests`
  2. `refactor(ingest): mark ccusage report type unused`
  3. `refactor(ingest): preserve opencode cost contract`
  4. `refactor(pricing): remove dead cache path state`
  5. `refactor(todoist): remove redundant default project field`
- Do not squash before the final verification wave is approved.

## Success Criteria
- All five original warning sites are eliminated.
- Release build completes without the five warning messages.
- `cargo test` passes.
- No runtime behavior changes are introduced in ccusage parsing, opencode cost derivation, pricing cache I/O, or Todoist project selection.

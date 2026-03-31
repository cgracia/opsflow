# Tidy Up: Gitignore + Archive Completed Plan

## TL;DR
> **Summary**: Update `.gitignore` to follow Sisyphus artifact conventions (commit plans, gitignore evidence/notepads), then commit the completed dead-code plan as historical record.
> 
> **Deliverables**:
> - Updated `.gitignore` with selective `.sisyphus/` pattern
> - Completed plan committed to repo
> 
> **Estimated Effort**: Quick
> **Parallel Execution**: NO — sequential, 2 tasks
> **Critical Path**: Task 1 → Task 2

---

## Context

### Original Request
Close out the completed `dead-code-warning-cleanup` plan and tidy up the `.sisyphus/` directory. Research confirmed the official oh-my-openagent convention: gitignore everything in `.sisyphus/` except `plans/` and `rules/`.

### Research Findings
- Official pattern from oh-my-openagent: `.sisyphus/*` + `!.sisyphus/rules/`
- Real-world projects commit plans as historical documentation
- Evidence files (build/test logs) should NOT be committed — they're transient
- GitHub issue #1101 confirms committing evidence causes problems

---

## Work Objectives

### Core Objective
Establish a clean gitignore policy for Sisyphus artifacts and archive the completed plan.

### Definition of Done
- [ ] `.gitignore` contains selective `.sisyphus/` pattern
- [ ] `.sisyphus/plans/dead-code-warning-cleanup.md` is tracked in git
- [ ] `.sisyphus/evidence/` and `.sisyphus/notepads/` are NOT tracked

### Must Have
- Selective gitignore: plans committed, evidence/notepads ignored
- The completed plan file committed

### Must NOT Have
- No changes to source code
- No evidence or notepad files committed

---

## Verification Strategy
- Test decision: None (no code changes)
- QA: `git status` after each task confirms correct tracking state

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (sequential):
├── Task 1: Update .gitignore [quick]
└── Task 2: Commit completed plan [quick]
```

### Dependency Matrix
| Task | Depends On | Blocks |
|---|---|---|
| 1 | none | 2 |
| 2 | 1 | completion |

### Agent Dispatch Summary
- Wave 1 → 2 tasks → `quick`

---

## TODOs

- [x] 1. Update `.gitignore` with selective Sisyphus pattern

  **What to do**:
  - Replace the current `.gitignore` content (which only has `/target`) with the selective pattern
  - Add the following lines after `/target`:
    ```
    # Sisyphus orchestration artifacts
    .sisyphus/*
    !.sisyphus/plans/
    !.sisyphus/rules/
    ```
  - This ignores evidence/, notepads/, drafts/, and state files while allowing plans/ and rules/ to be committed

  **Must NOT do**:
  - Do not remove the existing `/target` line
  - Do not commit evidence or notepad files

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: single file edit, 3 lines added
  - Skills: `[]`

  **Parallelization**: Can Parallel: NO | Sequential | Blocks: Task 2 | Blocked By: none

  **References**:
  - `.gitignore` — current content is just `/target`
  - The convention comes from oh-my-openagent's official `.gitignore`

  **Acceptance Criteria**  (agent-executable only):
  - [ ] `.gitignore` contains `.sisyphus/*`
  - [ ] `.gitignore` contains `!.sisyphus/plans/`
  - [ ] `.gitignore` contains `!.sisyphus/rules/`
  - [ ] `.gitignore` still contains `/target`
  - [ ] `git status` shows `.sisyphus/plans/` as untracked (not ignored), but `.sisyphus/evidence/` and `.sisyphus/notepads/` are ignored

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: Plans are trackable, evidence is ignored
    Tool: Bash
    Steps:
      1. Run `git status .sisyphus/plans/`
      2. Verify plan files appear as untracked (not ignored)
      3. Run `git status .sisyphus/evidence/`
      4. Verify evidence files do NOT appear (they are gitignored)
      5. Run `git status .sisyphus/notepads/`
      6. Verify notepad files do NOT appear (they are gitignored)
    Expected: Plans trackable, evidence and notepads ignored
    Evidence: .sisyphus/evidence/task-1-gitignore-verify.log
  ```

  **Commit**: NO (groups with Task 2)

- [ ] 2. Commit the gitignore update and completed plan

  **What to do**:
  - Stage `.gitignore` and `.sisyphus/plans/dead-code-warning-cleanup.md`
  - Do NOT stage anything from `.sisyphus/evidence/` or `.sisyphus/notepads/`
  - Commit with message: `chore: archive dead-code plan, add sisyphus gitignore`

  **Must NOT do**:
  - Do not stage evidence or notepad files
  - Do not modify any source files

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: git add + commit
  - Skills: `['/git-master']` — Reason: git operations

  **Parallelization**: Can Parallel: NO | Sequential | Blocks: completion | Blocked By: Task 1

  **References**:
  - `.gitignore` — just updated in Task 1
  - `.sisyphus/plans/dead-code-warning-cleanup.md` — the completed plan to commit

  **Acceptance Criteria**  (agent-executable only):
  - [ ] `git log --oneline -1` shows the new commit
  - [ ] `git show --stat HEAD` includes `.gitignore` and `.sisyphus/plans/dead-code-warning-cleanup.md`
  - [ ] `git status` shows a clean working tree (no untracked `.sisyphus/` files appearing)

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: Clean state after commit
    Tool: Bash
    Steps:
      1. Run `git status`
      2. Verify working tree is clean (no untracked files)
      3. Run `git show --stat HEAD`
      4. Verify commit includes .gitignore and the plan file
    Expected: Clean working tree, correct files committed
    Evidence: .sisyphus/evidence/task-2-commit-verify.log
  ```

  **Commit**: YES
  - Message: `chore: archive dead-code plan, add sisyphus gitignore`
  - Files: `.gitignore`, `.sisyphus/plans/dead-code-warning-cleanup.md`

---

## Final Verification Wave

> Skipped — trivial change, no source code involved. The QA scenarios in Tasks 1-2 are sufficient.

---

## Commit Strategy
- Single commit: `chore: archive dead-code plan, add sisyphus gitignore`

---

## Success Criteria

### Verification Commands
```bash
git status              # Expected: clean working tree
git log --oneline -3    # Expected: new commit on top
ls .sisyphus/evidence/  # Expected: files still exist locally (not deleted, just ignored)
```

### Final Checklist
- [ ] `.gitignore` updated with selective pattern
- [ ] Plan file committed
- [ ] Evidence files still present locally but gitignored
- [ ] Working tree clean

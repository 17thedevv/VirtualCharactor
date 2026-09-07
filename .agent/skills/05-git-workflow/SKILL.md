---
name: Git Workflow
description: Skill for managing code changes safely in a parallel two-developer environment.
---

# Git Workflow

## Purpose
This skill ensures agents respect the collaborative boundaries of the VirtualCharacter project, allowing Dev A (Core) and Dev B (Runtime) to work simultaneously without creating merge conflicts or breaking the `main` branch.

## When to Apply
Apply this skill when structuring commits, creating branches, or preparing to integrate changes.

## Required Reading
- [docs/development.md](file:///d:/Project-17/VirtualCharactor/docs/development.md)

## Core Principles
1. **Main is Sacred**: The `main` branch must always compile and pass tests.
2. **Isolation**: Keep commits scoped to a specific task or subsystem.
3. **Respect Ownership**: Do not touch the other developer's files unless absolutely necessary and coordinated.

## Workflow
1. Create a feature branch off the latest `main`.
2. Implement changes, ensuring you are working within your assigned ownership area (`contracts.md`).
3. Write tests and run `cargo check` and `cargo test`.
4. Commit changes with clear, scoped messages.
5. If you must touch shared contracts, resolve those changes explicitly (ask the user).
6. Merge back to `main`.

## Rules
- **No Force-Pushing**: Do not force-push to shared branches.
- **Atomic Commits**: Keep commits logical. Don't mix formatting changes with core logic changes.
- **Update Before Integration**: Always ensure your changes are rebased or merged with the latest `main` before finalizing.

## Anti-Patterns
- Committing a broken build to `main`.
- Developer A (working on `vc-core`) arbitrarily refactoring `vc-runtime` code to make a test pass, breaking Developer B's workflow.
- Creating massive monolithic commits that touch 50 files across all crates.

## Validation Checklist
- [ ] Does `cargo test` pass cleanly on the branch?
- [ ] Are the changes scoped to my designated ownership area?
- [ ] Is the commit history logical and readable?

## Completion Criteria
Changes are successfully merged into the target branch, keeping the build green and respecting ownership boundaries.

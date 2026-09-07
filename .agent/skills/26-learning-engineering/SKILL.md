---
name: Learning Engineering
description: Skill for distinguishing real-time adaptation from offline model training.
---

# Learning Engineering

## Purpose
This skill teaches agents the strict architectural distinction between a character learning a fact (Runtime Adaptation) and an ML model updating its weights (Model Training). It prevents catastrophic drift.

## When to Apply
Apply this skill when discussing or implementing personalization, feedback loops, or learned ML components.

## Required Reading
- [docs/design/learning.md](file:///d:/Project-17/VirtualCharactor/docs/design/learning.md)

## Core Principles
1. **Runtime adaptation ≠ Model training**: Saving a memory is fast and deterministic. Training a model is offline and batched.
2. **Learned Models are Optional**: The system must work perfectly using only deterministic rules. Learned components (like emotion predictors) are advisory add-ons.
3. **Domain Validation Authority**: The Domain (`vc-core`) always validates and can veto any suggestion from a learned model.

## Workflow
1. If a user asks to "make the character learn", classify the request:
   - Is it remembering a fact? -> Use Memory update.
   - Is it changing an emotion? -> Use State update.
   - Is it updating a relationship? -> Use Relationship update.
   - Is it predicting behavior? -> This requires an offline Learned Component.
2. Never implement per-message automatic model weight updates in the runtime.

## Rules
- **Optional & Rollback-capable**: Any learned component introduced must be versioned and easy to disable via config, falling back to rule-based logic.
- **Privacy Filtering**: Runtime data must be redacted of PII before ever becoming training data.

## Anti-Patterns
- Adding a PyTorch dependency to `vc-core` to train a model during a chat interaction.
- Letting a learned component bypass `vc-core` validation and force an action.
- Confusing a user preference (e.g., "I like short answers") with global character personality drift.

## Validation Checklist
- [ ] Is the "learning" properly classified (Memory vs State vs Model)?
- [ ] Are learned components strictly advisory and subject to Domain Validation?
- [ ] Is model training kept strictly offline/batched?

## Completion Criteria
The system adapts to users gracefully through deterministic state/memory updates, while keeping actual ML model training safely isolated, optional, and offline.

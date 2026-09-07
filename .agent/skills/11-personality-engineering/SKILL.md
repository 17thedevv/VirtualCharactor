---
name: Personality Engineering
description: Skill for modifying and structuring character personality and core identity.
---

# Personality Engineering

## Purpose
This skill teaches agents how to modify a character's Personality safely. It emphasizes that Personality is a stable, structured domain object, not a simple string prompt, and must be protected from accidental runtime modification.

## When to Apply
Apply this skill when defining core traits, values, behavioral tendencies, or modifying `vc-core::personality`.

## Required Reading
- [docs/design/personality.md](file:///d:/Project-17/VirtualCharactor/docs/design/personality.md)

## Core Principles
1. **Personality is Structured Data**: Traits, Values, and Boundaries are distinct fields, not just a blob of text.
2. **Core Identity is Stable**: The character's fundamental nature does not change from a single interaction.
3. **Not Just a System Prompt**: Personality structures are parsed and evaluated by domain logic, then injected into context.

## Workflow
1. When adding a new personality dimension, define it as a strongly typed enum or struct in `vc-core::personality`.
2. Add it to the `Personality` contract.
3. Update `docs/contracts.md` or request a contract review if modifying an existing stable field.

## Rules
- **Temporary Behavior Belongs to State**: Do not modify Personality to make a character "angry". That is `EmotionState`.
- **No Silent Rewrites**: User-specific adaptation must NOT silently rewrite the global personality of the character.

## Anti-Patterns
- Storing "Current Mood" inside the `Personality` struct.
- Changing `Core Value: Honesty = Low` just because one user asked the character to lie.

## Validation Checklist
- [ ] Are new personality dimensions structured data (not raw strings)?
- [ ] Did I ensure this change doesn't make personality ephemeral?
- [ ] Was contract review requested for stable fields?

## Completion Criteria
Personality modifications are strongly typed, globally stable, and conceptually distinct from temporary state and memory.

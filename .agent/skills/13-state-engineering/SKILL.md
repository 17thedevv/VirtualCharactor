---
name: State Engineering
description: Skill for managing dynamic, transient, and persistent state transitions.
---

# State Engineering

## Purpose
This skill teaches agents how to design and modify the `CharacterState` and its sub-components, ensuring correct transitions, invariants, and proper segregation from relationship states.

## When to Apply
Apply this skill when modifying emotions, cognitive load, behavioral modifiers, short-term goals, or session boundaries.

## Required Reading
- [docs/design/state.md](file:///d:/Project-17/VirtualCharactor/docs/design/state.md)

## Core Principles
1. **State is Dynamic**: State changes frequently during interactions.
2. **State Subsystems**: `CharacterState` comprises Emotion, Cognitive, Behavior, Goals, and Session state.
3. **RelationshipState is Separate**: Relationship state belongs to the `Relationship` entity, not `CharacterState`.

## Workflow
1. Identify the state subsystem to modify (e.g., `state/emotion.rs`).
2. Define the invariant rules for the state (e.g., Joy must be between 0.0 and 1.0).
3. Write deterministic state transition tests.
4. Ensure the state can decay or normalize over time if applicable.

## Rules
- **Preserve Invariants**: Use functions to mutate state (e.g., `add_joy(0.1)`), never public fields (`pub joy: f32`).
- **Support Future Engines**: Design state (like Emotion) so it can eventually support `RuleBasedEmotionEngine`, `LearnedEmotionEngine`, or `HybridEmotionEngine`.

## Anti-Patterns
- Treating State like a generic `HashMap<String, String>`.
- Updating `CharacterState` based on a specific User's input without considering the global impact (if CharacterState is global).

## Validation Checklist
- [ ] Are state transitions encapsulated in methods that enforce bounds?
- [ ] Are deterministic unit tests provided for transitions?
- [ ] Is it clearly defined whether this state persists across sessions?

## Completion Criteria
State structures enforce domain invariants programmatically and have robust tests proving correct transitions and decay.

---
name: Event Engineering
description: Skill for designing and dispatching domain events correctly.
---

# Event Engineering

## Purpose
This skill teaches agents the semantic meaning of an Event in the VirtualCharacter architecture, ensuring it is used to decouple systems rather than as a substitute for direct function calls or logging.

## When to Apply
Apply this skill when defining new events in `vc-core::event` or implementing event dispatching in the runtime.

## Required Reading
- [docs/design/event.md](file:///d:/Project-17/VirtualCharactor/docs/design/event.md)

## Core Principles
1. **Event = Fact**: An event represents something that has already happened (e.g., `UserMessageReceived`, `EmotionChanged`).
2. **Immutability**: Events cannot be changed once emitted.
3. **Decoupling**: Events are used so subsystem A doesn't need to know about subsystem B (e.g., Interaction emitting an event that Memory listens to for consolidation).

## Workflow
1. Identify the domain fact that occurred.
2. Define the Event struct/enum.
3. Determine its persistence policy (Ephemeral vs Persistent).
4. Dispatch it via the orchestration layer.

## Rules
- **Not a Command**: Do not use events to tell another system what to do (e.g., `UpdateMemoryEvent` is bad. `InteractionCompletedEvent` is good).
- **Not a Log**: Do not create events for every trivial function call. Use Observability (tracing) for execution evidence.
- **Idempotency**: Event handlers must be idempotent. Processing `EmotionChanged` twice shouldn't double the emotion score.

## Anti-Patterns
- Using an external message broker (Kafka/Redis) when the architecture explicitly forbids it for Phase 0.
- Creating three copies of the exact same data as an Event, a Memory, and a Trace.

## Validation Checklist
- [ ] Is the event named in the past tense?
- [ ] Is it a domain fact rather than an infrastructure log?
- [ ] Are handlers for this event idempotent?

## Completion Criteria
Events correctly broadcast domain facts to decoupled subsystems without blurring the lines with Observability or Memory.

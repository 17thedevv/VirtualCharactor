---
name: Observability Engineering
description: Skill for tracing execution paths without leaking sensitive domain or user data.
---

# Observability Engineering

## Purpose
This skill guides agents in adding structured logging and tracing to VirtualCharacter. It enforces the boundary between "evidence of execution" (Observability) and "character knowledge" (Domain State).

## When to Apply
Apply this skill when adding `tracing` spans, logging errors, or tracking the interaction pipeline.

## Required Reading
- [docs/design/observability.md](file:///d:/Project-17/VirtualCharactor/docs/design/observability.md)

## Core Principles
1. **Observability ≠ Domain State**: A trace proves code ran; it does not constitute a character memory.
2. **Correlation**: Use Correlation IDs (`InteractionId`) to trace a single request across subsystems.
3. **Privacy First**: Logs must never leak PII or secrets.

## Workflow
1. Instrument the pipeline using `tracing::instrument`.
2. Add spans for: Memory retrieval, Context construction, Decision, LLM call, State update.
3. Attach `InteractionId` and `CharacterId` to spans, but NOT raw user text by default.

## Rules
- **Never log by default**: API keys, credentials, private memory contents, unnecessary raw prompts, or hidden chain-of-thought text.
- **Structured Traces**: Prefer structured metadata over raw string concatenation in logs.

## Anti-Patterns
- `println!("User said: {}", raw_message);` in production code.
- Using the observability system as a hacky way to store character memories.
- Adding heavyweight OpenTelemetry SDKs when not explicitly requested by the repo.

## Validation Checklist
- [ ] Are spans correctly nested to reflect the interaction lifecycle?
- [ ] Have all secrets and PII been explicitly excluded from default log levels?
- [ ] Can an interaction be traced end-to-end using a single Correlation ID?

## Completion Criteria
The system provides deep, structured insight into its execution flow for debugging and evaluation, completely independent of the character's internal domain state and without compromising privacy.

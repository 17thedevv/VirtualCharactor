---
name: LLM Provider Engineering
description: Skill for implementing provider abstractions and ensuring core domain isolation from LLM SDKs.
---

# LLM Provider Engineering

## Purpose
This skill teaches agents how to work with external AI models (like Gemini) while strictly respecting the provider abstraction layer. It ensures the core domain remains completely unaware of which LLM is running.

## When to Apply
Apply this skill when modifying `vc-llm`, adding new AI providers, or adjusting how LLM requests/responses are handled.

## Required Reading
- [docs/design/llm-provider.md](file:///d:/Project-17/VirtualCharactor/docs/design/llm-provider.md)

## Core Principles
1. **Provider Abstraction**: `LlmProvider` is a trait. `GeminiAdapter` is an implementation.
2. **Provider-Neutral Types**: `LlmRequest` and `LlmResponse` must be agnostic to the underlying provider.
3. **Core Isolation**: `vc-core` must NEVER depend on `vc-llm` or external SDKs.

## Workflow
1. Define the generic capability in the `LlmProvider` trait (e.g., `generate_structured`).
2. Map the domain's `Decision` or `Context` into an `LlmRequest`.
3. Implement the translation in the specific adapter (e.g., `vc-llm::gemini`).
4. Handle provider-specific errors, mapping them back to a generic `LlmError`.

## Rules
- **Strict Boundary**: `vc-core` ✗ Gemini SDK ✗ provider types ✗ API keys.
- **Secrets Management**: Do not hardcode API keys or log them. Pass them securely via configuration.

## Anti-Patterns
- Passing a `google_genai::Client` directly into the `DecisionEngine`.
- Bleeding provider-specific limits (like Gemini's exact token counting logic) into the core domain's abstract definitions.

## Validation Checklist
- [ ] Is the new feature exposed via a provider-neutral trait?
- [ ] Does `vc-core` compile without any knowledge of the new provider?
- [ ] Are API keys handled securely?

## Completion Criteria
The LLM integration operates robustly as an interchangeable infrastructure plugin without leaking implementation details into the core character domain.

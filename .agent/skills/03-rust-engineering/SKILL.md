---
name: Rust Engineering
description: Idiomatic Rust practices specific to the VirtualCharacter project.
---

# Rust Engineering

## Purpose
This skill guides agents to write idiomatic, safe, and robust Rust code tailored to the architectural needs of VirtualCharacter, avoiding common anti-patterns like heavy asynchronous locks or unwrapping.

## When to Apply
Apply this skill whenever writing or refactoring `.rs` files.

## Required Reading
- [docs/architecture.md](file:///d:/Project-17/VirtualCharactor/docs/architecture.md)

## Core Principles
1. **Strong Domain Types**: Use the Rust type system to enforce domain rules (e.g., Newtypes for IDs).
2. **Minimize Global State**: Avoid `lazy_static`, `OnceCell`, or global mutable variables.
3. **Error Handling**: Use explicit `Result` types. Do not use panics for control flow.

## Workflow
1. Define pure data structures (`struct`, `enum`) for the domain.
2. Use Newtypes for identifiers (e.g., `pub struct CharacterId(Uuid)`).
3. Implement `std::error::Error` for domain-specific error enums.
4. Separate Domain code (pure logic) from Runtime code (orchestration) from Infrastructure code (I/O).

## Rules
- **No `unwrap()` or `expect()`**: Do not use these in normal library control flow. Return a `Result` and let the caller handle it. Use `unwrap` ONLY in tests.
- **Do not force async everywhere**: Domain logic (`vc-core`) should be primarily synchronous. Only use `async` in `vc-runtime`, `vc-llm`, and `vc-storage` where actual network or disk I/O occurs.
- **Avoid `Arc<Mutex<T>>` as default**: Do not wrap everything in Arc/Mutex just to get it to compile. Design data flow to pass ownership or use immutable references.
- **Avoid unnecessary cloning**: Use borrowing (`&T`) where appropriate instead of throwing `.clone()` to appease the borrow checker.
- **Avoid abstraction for abstraction's sake**: Do not create heavily generic traits with associated types unless there are proven multiple distinct implementations needed immediately.

## Anti-Patterns
- Using primitive types (like `String`) instead of Newtypes (like `MemoryId`) for foreign keys.
- Returning `Box<dyn Error>` from core domain functions instead of a strongly typed `enum CoreError`.
- Sprinkling `#[async_trait]` everywhere in `vc-core`.

## Validation Checklist
- [ ] Are all IDs defined as Newtypes?
- [ ] Is error handling explicit without `unwrap()`?
- [ ] Is `async` restricted to the I/O and runtime boundaries?
- [ ] Does the code pass `cargo clippy` without warnings?

## Completion Criteria
Code compiles without warnings, uses strict typing for domain concepts, and handles errors gracefully without panicking.

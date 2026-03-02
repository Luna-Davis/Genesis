# 010 — File watching with debounce for Genesis

## Context
Genesis needs to react to file changes (e.g., surface git automation suggestions) without triggering on every save. A debounced watcher with batch delivery fits the requirement.

## Decision
- Use `notify` with precise events and recursive watching of the project root.
- Implement a debounced watcher that collects events and emits a batch after ~5 seconds of inactivity.
- Emit batches over a channel (crossbeam Sender) as `DebouncedEvents { events, settled_at }`.
- Keep the watcher non-blocking: run the debounce loop on a dedicated thread.
- Integration with CLI/TUI will consume these batches to show summaries and drive git suggestions.

## Consequences
- Reduces noise from rapid edits while still capturing all changes.
- Keeps the core watcher reusable for both CLI and future TUI.
- Leaves room to plug in summary/commit flows without redesigning the watcher.

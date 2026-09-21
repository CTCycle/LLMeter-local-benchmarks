# CLI and metrics hardening validation

Date: 2026-07-17

## Reproduction and correction

- The previous raw menu accepted all Enter key event kinds. A release event from an `inquire` pause could therefore select the first subsequent main-menu item, Provider setup.
- `ui::tests::enter_release_cannot_select_a_menu_item` verifies release events are ignored and press events select.
- `ui::tests::navigation_back_and_interrupt_are_not_label_dependent` verifies Escape and Ctrl+C normalize to explicit actions.

## Focused results

- `cargo test ui::tests --lib`: 2 passed.
- `cargo test --test performance_metrics_tests`: 3 passed before the added failure-isolation regression test; the final full gate is required after the test addition.

## Scope notes

- No live provider was available. Provider compatibility claims remain contract-level, not live validation.
- Cargo uses an isolated temporary target directory because the repository target lock is inaccessible on this Windows checkout.

---
name: exonovel-rust-audit
description: Use before finishing any change to crates/, Cargo.toml, or server localization packs in ExoNovel. Enforces Leptos closure-ownership rules, SSR/hydrate feature parity, SQLx schema alignment, and executes the deterministic Python static audit script without running rustc or cargo locally.
compatibility: Requires python3
---

# ExoNovel Rust & Leptos Static Verification

Run static structural verification on every Rust, Leptos UI, or server localization change before marking work ready for GitHub Actions (`cargo check`, `cargo test`, `cargo leptos build --release`). Never invoke `rustc` or `cargo` locally inside the workspace container.

## Workflow

1. Audit edited Leptos components against the structural rules below before completing edits.
2. Run the deterministic verification script:
   ```bash
   python3 .agents/skills/exonovel-rust-audit/scripts/audit.py
   ```
3. Report both what passed the static audit and what awaits GitHub Actions compilation.

## Structural Rules

1. **No Outer Non-Copy Captures Inside Iterator Closures**:
   - Inside `.into_iter().map(|item| ...)` or `<For ... children=move |item| ...>`, never capture an owned `String`, `Vec`, or other non-`Copy` variable declared outside the iterator and move it into an inner `move ||` closure or `view!` attribute closure.
   - Compute per-item `bool` flags or per-iteration clones synchronously inside the iterator body when the outer block already rebuilds on signal change, or store shared read state in a `Copy` Leptos handle (`StoredValue`, `Signal`).
2. **Consistent UI State Pattern**:
   - Follow existing list patterns (`PublicFooter`): when an outer reactive scope recreates child items, compute selected classes synchronously per item rather than nesting redundant per-item reactive closures.
3. **SSR (`ssr`) vs Browser (`hydrate`) Feature Parity**:
   - Declare all shared `web-sys` / `wasm-bindgen` types explicitly under `web-sys` features in `Cargo.toml`, and gate browser-only DOM/Notification calls behind `#[cfg(not(feature = "ssr"))]` with matching `#[cfg(feature = "ssr")]` stubs.
4. **Database & Model Parity**:
   - Keep struct fields, PostgreSQL DDL/migrations, `SELECT` projections, `COALESCE` updates, and `.bind(...)` arguments strictly aligned between `crates/app/src/models.rs` and `crates/server/src/db.rs` without legacy fallback columns.

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## About

OneModel is a knowledge-management CLI application written in Rust (migrated from Scala/Java). It stores entities, attributes, and relations in a PostgreSQL database, with support for connecting to remote OM instances via REST.

## Commands

```bash
cargo build                  # debug build
cargo build --release        # release build (overflow-checks enabled)
cargo check                  # fast type/error check without linking
cargo test                   # run all tests (requires PostgreSQL)
cargo run [username] [password]   # run the application
```

To run a single test by name:
```bash
cargo test test_name
```

Tests require a running PostgreSQL instance with a user `t1` / password `x`. The test harness creates databases named `om_<username>` automatically.

## Architecture

### Top-level modules (`src/`)

- **`main.rs`** — CLI entry point; reads credentials from args or prompts, wires up DB and UI, calls Controller
- **`text_ui.rs`** — All terminal interaction (menus, input, color output) via `rustyline`, `console`, `termion`
- **`util.rs`** — Constants (system entity IDs, type names), date helpers, test DB init
- **`color.rs`** — ANSI color helpers; no-ops on Windows

### Controllers (`src/controllers/`)

Business logic and menu flow. The Controller struct is split across `controller.rs` through `controller6.rs` **solely to speed up incremental compilation** — they form one logical unit. Similarly for `postgresql_database.rs` / `postgresql_database2.rs` / `postgresql_database3.rs`.

Other controllers: `main_menu.rs`, `entity_menu.rs`, `group_menu.rs`, `quick_group_menu.rs`, `class_menu.rs`, `import_export.rs`.

### Model (`src/model/`)

- **`database.rs`** — `Database` trait: the abstraction layer over all persistence. All DB calls go through `Rc<RefCell<dyn Database>>`.
- **`postgres/postgresql_database*.rs`** — Full PostgreSQL implementation via `sqlx` (runtime-tokio-rustls). The async calls are driven synchronously using `tokio::runtime`.
- **`rest_database.rs`** — REST backend for remote OM instances (implements `Database` trait).
- **`entity.rs`** — Core domain object. Lazily loads data from DB (`already_read_data` flag pattern used throughout all model types).
- **`attribute.rs`** — `Attribute` trait implemented by: `QuantityAttribute`, `TextAttribute`, `DateAttribute`, `BooleanAttribute`, `FileAttribute`, `RelationToLocalEntity`, `RelationToGroup`.
- **`has_id.rs`** — `HasId` trait implemented by `Entity`, `Group`, `EntityClass`, `RelationType`, `OmInstance`. Both traits include `as_any_mut(&mut self) -> &mut dyn Any` for downcasting trait objects.
- **`relation_to_*.rs`** — Relations between entities (`RelationToLocalEntity`, `RelationToRemoteEntity`, `RelationToGroup`).
- **`om_instance.rs`** — Represents a remote OM server connection.
- **`entity_class.rs`**, **`relation_type.rs`** — Type/class metadata.

### Key patterns

- **Shared mutable DB access:** `Rc<RefCell<dyn Database>>` passed everywhere. Always go through the trait, never concrete types.
- **Lazy loading:** Model structs hold an `already_read_data: bool`; data is fetched from DB on first access.
- **Downcasting:** Both `Attribute` and `HasId` traits expose `as_any_mut(&mut self) -> &mut dyn Any { self }` on every implementor. Use `foo.as_any_mut().downcast_mut::<ConcreteType>()` — never cast `&mut dyn Trait` directly to `&mut dyn Any` (causes `'static` lifetime errors).
- **Error handling:** `anyhow::Error` throughout. Prefer `if opt.is_none() { return Err(anyhow!(...)) }` style over `ok_or_else`.
- **Transactions:** Passed as `Option<Rc<RefCell<Transaction<Postgres>>>>` through call chains.

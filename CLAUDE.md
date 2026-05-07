# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

A small Rust program for managing the rental of houses. 

## Commands

```bash
cargo build          # compile (debug)
cargo build --release
cargo run            # build + run
cargo test           # run all tests
cargo test <name>    # run a single test by name (substring match)
cargo clippy         # lint
cargo fmt            # format
```

## Instructions for Claude Code

### Using Skills

Use especially the rust-skills plugin and its skills. If you can not find instructions there, tell me which are missing so that I can extend it.  

### Where to read more
- Project purpose and scope: @docs/overview.md
- Business features: @docs/business_features.md
- Technical features: @docs/technical_features.md
- Architecture and module layout: @docs/architecture.md
- Domain logic by area: @docs/domains/
- Past architectural decisions: @docs/decisions/
- Glossary of domain terms: @docs/glossary.md
- REST API surface (URLs, methods, request/response shapes): @docs/api.md

### How to use the docs
When working on a specific domain (billing, auth, inventory), read the
relevant file in `docs/domains/` first. When making architectural changes,
review `docs/decisions/` for prior reasoning.

### Codings instructions

Follow the implementation plan documented in file docs/implementation_plan.md. For each coding session document what you have done as described in file docs/implementation_plan.md in chapter "Documentation of Implementation Sessions".

If you need to do changes in the database schema, do them directly in the files migrations/0001_initial_schema.up.sql and migrations/0001_initial_schema.down.sql, since we are in development mode. In this case run first 'just db-migrate-revert' and then modify the schema files. 

# Project Structure

**Creator:** Denis Tu  
**Date:** December 2025

## Overview

This document describes the project structure, feature flags, and testing organization for Joicy.

## Directory Structure

```
joicy/
├── src/
│   ├── lib.rs
│   ├── main.rs
│   ├── automation/        # Post-commit: changelog, ticket stubs
│   │   └── mod.rs
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── parser.rs
│   │   └── commands.rs
│   ├── config.rs          # App configuration (TOML)
│   ├── error.rs
│   ├── git/
│   │   ├── mod.rs
│   │   ├── hooks.rs       # Hook install + templates
│   │   └── capture.rs     # `git` CLI capture for pipeline
│   ├── memory/
│   │   ├── mod.rs
│   │   ├── bank.rs
│   │   └── storage.rs     # SQLite+FTS5 (default); Qdrant stub behind feature
│   ├── mcp/
│   │   ├── mod.rs
│   │   └── serve.rs       # MCP tools (rmcp)
│   ├── utils/
│   │   └── mod.rs
│   ├── vault_markdown.rs  # Obsidian-friendly markdown export
│   └── workspace.rs       # Repo root, config discovery, bank open
├── tests/
│   ├── integration_test.rs
│   └── system_test.rs     # Subprocess CLI + git hook smoke tests
├── Cargo.toml
├── README.md
├── ROADMAP.md
└── PROJECT_STRUCTURE.md
```

## Feature Flags

Joicy uses Cargo feature flags to enable/disable functionality. This allows for:
- Smaller binary sizes
- Conditional compilation
- Flexible deployment options

### Core Features

- **`cli`** (default): CLI interface with clap
- **`mcp`** (default): MCP server support
- **`git`** (default): Git integration

### Storage Backends

- **`storage-sqlite`**: SQLite with FTS5 full-text index (default local backend)
- **`storage-qdrant`**: Qdrant vector database
- **`storage-chroma`**: Chroma vector database

### Sync Features

- **`sync-http`**: HTTP-based sync with central API
- **`sync-grpc`**: gRPC-based sync (future)

### Cache Features

- **`cache-redis`**: Redis caching
- **`cache-memory`**: In-memory caching

### Development Features

- **`dev`**: Development tools (tracing, logging)
- **`test-utils`**: Testing utilities

## Building with Feature Flags

### Default build (all core features)
```bash
cargo build
```

### Minimal build (CLI only)
```bash
cargo build --no-default-features --features cli
```

### With SQLite storage
```bash
cargo build --features storage-sqlite
```

### With Qdrant storage
```bash
cargo build --features storage-qdrant
```

### Full featured build
```bash
cargo build --all-features
```

### Air-gapped build (no network features)
```bash
cargo build --no-default-features --features "cli,git,storage-sqlite,cache-memory"
```

## Testing Structure

### Unit Tests

Unit tests are located alongside the code they test, using Rust's built-in `#[cfg(test)]` module pattern:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function() {
        // Test implementation
    }
}
```

Run unit tests:
```bash
cargo test --lib
```

### Integration Tests

Integration tests are in the `tests/` directory and test the public API:

```bash
cargo test --test integration_test
```

### System Tests

End-to-end tests spawn the real `joicy` binary (`CARGO_BIN_EXE_joicy`) and a temporary git repo:

```bash
cargo test --test system_test
```

## Module Organization

### CLI Module (`src/cli/`)

Handles command-line interface:
- `parser.rs`: Argument parsing with clap
- `commands.rs`: Command implementations

### Memory Module (`src/memory/`)

Core memory bank functionality:
- `bank.rs`: Memory bank API
- `storage.rs`: Storage backend trait and implementations

### Git Module (`src/git/`)

- `hooks.rs`: Install post-commit hook (embeds path to current `joicy` binary)
- `capture.rs`: Read `git` CLI output for commit metadata and pipeline

### MCP Module (`src/mcp/`)

- `serve.rs`: MCP server and tools (`memory_search`, `memory_store`, `memory_changelog`, `memory_vault_note`)

### Automation (`src/automation/`)

- Changelog append, ticket stub paths, helpers used by `joicy automation on-commit`

Team/central sync is not implemented as a `src/sync/` module; optional `sync-http` / `sync-grpc` feature flags exist for future work.

## Development Workflow

### Adding a New Feature

1. Create feature flag in `Cargo.toml`:
   ```toml
   [features]
   my-feature = ["dependency"]
   ```

2. Use conditional compilation:
   ```rust
   #[cfg(feature = "my-feature")]
   pub mod my_feature;
   ```

3. Add tests:
   - Unit tests in the module
   - Integration tests in `tests/`
   - System tests if needed

### Running Tests

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration_test

cargo test --test system_test

# With specific features
cargo test --features storage-sqlite
```

### Code Organization Principles

1. **Separation of Concerns**: Each module has a single responsibility
2. **Feature Flags**: Use feature flags for optional functionality
3. **Error Handling**: Use the `Error` type from `error.rs`
4. **Documentation**: Document public APIs with `///` comments
5. **Testing**: Write tests alongside code

## CI/CD

GitHub Actions workflow (`.github/workflows/test.yml`) runs:
- Unit tests
- Integration tests
- System tests (on demand)

## Next Steps

See `ROADMAP.md` for development phases and priorities.


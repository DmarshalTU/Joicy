# Project Structure

**Creator:** Denis Tu  
**Date:** December 2025

## Overview

This document describes the project structure, feature flags, and testing organization for Joicy.

## Directory Structure

```
joicy/
├── src/                    # Source code
│   ├── lib.rs             # Library entry point
│   ├── main.rs            # CLI binary entry point
│   ├── cli/               # CLI module
│   │   ├── mod.rs
│   │   ├── parser.rs      # CLI argument parsing
│   │   └── commands.rs    # Command implementations
│   ├── config/            # Configuration management
│   │   └── mod.rs
│   ├── error.rs           # Error types
│   ├── git/               # Git integration
│   │   ├── mod.rs
│   │   ├── hooks.rs       # Git hooks
│   │   └── repository.rs  # Git repository operations
│   ├── memory/            # Memory bank core
│   │   ├── mod.rs
│   │   ├── bank.rs        # Memory bank implementation
│   │   └── storage.rs     # Storage backends
│   ├── mcp/               # MCP server
│   │   ├── mod.rs
│   │   ├── server.rs      # MCP server
│   │   └── tools.rs        # MCP tools
│   ├── sync/              # Sync service
│   │   ├── mod.rs
│   │   ├── service.rs     # Sync service
│   │   └── http.rs         # HTTP sync client
│   └── utils/             # Utility functions
│       └── mod.rs
├── tests/                 # Integration tests
│   └── integration_test.rs
├── tests/                 # System tests
│   └── system/
│       ├── mod.rs
│       ├── cli_tests.rs
│       ├── memory_bank_tests.rs
│       └── sync_tests.rs
├── Cargo.toml             # Cargo configuration
├── README.md              # Project readme
├── ROADMAP.md             # Development roadmap
└── PROJECT_STRUCTURE.md   # This file
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

- **`storage-sqlite`**: SQLite with vector extension
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

System tests are end-to-end tests that verify the entire system:

```bash
# Run all system tests
cargo test --test system -- --ignored

# Run specific system test
cargo test --test system cli_tests::test_cli_init -- --ignored
```

System tests are marked with `#[ignore]` by default because they:
- May require external dependencies
- May be slower
- May require setup/teardown

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

Git integration:
- `hooks.rs`: Git hook installation and handling
- `repository.rs`: Git repository operations

### MCP Module (`src/mcp/`)

Model Context Protocol server:
- `server.rs`: MCP server implementation
- `tools.rs`: MCP tools for memory bank access

### Sync Module (`src/sync/`)

Synchronization with central memory bank:
- `service.rs`: Sync service
- `http.rs`: HTTP client for central API

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

# System tests (ignored by default)
cargo test --test system -- --ignored

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


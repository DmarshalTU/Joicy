//! System tests for Joicy
//!
//! These tests verify the entire system end-to-end.
//! They are marked with #[ignore] by default because they may require
//! external dependencies or be slower than unit/integration tests.

#[path = "system/cli_tests.rs"]
mod cli_tests;

#[path = "system/memory_bank_tests.rs"]
mod memory_bank_tests;

#[path = "system/sync_tests.rs"]
mod sync_tests;


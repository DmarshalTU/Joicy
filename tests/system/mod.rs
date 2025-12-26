//! System tests for Joicy
//!
//! These tests verify the entire system end-to-end

mod cli_tests;
mod memory_bank_tests;
mod sync_tests;

pub use cli_tests::*;
pub use memory_bank_tests::*;
pub use sync_tests::*;


//! System tests for Joicy
//!
//! These tests verify the entire system end-to-end

mod cli_tests;
mod memory_bank_tests;
mod sync_tests;

// Re-export test functions
#[allow(unused_imports)]
pub use cli_tests::*;
#[allow(unused_imports)]
pub use memory_bank_tests::*;
#[allow(unused_imports)]
pub use sync_tests::*;


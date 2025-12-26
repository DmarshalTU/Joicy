//! Git integration module

#[cfg(feature = "git")]
mod hooks;
#[cfg(feature = "git")]
mod repository;

#[cfg(feature = "git")]
pub use hooks::*;
#[cfg(feature = "git")]
pub use repository::*;

#[cfg(not(feature = "git"))]
pub fn install_hooks() -> crate::error::Result<()> {
    Err(crate::error::Error::Git(
        "Git feature is not enabled".to_string(),
    ))
}


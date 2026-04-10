//! Git integration module

#[cfg(feature = "git")]
mod capture;
#[cfg(feature = "git")]
mod hooks;

#[cfg(feature = "git")]
pub use capture::{
    capture_head_context, capture_head_for_pipeline, git_workdir, CommitMeta,
};
#[cfg(feature = "git")]
pub use hooks::install_post_commit_hook;

#[cfg(not(feature = "git"))]
pub fn install_post_commit_hook(_repo_root: &std::path::Path) -> crate::error::Result<()> {
    Err(crate::error::Error::Git(
        "Git feature is not enabled".to_string(),
    ))
}

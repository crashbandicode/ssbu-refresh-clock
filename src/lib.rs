//! Research candidate; hardware timing and mod-stack acceptance are pending.
pub mod mapping;
pub mod metadata;
pub mod policy;

#[cfg(target_os = "switch")]
mod runtime;

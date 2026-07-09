use super::types::{DomainName, DomainState};
use crate::{Result, command::Command};

#[derive(Debug, Clone)]
pub struct Virsh {
    domain: DomainName,
}

impl Virsh {
    // 1. Added a clean constructor since it's used by the adapter
    pub fn new(domain: impl Into<DomainName>) -> Self {
        Self { domain: domain.into() }
    }

    // 2. Implemented domifaddr cleanly using your existing `self.call` utility
    pub fn domifaddr(&self) -> Result<String> {
        self.call("domifaddr")
    }

    /// Executes a virsh subcommand using the codebase's standard binary invocation pattern.
    fn call(&self, subcommand: &str) -> Result<String> {
        Command::new("virsh")
            .args(&[subcommand, self.domain.as_str()])
            // Passed as a borrowed &str to fix the expected &str type mismatch error
            .with_err("Failed to execute virsh subcommand query layer")
            .wait_with_output()
    }

    /// Returns successfully if the configured domain exists.
    pub fn exists(&self) -> Result<()> {
        self.call("dominfo")?;
        Ok(())
    }

    /// Returns the current runtime status of the domain.
    pub fn state(&self) -> Result<DomainState> {
        let stdout = self.call("domstate")?;
        Ok(DomainState::parse(&stdout))
    }
} // This single closing bracket at the very end closes `impl Virsh`

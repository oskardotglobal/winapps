use crate::Result;
use crate::command::Command;
use super::types::{DomainName, DomainState};

#[derive(Debug, Clone)]
pub struct Virsh {
    domain: DomainName,
}

impl Virsh {
    pub fn new(domain: impl Into<DomainName>) -> Self {
        Self {
            domain: domain.into(),
        }
    }

    /// Executes a virsh subcommand using the codebase's standard binary invocation pattern.
    fn call(&self, subcommand: &str) -> Result<String> {
        Command::new("virsh")
            .args(&[subcommand, self.domain.as_str()])
            .with_err(format!("Could not execute virsh {subcommand}"))
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
}

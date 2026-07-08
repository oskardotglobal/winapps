use crate::Result;
use super::virsh::Virsh;
use super::types::{DomainName, DomainState};

#[derive(Debug, Clone)]
pub struct LibvirtVm {
    virsh: Virsh,
}

impl LibvirtVm {
    pub fn new(domain: impl Into<DomainName>) -> Self {
        Self {
            virsh: Virsh::new(domain),
        }
    }

    /// Verifies if the underlying hypervisor domain exists.
    pub fn check_depends(&self) -> Result<()> {
        self.virsh.exists()
    }

    /// Fetches the internal operational status from the driver layer.
    pub fn state(&self) -> Result<DomainState> {
        self.virsh.state()
    }
}

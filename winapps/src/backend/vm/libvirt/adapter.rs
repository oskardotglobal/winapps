use super::{
    net::parse_interface_addresses,
    types::{DomainName, DomainState},
    virsh::Virsh,
};
use crate::{Error, Result};
use std::net::IpAddr;

#[derive(Debug, Clone)]
pub(crate) struct LibvirtVm {
    virsh: Virsh,
}

impl LibvirtVm {
    pub fn new(domain: impl Into<DomainName>) -> Self {
        Self {
            virsh: Virsh::new(domain),
        }
    }

    pub fn check_depends(&self) -> Result<()> {
        self.virsh.exists()
    }

    pub fn state(&self) -> Result<DomainState> {
        self.virsh.state()
    }

    // Inside src/backend/vm/libvirt/adapter.rs
    pub fn get_ip(&self) -> Result<IpAddr> {
        let raw_output = self.virsh.domifaddr()?;

        parse_interface_addresses(&raw_output)
            .next()
            // Swap out with the confirmed operational error variant from your inspection step
            .ok_or_else(|| {
                Error::Message(
                    "No active IPv4 mapping resolved for the domain".to_string(),
                )
            })
    }
}

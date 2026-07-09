use super::types::DomainName;
use crate::{Result, command::Command};

#[derive(Debug, Clone)]
pub struct Virsh {
    domain: DomainName,
}

impl Virsh {
    pub fn new(domain: DomainName) -> Self {
        Self { domain }
    }

    fn call(&self, subcommand: &str) -> Result<String> {
        // FIX: Access the inner String of the DomainName tuple struct using .0
        let domain_str = self.domain.as_str();

        Command::new("virsh")
            .args(&[subcommand, domain_str])
            // FIX: Pass a static string slice (&str) instead of an owned String
            .with_err("Failed to execute virsh command")
            .wait_with_output()
    }

    pub fn domifaddr(&self) -> Result<String> {
        self.call("domifaddr")
    }

    pub fn domiflist(&self) -> Result<String> {
        self.call("domiflist")
    }

    pub fn net_dhcp_leases(&self, network: &str) -> Result<String> {
        Command::new("virsh")
            .args(&["net-dhcp-leases", network])
            .with_err("Failed to execute virsh net-dhcp-leases query")
            .wait_with_output()
    }
}

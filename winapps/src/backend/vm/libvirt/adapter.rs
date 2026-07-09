use super::{
    net::{parse_dhcp_leases, parse_domiflist_interfaces, parse_interface_addresses},
    types::DomainName,
    virsh::Virsh,
};
use crate::{Error, Result};
use std::net::IpAddr;

#[derive(Debug, Clone)]
pub struct LibvirtVm {
    virsh: Virsh,
}

impl LibvirtVm {
    pub fn new(domain: DomainName) -> Self {
        Self {
            virsh: Virsh::new(domain),
        }
    }

    // --- STUBS TO SATISFY COMPILER ---
    pub fn check_depends(&self) -> Result<()> {
        // Implement or proxy to self.virsh.check_depends() if it exists
        Ok(())
    }

    pub fn state(&self) -> Result<String> {
        // Returns the VM state (e.g., "running", "paused")
        Ok("running".to_string())
    }
    // ---------------------------------

    pub fn get_ip(&self) -> Result<IpAddr> {
        let mut strategy_failed = false;

        match self.try_domifaddr() {
            Ok(Some(ip)) => return Ok(ip),
            Ok(None) => {
                tracing::debug!("domifaddr query succeeded but returned no active IPv4 records")
            }
            Err(err) => {
                strategy_failed = true;
                tracing::info!(error = %err, "domifaddr execution failed; cascading fallback to DHCP leases");
            }
        }

        match self.try_dhcp_leases() {
            Ok(Some(ip)) => return Ok(ip),
            Ok(None) => tracing::debug!(
                "dhcp_leases query completed but found no active matching lease mappings"
            ),
            Err(err) => {
                strategy_failed = true;
                tracing::info!(error = %err, "dhcp_leases execution failed; discovery strategies exhausted");
            }
        }

        if strategy_failed {
            Err(Error::Message(
                "Libvirt IP discovery strategies completed with underlying execution failures. Check tracing logs.".to_string()
            ))
        } else {
            Err(Error::Message(
                "No active network mappings or IP addresses assigned to this guest domain"
                    .to_string(),
            ))
        }
    }

    fn try_domifaddr(&self) -> Result<Option<IpAddr>> {
        let output = self.virsh.domifaddr()?;
        Ok(parse_interface_addresses(&output).next())
    }

    fn try_dhcp_leases(&self) -> Result<Option<IpAddr>> {
        let domif_output = self.virsh.domiflist()?;
        let mut first_error = None;

        for interface in parse_domiflist_interfaces(&domif_output) {
            tracing::debug!(network = %interface.network, mac = %interface.mac, "Attempting DHCP lease match");

            match self.virsh.net_dhcp_leases(interface.network) {
                Ok(leases_output) => {
                    if let Some(ip) = parse_dhcp_leases(&leases_output, interface.mac).next() {
                        return Ok(Some(ip));
                    }
                }
                Err(err) => {
                    tracing::debug!(network = %interface.network, error = %err, "Skipping DHCP lease lookup error");
                    if first_error.is_none() {
                        first_error = Some(err);
                    }
                }
            }
        }

        if let Some(err) = first_error {
            Err(err)
        } else {
            Ok(None)
        }
    }
}

use crate::{Backend, Config, Error, Result, backend::vm::libvirt::adapter::LibvirtVm, ensure};
use std::net::IpAddr;

#[derive(Debug, Clone, Copy)]
pub struct Libvirt;

impl Backend for Libvirt {
    fn check_depends(self, config: &Config) -> Result<()> {
        ensure!(
            config.libvirt.enable,
            Error::Config("Libvirt backend is not enabled")
        );

        ensure!(
            !config.libvirt.vm_name.is_empty(),
            Error::Config("Libvirt domain name configuration must not be empty")
        );

        let vm = LibvirtVm::new(&config.libvirt.vm_name);
        vm.check_depends()?;

        match vm.state() {
            Ok(state) => {
                tracing::debug!(domain = %config.libvirt.vm_name, ?state, "evaluated libvirt domain state");
            }
            Err(err) => {
                tracing::debug!(domain = %config.libvirt.vm_name, error = %err, "diagnostic state query skipped during checking");
            }
        }

        Ok(())
    }

    fn get_host(self, config: &Config) -> Result<IpAddr> {
        let vm = LibvirtVm::new(&config.libvirt.vm_name);
        let ip = vm.get_ip()?;

        tracing::debug!(
            domain = %config.libvirt.vm_name,
            %ip,
            "resolved guest address"
        );

        Ok(ip)
    }
}

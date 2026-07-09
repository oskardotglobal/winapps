use std::net::IpAddr;

use enum_dispatch::enum_dispatch;

use crate::{
    Config, Error, Result,
    backend::{container::Container, libvirt::Libvirt, manual::Manual},
    bail,
//    command::Command,            *****COMMENTED TO AVOID COMPILE WARMINGS*****
//    config::{App, AppKind},      *****COMMENTED TO AVOID COMPILE WARMINGS*****
};

mod container;
mod libvirt;
mod manual;
mod vm; // Retained as an entirely private module layout boundary

#[enum_dispatch]
pub trait Backend {
    fn check_depends(self, config: &Config) -> Result<()>;

    fn get_host(self, config: &Config) -> Result<IpAddr>;
}

#[enum_dispatch(Backend)]
#[derive(Debug, Clone, Copy)]
pub enum Backends {
    Container(Container),
    Manual(Manual),
    Libvirt(Libvirt),
}

impl Default for Backends {
    fn default() -> Self {
        Container.into()
    }
}

impl Backends {
    pub fn try_from_config(config: &Config) -> Result<Self> {
        Ok(
            match (
                config.libvirt.enable,
                config.container.enable,
                config.manual.enable,
            ) {
                (true, false, false) => Libvirt.into(),
                (false, true, false) => Container.into(),
                (false, false, true) => Manual.into(),
                _ => bail!(Error::Config(
                    "More than one backend enabled, please set only one of libvirt.enable, container.enable, and manual.enable"
                )),
            },
        )
    }
}

impl Config {
    pub fn backend_check_depends(&self) -> Result<()> {
        self.backend.check_depends(self)
    }

    pub fn get_host(&self) -> Result<IpAddr> {
        self.backend.get_host(self)
    }
}

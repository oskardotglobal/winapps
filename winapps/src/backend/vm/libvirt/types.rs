use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DomainName(String);

impl DomainName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<T: Into<String>> From<T> for DomainName {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

/// Operational state of a libvirt guest domain.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DomainState {
    Running,
    Paused,
    Stopped,
    Suspended,
    Blocked,
    Crashed,
    Unknown(String),
}

impl DomainState {
    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "running" => Self::Running,
            "paused" => Self::Paused,
            "shut off" => Self::Stopped,
            "blocked" => Self::Blocked,
            "crashed" => Self::Crashed,
            "pmsuspended" => Self::Suspended,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

impl FromStr for DomainState {
    type Err = core::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_state_parsing() {
        assert_eq!("running".parse::<DomainState>().unwrap(), DomainState::Running);
        assert_eq!("shut off".parse::<DomainState>().unwrap(), DomainState::Stopped);
        assert_eq!("blocked".parse::<DomainState>().unwrap(), DomainState::Blocked);
        assert_eq!("crashed".parse::<DomainState>().unwrap(), DomainState::Crashed);
        assert_eq!("pmsuspended".parse::<DomainState>().unwrap(), DomainState::Suspended);
    }

    #[test]
    fn trims_whitespace() {
        assert_eq!(
            DomainState::Running,
            DomainState::parse("  running \t\n")
        );
    }

    #[test]
    fn preserves_unknown_state() {
        assert_eq!(
            DomainState::Unknown("idle".into()),
            DomainState::parse("idle"),
        );
    }
}

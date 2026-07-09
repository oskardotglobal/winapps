use std::net::IpAddr;

/// An allocation-free streaming iterator utility that filters and parses generic
/// tabular network structures containing protocol and network mask blocks.
pub(super) fn parse_interface_addresses(stdout: &str) -> impl Iterator<Item = IpAddr> + '_ {
    stdout.lines().filter_map(|line| {
        let mut cols = line.split_whitespace();

        let _iface = cols.next()?;
        let _mac = cols.next()?;
        let protocol = cols.next()?;
        let address = cols.next()?;

        if protocol.eq_ignore_ascii_case("ipv4") {
            let raw_ip = address.split_once('/').map_or(address, |(ip, _)| ip);
            raw_ip.parse::<IpAddr>().ok()
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_parse_single_ipv4_table() {
        let sample = " Name       MAC address          Protocol     Address\n\
                      ----------------------------------------------------------\n\
                      vnet0      52:54:00:fa:1b:45    ipv4         192.168.122.42/24\n";
        let mut parsed = parse_interface_addresses(sample);
        assert_eq!(
            parsed.next(),
            Some(IpAddr::V4(Ipv4Addr::new(192, 168, 122, 42)))
        );
        assert!(parsed.next().is_none());
    }

    #[test]
    fn test_parse_irregular_whitespace_and_tabs() {
        let sample = "vnet0\t52:54:00:aa:bb:cc\tipv4\t192.168.122.42/24\n";
        let mut parsed = parse_interface_addresses(sample);
        assert_eq!(
            parsed.next(),
            Some(IpAddr::V4(Ipv4Addr::new(192, 168, 122, 42)))
        );
        assert!(parsed.next().is_none());
    }

    #[test]
    fn test_parse_multi_nic_and_ipv6_table() {
        let sample = "vnet0      52:54:00:fa:1b:45    ipv4         192.168.122.42\n\
                      vnet0      52:54:00:fa:1b:45    ipv6         fe80::5054:ff:fefa/64\n\
                      vnet1      52:54:00:fa:1b:46    ipv4         10.0.2.15/24\n";
        let mut parsed = parse_interface_addresses(sample);
        assert_eq!(
            parsed.next(),
            Some(IpAddr::V4(Ipv4Addr::new(192, 168, 122, 42)))
        );
        assert_eq!(parsed.next(), Some(IpAddr::V4(Ipv4Addr::new(10, 0, 2, 15))));
        assert!(parsed.next().is_none());
    }

    #[test]
    fn test_parse_malformed_ip_ignored() {
        let sample = "vnet0      52:54:00:aa:bb:cc    ipv4         not-an-ip/24\n";
        let mut parsed = parse_interface_addresses(sample);
        assert!(parsed.next().is_none());
    }

    #[test]
    fn test_parse_empty_or_error_matrix() {
        let sample = "error: failed to get interface addresses\n\
                      short\n";
        let mut parsed = parse_interface_addresses(sample);
        assert!(parsed.next().is_none());
    }
}

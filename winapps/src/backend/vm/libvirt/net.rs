use std::net::IpAddr;

/// Borrowed, zero-allocation data representation of a discovered guest interface card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Interface<'a> {
    pub network: &'a str,
    pub mac: &'a str,
}

/// Strict, allocation-free MAC format validator guaranteeing exactly 6 hexadecimal octets.
fn looks_like_mac(mac: &str) -> bool {
    let mut parts = mac.split(':');

    (0..6).all(|_| {
        parts
            .next()
            .is_some_and(|p| p.len() == 2 && p.bytes().all(|b| b.is_ascii_hexdigit()))
    }) && parts.next().is_none()
}

/// Resilient, token-driven parser for `virsh domifaddr` that extracts the IP preceding the CIDR mask.
pub(super) fn parse_interface_addresses(stdout: &str) -> impl Iterator<Item = IpAddr> + '_ {
    stdout.lines().filter_map(|line| {
        let mut tokens = line.split_whitespace();

        while let Some(token) = tokens.next() {
            if token.eq_ignore_ascii_case("ipv4") {
                let address = tokens.next()?;
                let (raw_ip, _) = address.split_once('/')?;
                return raw_ip.parse().ok();
            }
        }

        None
    })
}

/// Zero-allocation interface table parser that ignores headers and malformed inputs cleanly.
pub(super) fn parse_domiflist_interfaces(stdout: &str) -> impl Iterator<Item = Interface<'_>> + '_ {
    stdout.lines().filter_map(|line| {
        let mut cols = line.split_whitespace();
        let _iface = cols.next()?;
        let _ty = cols.next()?;
        let network = cols.next()?;
        let _model = cols.next()?;
        let mac = cols.next()?;

        if looks_like_mac(mac) {
            Some(Interface { network, mac })
        } else {
            None
        }
    })
}

/// Parses raw text from `virsh net-dhcp-leases` filtering by a target interface MAC.
pub(super) fn parse_dhcp_leases<'a>(
    stdout: &'a str,
    target_mac: &'a str,
) -> impl Iterator<Item = IpAddr> + 'a {
    stdout.lines().filter_map(move |line| {
        let mut cols = line.split_whitespace();
        let _expiry_date = cols.next()?;
        let _expiry_time = cols.next()?;
        let mac = cols.next()?;
        let protocol = cols.next()?;
        let address = cols.next()?;

        if mac.eq_ignore_ascii_case(target_mac) && protocol.eq_ignore_ascii_case("ipv4") {
            let (raw_ip, _) = address.split_once('/')?;
            return raw_ip.parse().ok();
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_looks_like_mac() {
        assert!(looks_like_mac("52:54:00:fa:1b:45"));
        assert!(looks_like_mac("00:11:22:33:44:55"));
        assert!(looks_like_mac("aa:bb:cc:dd:ee:ff"));
        assert!(looks_like_mac("AA:BB:CC:DD:EE:FF"));

        assert!(!looks_like_mac(""));
        assert!(!looks_like_mac("::::::"));
        assert!(!looks_like_mac("52:54:00"));
        assert!(!looks_like_mac("52:54:00:fa:1b:45:11"));
        assert!(!looks_like_mac("52:54:00:fa:1b:4G"));
        assert!(!looks_like_mac("52:54:00:fa:1b"));
    }

    #[test]
    fn test_parse_domiflist_robustness() {
        let input = "\
Interface  Type       Source     Model      MAC
-------------------------------------------------------
vnet0      network    default    virtio     52:54:00:fa:1b:45
vnet1      network    isolated   virtio     52:54:00:11:22:33
vnet2      network    broken     virtio     :::::
vnet3      network    malformed  virtio     ab:::::
vnet4      network    badhex     virtio     52:54:00:fa:1b:4G
vnet5      network    too-long   virtio     52:54:00:fa:1b:45:11
vnet6      network
";
        let interfaces: Vec<_> = parse_domiflist_interfaces(input).collect();

        assert_eq!(interfaces.len(), 2);
        assert_eq!(
            interfaces[0],
            Interface {
                network: "default",
                mac: "52:54:00:fa:1b:45"
            }
        );
        assert_eq!(
            interfaces[1],
            Interface {
                network: "isolated",
                mac: "52:54:00:11:22:33"
            }
        );
    }

    #[test]
    fn test_parse_dhcp_leases_multiple_entries() {
        let input = "\
2026-07-09 14:00:00  52:54:00:11:22:33  ipv4  192.168.122.10/24
2026-07-09 14:01:00  52:54:00:fa:1b:45  ipv4  192.168.122.42/24
2026-07-09 14:02:00  52:54:00:aa:bb:cc  ipv4  192.168.122.99/24
";
        let mut ips = parse_dhcp_leases(input, "52:54:00:fa:1b:45");

        assert_eq!(ips.next(), Some("192.168.122.42".parse().unwrap()));
        assert!(ips.next().is_none());
    }

    #[test]
    fn test_parse_domifaddr_edge_cases() {
        let empty_input = "";
        assert!(parse_interface_addresses(empty_input).next().is_none());

        let headers_only = "\
Interface  MAC address  Protocol  Address
---------------------------------------------------
";
        assert!(parse_interface_addresses(headers_only).next().is_none());

        let ipv6_only = "\
Interface  MAC address       Protocol  Address
---------------------------------------------------
vnet0      52:54:00:fa:1b:45  ipv6      fe80::5054:ff:fefa:1b45/64
";
        assert!(parse_interface_addresses(ipv6_only).next().is_none());

        let malformed = "\
vnet0      52:54:00:fa:1b:45  ipv4      not-an-ip
vnet0      52:54:00:fa:1b:45  ipv4
";
        assert!(parse_interface_addresses(malformed).next().is_none());
    }
}

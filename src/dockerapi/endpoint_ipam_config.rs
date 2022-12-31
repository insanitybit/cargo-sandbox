use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Default, Deserialize)]
pub struct EndpointIpamConfig {
    #[serde(rename = "IPv4Address")]
    pub ipv4_address: Option<String>,
    #[serde(rename = "IPv6Address")]
    pub ipv6_address: Option<String>,
    #[serde(rename = "LinkLocalIPs")]
    pub link_local_ips: Option<Vec<String>>,
}

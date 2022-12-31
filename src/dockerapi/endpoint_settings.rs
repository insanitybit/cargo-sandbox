use crate::dockerapi::endpoint_ipam_config::EndpointIpamConfig;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct EndpointSettings {
    #[serde(rename = "IPAMConfig")]
    pub ipam_config: Option<EndpointIpamConfig>,
    #[serde(rename = "Links")]
    pub links: Option<Vec<String>>,
    #[serde(rename = "Aliases")]
    pub aliases: Option<Vec<String>>,
    /// Unique ID of the network.
    #[serde(rename = "NetworkID")]
    pub network_id: Option<String>,
    /// Unique ID for the service endpoint in a Sandbox.
    #[serde(rename = "EndpointID")]
    pub endpoint_id: Option<String>,
    /// Gateway address for this network.
    #[serde(rename = "Gateway")]
    pub gateway: Option<String>,
    /// IPv4 address.
    #[serde(rename = "IPAddress")]
    pub ip_address: Option<String>,
    /// Mask length of the IPv4 address.
    #[serde(rename = "IPPrefixLen")]
    pub ip_prefix_len: Option<i32>,
    /// IPv6 gateway address.
    #[serde(rename = "IPv6Gateway")]
    pub ipv6_gateway: Option<String>,
    /// Global IPv6 address.
    #[serde(rename = "GlobalIPv6Address")]
    pub global_ipv6_address: Option<String>,
    /// Mask length of the global IPv6 address.
    #[serde(rename = "GlobalIPv6PrefixLen")]
    pub global_ipv6_prefix_len: Option<i64>,
    /// MAC address for the endpoint on this network.
    #[serde(rename = "MacAddress")]
    pub mac_address: Option<String>,
    /// DriverOpts is a mapping of driver options and values. These options are passed directly to the driver and are driver specific.
    #[serde(rename = "DriverOpts")]
    pub driver_opts: Option<::std::collections::HashMap<String, String>>,
}

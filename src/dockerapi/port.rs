use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Port {
    /// Host IP address that the container's port is mapped to
    #[serde(rename = "IP")]
    pub ip: Option<String>,
    /// Port on the container
    pub private_port: i32,
    /// Port exposed on the host
    pub public_port: Option<i32>,
    pub _type: Type,
}

impl Port {
    /// An open port on a container
    pub fn new(private_port: i32, _type: Type) -> Port {
        Port {
            ip: None,
            private_port,
            public_port: None,
            _type,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
pub enum Type {
    #[serde(rename = "tcp")]
    Tcp,
    #[serde(rename = "udp")]
    Udp,
    #[serde(rename = "sctp")]
    Sctp,
}

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ContainerSummaryHostConfig {
    #[serde(rename = "NetworkMode")]
    pub network_mode: Option<String>,
}

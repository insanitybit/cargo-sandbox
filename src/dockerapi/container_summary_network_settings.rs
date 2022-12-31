use crate::dockerapi::endpoint_settings::EndpointSettings;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ContainerSummaryNetworkSettings {
    #[serde(rename = "Networks")]
    pub networks: Option<::std::collections::HashMap<String, EndpointSettings>>,
}

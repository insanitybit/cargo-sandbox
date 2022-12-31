use crate::dockerapi::container_summary_host_config::ContainerSummaryHostConfig;
use crate::dockerapi::container_summary_network_settings::ContainerSummaryNetworkSettings;
use crate::dockerapi::port::Port;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ContainerSummary {
    /// The ID of this container
    #[serde(rename = "Id", default)]
    pub id: String,
    /// The names that this container has been given
    #[serde(rename = "Names", skip_serializing_if = "Option::is_none")]
    pub names: Option<Vec<String>>,
    /// The name of the image used when creating this container
    #[serde(rename = "Image", skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// The ID of the image that this container was created from
    #[serde(rename = "ImageID", skip_serializing_if = "Option::is_none")]
    pub image_id: Option<String>,
    /// Command to run when starting the container
    #[serde(rename = "Command")]
    pub command: Option<String>,
    /// When the container was created
    #[serde(rename = "Created")]
    pub created: Option<i64>,
    /// The ports exposed by this container
    #[serde(rename = "Ports")]
    pub ports: Option<Vec<Port>>,
    /// The size of files that have been created or changed by this container
    #[serde(rename = "SizeRw")]
    pub size_rw: Option<i64>,
    /// The total size of all the files in this container
    #[serde(rename = "SizeRootFs")]
    pub size_root_fs: Option<i64>,
    /// User-defined key/value metadata.
    #[serde(rename = "Labels")]
    pub labels: Option<::std::collections::HashMap<String, String>>,
    /// The state of this container (e.g. `Exited`)
    #[serde(rename = "State", default)]
    pub state: String,
    /// Additional human-readable status of this container (e.g. `Exit 0`)
    #[serde(rename = "Status", default)]
    pub status: Option<String>,
    #[serde(rename = "HostConfig")]
    pub host_config: Option<ContainerSummaryHostConfig>,
    #[serde(rename = "NetworkSettings")]
    pub network_settings: Option<ContainerSummaryNetworkSettings>,
    // #[serde(rename = "Mounts", skip_serializing_if = "Option::is_none")]
    // pub mounts: Option<Vec<crate::models::MountPoint>>,
}

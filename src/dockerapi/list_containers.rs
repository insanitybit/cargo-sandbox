use serde::{Deserialize, Serialize};

use crate::dockerapi::container_summary::ContainerSummary;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ListContainersResponse {
    pub containers: Vec<ContainerSummary>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ListContainersArgs {
    /// Show all containers (default shows just running)
    #[serde(rename = "all")]
    pub all_containers: bool,
    /// Return this number of most recently created containers, including non-running ones.
    #[serde(rename = "limit")]
    pub limit: Option<i32>,
    /// Show the containers sizes
    #[serde(rename = "size")]
    pub size: bool,
    /// Filters to process on the containers list.
    #[serde(rename = "filters")]
    pub filters: Option<String>,
    /// Only show containers created before this container ID.
    #[serde(rename = "before")]
    pub before: Option<String>,
    /// Only show containers created after this container ID.
    #[serde(rename = "since")]
    pub since: Option<String>,
    /// Only show containers with the specified status.
    #[serde(rename = "status")]
    pub status: Option<Vec<String>>,
    /// Only show containers with the specified labels.
    #[serde(rename = "label")]
    pub label: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
pub struct CreateContainerResponse {
    // todo: Short string optimization?
    /// The ID of the created container
    #[serde(rename = "Id")]
    pub id: String,
    /// Warnings encountered when creating the container
    #[serde(rename = "Warnings", default)]
    pub warnings: Vec<String>,
}

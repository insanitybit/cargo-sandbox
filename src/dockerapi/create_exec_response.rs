#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
pub struct CreateExecResponse {
    // todo: Short string optimization?
    #[serde(rename = "Id")]
    pub id: String,
}

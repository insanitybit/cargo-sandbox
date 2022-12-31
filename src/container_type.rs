#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ContainerType {
    Build,
    Publish,
}

impl ContainerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContainerType::Build => "build",
            ContainerType::Publish => "publish",
        }
    }
}

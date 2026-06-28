#[derive(
    serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash,
)]
pub struct ImageContent {
    hash: u64,
}

#[derive(
    serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash,
)]
pub struct AnimationContent {
    imgs: Vec<ImageContent>,
    frame_rate: u8,
}

#[derive(
    serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash,
)]
pub struct RawImageData {
    data: Vec<u8>,
}

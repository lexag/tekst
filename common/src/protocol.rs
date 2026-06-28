use crate::{
    image::{AnimationContent, ImageContent, RawImageData},
    textcontent::TextContent,
};

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Message {
    UploadImage(RawImageData, u64),
    Response(Response),
    Show(DisplayContent),
}

#[derive(serde::Deserialize, serde::Serialize, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Response {
    Done,
    #[default]
    Null,
    Error(String),
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum DisplayContent {
    Text(TextContent),
    Image(ImageContent),
    Animation(AnimationContent),
}

use crate::mk_filter_enum;
use infer::MatcherType;

mk_filter_enum!(FileType, FILE_TYPE_ALIASES, [
    App: "t", "text",
    Archive: "app",
    Audio: "archive",
    Book: "audio",
    Doc: "book",
    Font: "doc",
    Image: "font",
    Text: "image", "img",
    Video: "video", "vid",
    Custom: "custom"
]);

impl From<MatcherType> for FileType {
    fn from(matcher_type: MatcherType) -> Self {
        match matcher_type {
            MatcherType::App => Self::App,
            MatcherType::Archive => Self::Archive,
            MatcherType::Audio => Self::Audio,
            MatcherType::Book => Self::Book,
            MatcherType::Doc => Self::Doc,
            MatcherType::Font => Self::Font,
            MatcherType::Image => Self::Image,
            MatcherType::Text => Self::Text,
            MatcherType::Video => Self::Video,
            MatcherType::Custom => Self::Custom,
        }
    }
}

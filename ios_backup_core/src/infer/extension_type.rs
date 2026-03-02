use core::{
    fmt::{self, Display},
    hash::{Hash, Hasher},
    str::FromStr,
};

use infer::MatcherType;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExtensionType(pub MatcherType);

impl Display for ExtensionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            MatcherType::Custom => write!(f, "Other"),
            _ => write!(f, "{:?}", self.0),
        }
    }
}

#[derive(Debug)]
pub struct ParseExtensionTypeError;

impl FromStr for ExtensionType {
    type Err = ParseExtensionTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let matcher_type = match s {
            "App" => Ok(MatcherType::App),
            "Archive" => Ok(MatcherType::Archive),
            "Audio" => Ok(MatcherType::Audio),
            "Book" => Ok(MatcherType::Book),
            "Doc" => Ok(MatcherType::Doc),
            "Font" => Ok(MatcherType::Font),
            "Image" => Ok(MatcherType::Image),
            "Text" => Ok(MatcherType::Text),
            "Video" => Ok(MatcherType::Video),
            "Custom" => Ok(MatcherType::Custom),
            _ => Err(ParseExtensionTypeError),
        };
        matcher_type.map(ExtensionType)
    }
}

impl Hash for ExtensionType {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        match self.0 {
            MatcherType::App => 0u8.hash(state),
            MatcherType::Archive => 1u8.hash(state),
            MatcherType::Audio => 2u8.hash(state),
            MatcherType::Book => 3u8.hash(state),
            MatcherType::Doc => 4u8.hash(state),
            MatcherType::Font => 5u8.hash(state),
            MatcherType::Image => 6u8.hash(state),
            MatcherType::Text => 7u8.hash(state),
            MatcherType::Video => 8u8.hash(state),
            MatcherType::Custom => 9u8.hash(state),
        }
    }
}

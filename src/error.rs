use thiserror::Error;

pub type Result<T> = std::result::Result<T, ParserError>;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("XML parsing error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("UTF-8 encoding error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("Parse integer error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("XML attribute error: {0}")]
    Attr(String),

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[error("Schema validation error: {0}")]
    Schema(String),

    #[error("Parse error: {message} at {context}")]
    Parse { message: String, context: String },

    #[error("Invalid tile ID: {0}")]
    InvalidTileId(u32),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

impl ParserError {
    pub fn parse(message: impl Into<String>, context: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
            context: context.into(),
        }
    }

    pub fn encoding(message: impl Into<String>) -> Self {
        Self::Encoding(message.into())
    }

    pub fn schema(message: impl Into<String>) -> Self {
        Self::Schema(message.into())
    }

    pub fn invalid_format(message: impl Into<String>) -> Self {
        Self::InvalidFormat(message.into())
    }
}

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
    /// Creates a `ParserError::Parse` variant with a custom error message and context.
    ///
    /// # Parameters
    /// - `message`: The error message describing the parse failure.
    /// - `context`: Additional context about where or why the error occurred.
    ///
    /// # Returns
    /// A `ParserError` representing a generic parse error with the provided message and context.
    pub fn parse(message: impl Into<String>, context: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
            context: context.into(),
        }
    }

    /// Creates a `ParserError` representing an encoding error with the provided message.
    ///
    /// # Parameters
    /// - `message`: A description of the encoding error.
    ///
    /// # Returns
    /// A `ParserError::Encoding` variant containing the error message.
    pub fn encoding(message: impl Into<String>) -> Self {
        Self::Encoding(message.into())
    }

    /// Creates a `ParserError` representing a schema validation error with the provided message.
    ///
    /// # Parameters
    /// - `message`: Description of the schema error.
    ///
    /// # Returns
    /// A `ParserError::Schema` variant containing the given message.
    pub fn schema(message: impl Into<String>) -> Self {
        Self::Schema(message.into())
    }

    /// Creates a `ParserError` representing an invalid format error with the provided message.
    ///
    /// # Parameters
    /// - `message`: A description of the invalid format encountered.
    ///
    /// # Returns
    /// A `ParserError::InvalidFormat` variant containing the given message.
    pub fn invalid_format(message: impl Into<String>) -> Self {
        Self::InvalidFormat(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_error_constructors() {
        // Test parse error constructor
        let parse_err = ParserError::parse("test message", "test context");
        match parse_err {
            ParserError::Parse { message, context } => {
                assert_eq!(message, "test message");
                assert_eq!(context, "test context");
            }
            _ => panic!("Expected Parse variant"),
        }

        // Test encoding error constructor
        let encoding_err = ParserError::encoding("encoding test");
        match encoding_err {
            ParserError::Encoding(msg) => {
                assert_eq!(msg, "encoding test");
            }
            _ => panic!("Expected Encoding variant"),
        }

        // Test schema error constructor
        let schema_err = ParserError::schema("schema test");
        match schema_err {
            ParserError::Schema(msg) => {
                assert_eq!(msg, "schema test");
            }
            _ => panic!("Expected Schema variant"),
        }

        // Test invalid format error constructor
        let format_err = ParserError::invalid_format("format test");
        match format_err {
            ParserError::InvalidFormat(msg) => {
                assert_eq!(msg, "format test");
            }
            _ => panic!("Expected InvalidFormat variant"),
        }
    }

    #[test]
    fn test_error_display() {
        let parse_err = ParserError::parse("parse failed", "line 42");
        assert_eq!(
            format!("{}", parse_err),
            "Parse error: parse failed at line 42"
        );

        let encoding_err = ParserError::encoding("bad encoding");
        assert_eq!(format!("{}", encoding_err), "Encoding error: bad encoding");

        let schema_err = ParserError::schema("invalid schema");
        assert_eq!(
            format!("{}", schema_err),
            "Schema validation error: invalid schema"
        );

        let format_err = ParserError::invalid_format("bad format");
        assert_eq!(format!("{}", format_err), "Invalid format: bad format");

        let tile_err = ParserError::InvalidTileId(999);
        assert_eq!(format!("{}", tile_err), "Invalid tile ID: 999");

        let attr_err = ParserError::Attr("missing attribute".to_string());
        assert_eq!(
            format!("{}", attr_err),
            "XML attribute error: missing attribute"
        );
    }

    #[test]
    fn test_from_conversions() {
        // Test std::io::Error conversion
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let parser_err = ParserError::from(io_error);
        assert!(matches!(parser_err, ParserError::Io(_)));

        // Test ParseIntError conversion
        let parse_int_err = "abc".parse::<i32>().unwrap_err();
        let parser_err = ParserError::from(parse_int_err);
        assert!(matches!(parser_err, ParserError::ParseInt(_)));

        // Test Utf8Error conversion - use a runtime error
        #[allow(invalid_from_utf8)]
        {
            let invalid_utf8 = &[0xff, 0xfe, 0xfd];
            let utf8_err = std::str::from_utf8(invalid_utf8).unwrap_err();
            let parser_err = ParserError::from(utf8_err);
            assert!(matches!(parser_err, ParserError::Utf8(_)));
        }

        // Test quick_xml::Error conversion - we'll just create one directly
        let xml_err = quick_xml::Error::UnexpectedEof("test".to_string());
        let parser_err = ParserError::from(xml_err);
        assert!(matches!(parser_err, ParserError::Xml(_)));
    }

    #[test]
    fn test_error_debug() {
        let parse_err = ParserError::parse("debug test", "context");
        let debug_output = format!("{:?}", parse_err);
        assert!(debug_output.contains("Parse"));
        assert!(debug_output.contains("debug test"));
        assert!(debug_output.contains("context"));
    }
}

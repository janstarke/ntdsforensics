use std::num::TryFromIntError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("this value has no data")]
    ValueIsMissing,

    #[error("invalid value detected: '{0:?}'; expected type was {1}")]
    InvalidValueDetected(String, &'static str),

    #[error("unable to convert integer '{value:?}' to {intended_type}: {why}")]
    IntegerConversionError {
        value: String,
        intended_type: &'static str,
        why: TryFromIntError,
    },

    #[error("unable to convert '{value:?}' to {intended_type}: {why}")]
    MiscConversionError {
        value: String,
        intended_type: &'static str,
        why: anyhow::Error,
    },

    #[error("no schema record found")]
    MissingSchemaRecord,

    #[error("The schema record has no children")]
    SchemaRecordHasNoChildren,

    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid UUID: {0}")]
    UuidError(#[from] uuid::Error),

    #[error("Invalid SDDL: {0}")]
    SddlError(#[from] sddl::Error)
}

pub type Result<T> = core::result::Result<T, Error>;
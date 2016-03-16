//! A module for the resource parsers

use super::Resource;
use std::rc::Rc;
use std;

/// The different file types a Loader can load from.
#[derive(Debug)]
pub enum FileType {
    ///
    Binary,
    ///
    Text,
}

/// If something goes wrong when a Loader is parsing, this error is returned.
#[derive(Debug)]
pub enum ResourceLoadError {
    /// Represents a file that could not be parsed, whichever error the parser encountered will be
    /// described by the String parameter.
    InvalidFile(String),
    /// File could not be loaded or read
    IoError(std::io::Error),
}


impl std::convert::From<std::io::Error> for ResourceLoadError {
    fn from(e: std::io::Error) -> Self {
        ResourceLoadError::IoError(e)
    }
}

struct EmptyResource;
impl Resource for EmptyResource {}

/// A trait that every resource parser should implement
pub trait ResourceLoader {
    /// This function should return a list of extensions this type can parse. For instance, a jpg
    /// loader would return `Box::new(["jpg", "jpeg"])`.
    fn extensions(&self) -> Box<[&str]>;

    /// What kind of files this parser understands. An OBJ file parser would return
    /// `FileType::Text`. A PNG file parser would return `FileType::Binary`.
    fn file_type(&self) -> FileType;

    /// If the resource loader file type is binary, when it is chosen to parse, this function will
    /// be called. If you override this, you don't have to override `load_from_text`.
    fn load_from_binary(&self, _: &[u8]) -> Result<Rc<Resource>, ResourceLoadError> {
        Ok(Rc::new(EmptyResource))
    }

    /// If the resource loader file type is text, when it is chosen to parse, this function will
    /// be called. If you override this, you don't have to override `load_from_binary`.
    fn load_from_text(&self, _: &str) -> Result<Rc<Resource>, ResourceLoadError> {
        Ok(Rc::new(EmptyResource))
    }
}

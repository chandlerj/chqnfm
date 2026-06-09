use std::path::Path;
use id3::TagLike;
use lofty::{
    error::LoftyError, file::{AudioFile, TaggedFileExt}, probe::Probe, properties, read_from_path, tag::Accessor
};
use serde::Serialize;


use std::fmt;

#[derive(Debug)]
pub enum TrackInfoError {
    Io(std::io::Error),
    TagRead(lofty::error::LoftyError),
    NoTag,
    NoBitrate,
}

impl fmt::Display for TrackInfoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrackInfoError::Io(e) => write!(f, "IO error: {e}"),
            TrackInfoError::TagRead(e) => write!(f, "Tag read error: {e}"),
            TrackInfoError::NoTag => write!(f, "No tag found"),
            TrackInfoError::NoBitrate => write!(f, "Could not parse bitrate")
        }
    }
}

impl std::error::Error for TrackInfoError {}

// From impls let you use ? operator to auto-convert errors
impl From<std::io::Error> for TrackInfoError {
    fn from(e: std::io::Error) -> Self {
        TrackInfoError::Io(e)
    }
}

impl From<lofty::error::LoftyError> for TrackInfoError {
    fn from(e: lofty::error::LoftyError) -> Self {
        TrackInfoError::TagRead(e)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct TrackInfo {
    pub title:   Option<String>,
    pub artist:  Option<String>,
    pub album:   Option<String>,
    pub bitrate: u32,
    pub path:    String,
}

impl TrackInfo {
    pub fn read(path: &Path) -> Result<Self, TrackInfoError> {
        let file = Probe::open(path)?.guess_file_type()?.read()?;
        let tag = file.primary_tag().ok_or(TrackInfoError::NoTag)?; 
        let properties = file.properties();
        Ok(Self {
            title:   tag.title().map(|s| s.to_string()),
            artist:  tag.artist().map(|s| s.to_string()),
            album:   tag.album().map(|s| s.to_string()),
            bitrate: properties.audio_bitrate().ok_or(TrackInfoError::NoBitrate)?,
            path:    path.display().to_string(),
        })
    }
}

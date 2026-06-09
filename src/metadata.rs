use std::path::Path;
use lofty::{
    file::{AudioFile, TaggedFileExt}, probe::Probe,  tag::Accessor
};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Serialize)]
pub enum TrackInfoError {
    Io(String),
    TagRead(String),
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

impl From<std::io::Error> for TrackInfoError {
    fn from(e: std::io::Error) -> Self {
        TrackInfoError::Io(e.to_string())
    }
}

impl From<lofty::error::LoftyError> for TrackInfoError {
    fn from(e: lofty::error::LoftyError) -> Self {
        TrackInfoError::TagRead(e.to_string())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct TrackInfo {
    pub title:   String,
    pub artist:  String,
    pub album:   String,
    pub bitrate: Result<u32, TrackInfoError>,
    pub path:    String,
}

impl TrackInfo {
    pub fn read(path: &Path) -> Result<Self, TrackInfoError> {
        let file = Probe::open(path)?.guess_file_type()?.read()?;
        let tag = file.primary_tag().ok_or(TrackInfoError::NoTag)?; 
        let properties = file.properties();
        Ok(Self {
            title:   tag.title().map(|s| s.to_string()).unwrap_or("Unknown Title".into()),
            artist:  tag.artist().map(|s| s.to_string()).unwrap_or("Unknown Artist".into()),
            album:   tag.album().map(|s| s.to_string()).unwrap_or("Unknown Album".into()),
            bitrate: properties.audio_bitrate().ok_or(TrackInfoError::NoBitrate),
            path:    path.display().to_string(),
        })
    }
}

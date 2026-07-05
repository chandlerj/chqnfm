use crate::metadata::TrackInfo;
use std::collections::HashMap;
use frizbee::{Match, match_list_parallel, Config, radix_sort_matches};
use log::info;

// attributes of the songs with regards to the queue
// describes the allowed bahaviors on the song
pub enum SongProperties {
    Unskippable
}

#[derive(Clone, Debug)] 
pub struct Songbank {
   bank: HashMap<String, TrackInfo>,
}

impl Songbank {
    pub fn new() -> Self {
        Self { 
            bank: HashMap::new() 
        }
    }

    fn store_track(&mut self, track: &TrackInfo) {
        let key = track.get_track_key_id();
        match self.bank.insert(key, track.clone()) {
            Some(_) => {info!("The track {} - {} : {} was already added to the song bank. updating entry with metadata from this insert...", &track.title, &track.artist, &track.album)},
            None => (),
        }
    }

    fn search(&self, query: &str) -> Vec<TrackInfo> {
        let haystacks: Vec<&String> = self.bank.keys().collect();
        let mut matches = match_list_parallel(query, &haystacks, &Config::default(), 8);
        radix_sort_matches(&mut matches);
        
        matches.iter().map(|m| self.bank[m.])
    }
}

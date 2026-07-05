use crate::metadata::TrackInfo;
use frizbee::{match_list_parallel, Config, radix_sort_matches};
use indexmap::IndexMap;
use log::info;

// attributes of the songs with regards to the queue
// describes the allowed bahaviors on the song
pub enum SongProperties {
    Unskippable
}

#[derive(Clone, Debug)] 
pub struct Songbank {
   bank: IndexMap<String, TrackInfo>,
}

impl Songbank {
    pub fn new() -> Self {
        Self { 
            bank: IndexMap::new() 
        }
    }

    fn store_track(&mut self, track: &TrackInfo) {
        let key = track.get_track_key_id();
        match self.bank.insert(key, track.clone()) {
            Some(t) => {info!("The track {} - {} : {} was already inserted into the songbank. Updating with most recent metadata:\n{:?}", t.title, t.artist, t.album, &track)}
            None => ()
        }
    }

    fn search(&self, query: &str) -> Vec<&TrackInfo> {
        let haystacks: Vec<&str> = self.bank.keys().map(|t| t.as_str()).collect();
        let mut matches = match_list_parallel(query, &haystacks, &Config::default(), 8);
        radix_sort_matches(&mut matches);
        
        matches.iter()
            .filter_map(|m| self.bank.get_index(m.index as usize))
            .map(|(_, track)| track)
            .collect()
    }
}

use crate::{
    metadata::TrackInfo,
    playlist::expand
};
use std::path::PathBuf;
use frizbee::{match_list_parallel, Config, radix_sort_matches};
use indexmap::IndexMap;
use log::info;


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

    pub async fn build_songbank(&mut self, path: String) {
        let tracks = expand(PathBuf::from(path)).await;
        for track in tracks {
            self.store_track(&track);
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
        let matches = match_list_parallel(query, &haystacks, &Config::default(), 8);
        
        matches
            .iter()
            .filter_map(|m| self.bank.get_index(m.index as usize))
            .map(|(_, track)| track)
            .collect()
    }

    pub fn print_bank(&self) {
        println!("track bank thus far:");
        for (id, track) in &self.bank {
            println!("{id}: {:?}", track);
        }
    }
}

use crate::metadata::trackInfo;

// attributes of the songs with regards to the queue
// describes the allowed bahaviors on the song
pub enum SongProperties {
    Unskippable
}

#[derive(Clone, Deubg)] 
pub struct Songbank {
   pub bank: Vec<trackInfo>,

}


use nbtrs::Tag;

use std::path::{PathBuf, Path};

/// Encapsulates the concept of a Minecraft "world". A Minecraft world is a
/// level.dat file, a players directory with info about each player, a data
/// directory with info about that world's maps, and one or more "dimension"
/// directories containing a set of region files with the actual world data.
pub struct World {
    world_dir: PathBuf,
    regionsets: Vec<Regionset>
}
impl World {
    pub fn new<P: AsRef<Path>>(p: P) -> World {
        unimplemented!()
    }

    pub fn get_regionsets(&self) -> RegionsetIter {
        unimplemented!()
    }

    pub fn get_regionset(&self, idx: usize) -> Regionset {
        unimplemented!()
    }
}


pub struct RegionsetIter;

impl Iterator for RegionsetIter {
    type Item = Regionset;
    fn next(&mut self) -> Option<Regionset> {
        unimplemented!()
    }
}



/// This object is the gateway to a particular Minecraft dimension within a
/// world. It corresponds to a set of region files containing the actual
/// world data. This object has methods for parsing and returning data from the
/// chunks from its regions.
/// 
/// See the docs for the World object for more information on the difference
/// between Worlds and RegionSets.
pub struct Regionset;
impl Regionset {
    pub fn new() -> Regionset {
        unimplemented!()
    }
    pub fn get_type(&self) -> String {
        unimplemented!()
    }

    pub fn get_chunk(&self, x: u8, z: u8) -> Chunk {
        unimplemented!()
    }

    /// Returns an iterator over all chunk metadata in this world. Iterates
    /// over tuples of integers (x,z,mtime) for each chunk.  Other chunk data
    /// is not returned here.
    pub fn get_chunks(&self) -> ChunkIter {
        unimplemented!()
    }

    pub fn get_chunk_mtime(&self, x: u8, z: u8) -> u64 {
        unimplemented!()
    }

}

pub struct Chunk(Tag);
pub struct ChunkIter;

impl Iterator for ChunkIter {
    type Item = Chunk;
    fn next(&mut self) -> Option<Chunk> {
        unimplemented!()
    }
}

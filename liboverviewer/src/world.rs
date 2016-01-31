
use nbtrs::{Tag, RegionFile};
use super::error::OverviewerError;
use std::path::{PathBuf, Path};
use std::convert::From;
use std::fs::File;

use super::coords;
use super::coords::Coord;

/// Encapsulates the concept of a Minecraft "world". A Minecraft world is a
/// level.dat file, a players directory with info about each player, a data
/// directory with info about that world's maps, and one or more "dimension"
/// directories containing a set of region files with the actual world data.
// TODO consider making these not public members
pub struct World {
    pub world_dir: PathBuf,
    pub regionsets: Vec<Regionset>,
    pub level_dat: Tag,
}
impl World {
    pub fn new<P: AsRef<Path>>(p: P) -> Result<World, OverviewerError> {
        use flate2::read::GzDecoder;

        let world_dir = p.as_ref();
        if !world_dir.exists() {
            return Err(From::from(format!("Path {:?} does not exist", world_dir)));
        }

        let level_dat = world_dir.join("level.dat");
        let level_dat_file = try!(File::open(level_dat));
        let mut decoder = try!(GzDecoder::new(level_dat_file)); 
        let (_, level_dat_nbt) = try!(Tag::parse(&mut decoder));

        let mut regionsets = Vec::new();
        for entry in try!(world_dir.read_dir()) {
            // if this is a directory and it contains .mca files, then assume that it's a regionset
            let path = try!(entry).path();
            if path.is_dir() {
                if try!(path.read_dir()).any(|e| e.ok().map_or(false, |f| f.path().extension().map_or(false, |ex| ex == "mca"))) {
                    regionsets.push(try!(Regionset::new(path)));
                }
            }
        }

        Ok(World {
            world_dir: world_dir.to_owned(),
            regionsets: regionsets,
            level_dat: level_dat_nbt,
        })
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
#[derive(Debug)]
pub struct Regionset {
    region_dir: PathBuf,
    regions: Vec<(i32, i32)>
}
impl Regionset {
    /// Given a folder of MCA files, create a RegionSet
    pub fn new<P: AsRef<Path>>(p: P) -> Result<Regionset, OverviewerError> {
        let region_dir = p.as_ref();
        if !region_dir.exists() {
            return Err(From::from(format!("Path {:?} does not exist", region_dir)))
        }

        let mut regions = Vec::new();
        for file in try!(region_dir.read_dir()) {
            let file = try!(file);
            let file_name = file.file_name();
            let fname_str = file_name.to_string_lossy();
            let components: Vec<&str> = fname_str.split('.').collect();
            if components.len() == 4 && components[0] == "r" && components[3] == "mca" {
                let x = i32::from_str_radix(components[1], 10);
                let z = i32::from_str_radix(components[2], 10);
                if x.is_ok() && z.is_ok() {
                    regions.push((x.unwrap(), z.unwrap())); 
                }
            }

        }

        Ok(Regionset{
            region_dir: region_dir.to_owned(),
            regions: regions
        })

    }

    pub fn get_type(&self) -> String {
        unimplemented!()
    }

    //pub fn get_chunk(&self, xz: ChunkCoord) -> Chunk {
    //    unimplemented!()
    //}

    /// Returns an iterator over all chunk metadata in this world. Iterates
    /// over tuples of integers (x,z,mtime) for each chunk.  Other chunk data
    /// is not returned here.
    pub fn get_chunks(&self) -> ChunkIter {
        unimplemented!()
    }

    pub fn get_chunk_mtime(&self, xz: Coord<coords::Chunk, coords::Region>) -> u64 {
        let x = xz.x;
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic(expected = "IOError")]
    fn test_world_open_error() {
        let world = World::new("/").unwrap();
    }

    #[test]
    fn test_regionset() {
        let rset = Regionset::new("tests/data/OTD/world_189/region").unwrap();
        assert_eq!(rset.regions.len(), 6);
    }

    #[test]
    fn test_world_open() {
        let world = World::new("tests/data/OTD/world_189/").unwrap();
        assert_eq!(world.regionsets.len(), 1);
    }

}

use nbtrs::{Tag, RegionFile, Taglike};
use lru_time_cache::LruCache;
use rio;
use super::error::OverviewerError;
use std::path::{PathBuf, Path};
use std::convert::From;
use std::fs::File;
use std::io::{Read, Seek};
use std::cell::RefCell;

use super::coords;
use super::coords::Coord;

/// Encapsulates the concept of a Minecraft "world". A Minecraft world is a
/// level.dat file, a players directory with info about each player, a data
/// directory with info about that world's maps, and one or more "dimension"
/// directories containing a set of region files with the actual world data.
// TODO consider making these not public members
pub struct World<'fs, FS: rio::FSRead<'fs>> {
    pub world_dir: rio::PathBuf,
    pub regionsets: Vec<Regionset<'fs, FS>>,
    pub level_dat: Tag,
    fs: &'fs FS
}
impl<'fs, FS> World<'fs, FS> where FS: rio::FSRead<'fs>, FS::ReadFile: Read + Seek {
    /// Given a path to a world (a directory containing a level.dat file), construct a World
    pub fn new<P: AsRef<rio::Path>>(fs: &'fs FS, p: P) -> Result<World<FS>, OverviewerError> {
        use flate2::read::GzDecoder;

        let world_dir = p.as_ref();
        if ! fs.exists(world_dir) {
            return Err(From::from(format!("Path {:?} does not exist", world_dir)));
        }

        let level_dat = world_dir.join("level.dat");
        let level_dat_file = try!(fs.open(level_dat));
        let mut decoder = try!(GzDecoder::new(level_dat_file));
        let (_, level_dat_nbt) = try!(Tag::parse(&mut decoder));

        let mut regionsets = Vec::new();
        for entry in try!(fs.read_dir(world_dir)) {
            // if this is a directory and it contains .mca files, then assume that it's a regionset
            //let path = entry.path();
            if entry.is_dir() {
                if try!(entry.read_dir()).any(|e| {
                    // e is a QPath
                    e.path().extension() == Some("mca")
                }) {
                    regionsets.push(try!(Regionset::new(fs, entry)));
                }
            }
        }

        Ok(World {
            world_dir: world_dir.to_owned(),
            regionsets: regionsets,
            level_dat: level_dat_nbt,
            fs: fs
        })
    }

    //pub fn get_regionsets(&self) -> ::std::slice::Iter<Regionset<T>> {
    //    self.regionsets.iter()
    //}

    //pub fn get_regionset(&self, idx: usize) -> Option<&Regionset<T>> {
    //    self.regionsets.get(idx)
    //}
}

//pub struct RegionsetIter<T>;
//
//impl<T> Iterator for RegionsetIter<T> {
//    type Item = Regionset<T>;
//    fn next(&mut self) -> Option<Regionset<T>> {
//        unimplemented!()
//    }
//}


/// This object is the gateway to a particular Minecraft dimension within a
/// world. It corresponds to a set of region files containing the actual
/// world data. This object has methods for parsing and returning data from the
/// chunks from its regions.
///
/// See the docs for the World object for more information on the difference
/// between Worlds and RegionSets.
pub struct Regionset<'fs, FS: rio::FSRead<'fs>> {
    region_dir: rio::PathBuf,

    // A vec of regions might be too memory intensive, so hold a list of regions by coords
    regions: Vec<(i64, i64)>,

    cache: RefCell<LruCache<(i64, i64), RegionFile<FS::ReadFile>>>,
    fs: &'fs FS
}



impl<'fs, FS> Regionset<'fs, FS> where FS: rio::FSRead<'fs>, FS::ReadFile: Read + Seek {
    /// Given a folder of MCA files, create a RegionSet
    pub fn new<P: AsRef<rio::Path>>(fs: &'fs FS, p: P) -> Result<Regionset<'fs, FS>, OverviewerError> {
        let region_dir = p.as_ref();
        if !fs.exists(region_dir) {
            return Err(From::from(format!("Path {:?} does not exist", region_dir)));
        }

        let mut regions = Vec::new();
        for file in try!(fs.read_dir(region_dir)) {
            let fname_str = file.path().file_name().unwrap();
            let components: Vec<&str> = fname_str.split('.').collect();
            if components.len() == 4 && components[0] == "r" && components[3] == "mca" {
                let x = i64::from_str_radix(components[1], 10);
                let z = i64::from_str_radix(components[2], 10);
                if x.is_ok() && z.is_ok() {
                    regions.push((x.unwrap(), z.unwrap()));
                }
            }

        }

        Ok(Regionset {
            region_dir: region_dir.to_owned(),
            regions: regions,
            cache: RefCell::new(LruCache::with_capacity(16)),
            fs: fs
        })

    }

    pub fn get_type(&self) -> String {
        unimplemented!()
    }

    pub fn get_chunk(&self, xz: Coord<coords::Chunk, coords::World>) -> Option<Chunk> {
        // what regionfile is this chunk in?
        let (c, r) = xz.split::<coords::Region>();
        if !self.regions.contains(&(r.x, r.z)) {
            return None;
        }

        let mut cache = self.cache.borrow_mut();
        let region_file: &mut RegionFile<_> = cache.entry((r.x, r.z)).or_insert_with(|| {
            let fp = self.region_dir.join(format!("r.{}.{}.mca", r.x, r.z));
            let f = self.fs.open(fp).unwrap();
            RegionFile::new(f).unwrap()
        });

        if let Ok(chunk) = region_file.load_chunk(c.x as u8, c.z as u8) {
            return Some(Chunk(chunk));
        }

        None
    }

    /// Returns an iterator over all chunk metadata in this world. Iterates
    /// over tuples of integers (x,z,mtime) for each chunk.  Other chunk data
    /// is not returned here.
    pub fn get_chunks(&self) -> ChunkIter {
        unimplemented!()
    }

    // TODO consider using something other than a u32 for time (like bring in one of the types from
    // chrono)
    pub fn get_chunk_mtime(&self, xz: Coord<coords::Chunk, coords::World>) -> Option<u32> {
        // what regionfile is this chunk in?
        let (c, r) = xz.split::<coords::Region>();
        if !self.regions.contains(&(r.x, r.z)) {
            return None;
        }
        let f = self.region_dir.join(format!("r.{}.{}.mca", r.x, r.z));
        if let Ok(f) = self.fs.open(f) {
            if let Ok(region_file) = RegionFile::new(f) {
                return region_file.get_chunk_timestamp(c.x as u8, c.z as u8);
            }
        }

        None
    }
}

#[derive(Debug)]
pub struct Chunk(Tag);
pub struct ChunkIter;

impl Iterator for ChunkIter {
    type Item = Chunk;
    fn next(&mut self) -> Option<Chunk> {
        unimplemented!()
    }
}

impl Chunk {
    /// Heightmap for this chunk, pre-computed by Minecraft
    ///
    /// to index into this vec:
    ///
    /// let height = v.get(x + z*16)
    pub fn get_heightmap(&self) -> Vec<u32> {
        let &Chunk(ref tag) = self;
        // 256 tagints.  16x16
        //let h = map.get(x + (z*16)).unwrap() - 64;
        let data = tag.key("Level").key("HeightMap").as_ints().unwrap();
        return data.clone();
        //println!("height at x=3 z=12 {:?}", map.get(3 + 12*16));
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use coords::Coord;
    use ::rio;

    fn build_fs() -> rio::Native {
        rio::Native::new(::std::env::current_dir().unwrap())
    }

    #[test]
    #[should_panic(expected = "IOError")]
    fn test_world_open_error() {
        let fs: rio::Native = build_fs();
        let world = World::new(&fs, "/").unwrap();
    }

    #[test]
    fn test_regionset() {
        let fs: rio::Native = build_fs();
        let rset = Regionset::new(&fs, "tests/data/OTD/world_189/region").unwrap();
        assert_eq!(rset.regions.len(), 6);
    }

    #[test]
    fn test_world_open() {
        let fs: rio::Native = build_fs();
        let world = World::new(&fs, "tests/data/OTD/world_189/").unwrap();
        assert_eq!(world.regionsets.len(), 1);
    }

    #[test]
    fn test_regionset_get_chunk() {
        use nbtrs::Taglike;
        let fs: rio::Native = build_fs();

        {
            let mut rset = Regionset::new(&fs, "tests/data/OTD/world_189/region").unwrap();
            let Chunk(chunk) = rset.get_chunk(Coord::new(0, 0, 0)).unwrap();
            let x = &chunk.key("Level").key("xPos").as_i32().unwrap();
            let z = &chunk.key("Level").key("zPos").as_i32().unwrap();
            assert_eq!(x, &0);
            assert_eq!(z, &0);
        }
        {
            let mut rset = Regionset::new(&fs, "tests/data/OTD/world_189/region").unwrap();
            let Chunk(chunk) = rset.get_chunk(Coord::new(4, 0, 8)).unwrap();
            let x = &chunk.key("Level").key("xPos").as_i32().unwrap();
            let z = &chunk.key("Level").key("zPos").as_i32().unwrap();
            assert_eq!(x, &4);
            assert_eq!(z, &8);
        }
    }

    #[test]
    fn test_regionset_get_chunk_mtime() {
        let fs: rio::Native = build_fs();
        let mut rset = Regionset::new(&fs, "tests/data/OTD/world_189/region").unwrap();
        assert_eq!(rset.get_chunk_mtime(Coord::new(4, 0, 8)), Some(1454034069));
        assert_eq!(rset.get_chunk_mtime(Coord::new(12, 0, 3)), Some(1454033798));
    }

    #[test]
    fn test_chunk_heightmap() {
        return;
        let fs: rio::Native = build_fs();
        let mut rset = Regionset::new(&fs, "/storage/home/achin/.minecraft/saves/world_189/region").unwrap();
        let chunk = rset.get_chunk(Coord::new(6, 0, -1)).unwrap();
        chunk.get_heightmap();
    }

}

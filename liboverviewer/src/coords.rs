
use std::marker::PhantomData;

pub struct BlockCoordType;
pub struct ChunkCoordType;
pub struct RegionCoordType;
pub struct WorldSystemType;
pub struct ChunkSystemType;

/// Represents a generic 3-dimensional coordinate for a given coordinate type in a given system
/// type
pub struct Coord3<T, CoordType, SystemType> {
    x: T,
    y: T,
    z: T,
    _coord_type: PhantomData<CoordType>,
    _system_type: PhantomData<SystemType>
}


/// Represents a generic 2-dimensional coordinate for a given coordinate type in a given system
/// type
pub struct Coord2<T, CoordType, SystemType> {
    x: T,
    z: T,
    _coord_type: PhantomData<CoordType>,
    _system_type: PhantomData<SystemType>
}

/// A global block coordinate
pub type BlockCoord = Coord3<i32, BlockCoordType, WorldSystemType>;
macro_rules! block_coord {
    ($x:expr, $y:expr, $z:expr) => (BlockCoord{x:$x, y:$y, z:$z, _coord_type: PhantomData, _system_type: PhantomData})
}

/// A global chunk coordinate
pub type ChunkCoord = Coord3<i32, ChunkCoordType, WorldSystemType>;
macro_rules! chunk_coord {
    ($x:expr, $y:expr, $z:expr) => (ChunkCoord{x:$x, y:$y, z:$z, _coord_type: PhantomData, _system_type: PhantomData})
}

/// Coordinates for a block within a chunk.  bounded between 0, 15 (inclusive)
pub type BlockInChunkCoord = Coord3<i32, BlockCoordType, ChunkSystemType>;

/// Coordinates for a region file
pub type RegionCoord = Coord2<i32, RegionCoordType, WorldSystemType>;

impl BlockCoord {
    /// What chunk contains these coordinates
    pub fn to_chunk_coord(&self) -> ChunkCoord {
        ChunkCoord { x: self.x >> 4,
                     y: self.y >> 4,
                     z: self.z >> 4,
                     _coord_type: PhantomData,
                     _system_type: PhantomData
        }
    }
    /// Within a chunk, what are these coordinates
    pub fn to_inchunk_coord(&self) -> BlockInChunkCoord {
        let xmod = if self.x < 0 { 16 } else { 0 };
        let ymod = if self.y < 0 { 16 } else { 0 };
        let zmod = if self.z < 0 { 16 } else { 0 };
        BlockInChunkCoord { x: (self.x % 16) + xmod,
                            y: (self.y % 16) + ymod,
                            z: (self.z % 16) + zmod,
                            _coord_type: PhantomData,
                            _system_type: PhantomData
        }
    }

}

impl ChunkCoord {
    /// What region contains this chunk
    pub fn to_region_coord(&self) -> RegionCoord {
            RegionCoord{x: self.x >> 5,
                        z: self.z >> 5,
                        _coord_type: PhantomData,
                        _system_type: PhantomData
            }
    }
}



#[cfg(test)]
mod test {
    use super::*;
    use std::marker::PhantomData;


    #[test]
    fn test_coord_types() {
        {
            let block = block_coord!(5, 68, 0);
            let chunk = block.to_chunk_coord();
            assert_eq!(chunk.x, 0);
            assert_eq!(chunk.y, 4);
            assert_eq!(chunk.z, 0);

            let inchunk = block.to_inchunk_coord();
            assert_eq!(inchunk.x, 5);
            assert_eq!(inchunk.y, 4);
            assert_eq!(inchunk.z, 0);
        }

        {
            let block = block_coord!(31, 79, 31);
            let chunk = block.to_chunk_coord();
            assert_eq!(chunk.x, 1);
            assert_eq!(chunk.y, 4);
            assert_eq!(chunk.z, 1);

            let inchunk = block.to_inchunk_coord();
            assert_eq!(inchunk.x, 15);
            assert_eq!(inchunk.y, 15);
            assert_eq!(inchunk.z, 15);
        }

        {
            let block = block_coord!(-1, 63, -2);
            let chunk = block.to_chunk_coord();
            assert_eq!(chunk.x, -1);
            assert_eq!(chunk.y, 3);
            assert_eq!(chunk.z, -1);

            let inchunk = block.to_inchunk_coord();
            assert_eq!(inchunk.x, 15);
            assert_eq!(inchunk.y, 15);
            assert_eq!(inchunk.z, 14);
        }
        {
            let chunk = chunk_coord!(30, 4, -3);
            let region = chunk.to_region_coord();
            assert_eq!(region.x, 0);
            assert_eq!(region.z, -1);
        }
        {
            let chunk = chunk_coord!(70, 16, -30);
            let region = chunk.to_region_coord();
            assert_eq!(region.x, 2);
            assert_eq!(region.z, -1);
        }
    }
}

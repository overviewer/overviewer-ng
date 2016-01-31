//! Coordinate types for liboverviewer.
//!
//! Within Minecraft, there are several different coordinate types.  For example, block
//! coordinates, chunk coordintes, and region coordinates.  Each type can also exist in different
//! systems.  For example, a block with global world coordinates of `(27, 79, -8)` has in-chunk
//! coordinates of `(11, 79, -9)`.
//!
//! This module represents each type independently, via the
//! parametric [`Coord`] type.  This will be the type that you'll interact with most directly.  To
//! construct coordinates, use the [`coords!`] macro.
//!
//! To convert between the different coordinate systems, use the [`join`] and [`split`] methods.
//! See their documentation for examples.
//!
//! The rest of the types in this module are mostly internal types to glue things together.  Unless
//! you are trying to build a new coordinate type/system, you will not use them directly, and they
//! can be ignored.
//!
//! [`Coord`]: struct.Coord.html
//! [`coords!`]: ../macro.coord!.html
//! [`join`]: struct.Coord.html#method.join
//! [`split`]: struct.Coord.html#method.split
use std::fmt::{Formatter, Error, Debug};
use std::marker::PhantomData;

// our basic systems: Block, and Things Bigger Than Block
/// The most basic coordinate type
pub struct Block;

/// Abstractly represents a coordinate type that made of a smaller type `N`
///
/// For example:
///
/// ```ignore
/// type Region = Succ<Chunk>;
/// ```
///
/// indicates that a Region is made up of Chunks.  The exact number of Chunks is specified by
/// implementing the [`System`] trait on Region.
///
/// [`System`]: trait.System.html
pub struct Succ<N>(PhantomData<N>);

// a System is defined by its max coordinate type
// and its bit width in each direction, in terms of what it contains.
// A width of 3 on a Region means 8 chunks in that direction.
/// A Coordinate System
///
/// A `System` is defined by its name and its bitwidth in each direction.  This specifics how
/// things it contains as 2^n
///
/// For example, this is how the [`Region`] System is defined:
///
/// ```ignore
/// pub type Region = Succ<Chunk>;
/// impl System for Region {
///     fn name() -> &'static str { "Region" }
///     fn size() -> (u8, u8, u8) {
///         let (px, py, pz) = <Chunk as System>::size();
///         (5 + px, 0 + py, 5 + pz)
///     }
/// }
/// ```
///
/// [`Region`]: type.Region.html
pub trait System {
    /// The name of this sytem.
    fn name() -> &'static str;

    /// Its size in each direction, specificed as a bitwidth
    fn size() -> (u8, u8, u8) { (0, 0, 0) }
}

// Blocks are special, they have no parent and no width.
impl System for Block {
    fn name() -> &'static str { "Block" }
}

// handy macro: contains!(A, Coord, (wx, wy, wz), B)
// means A contains B, and uses Coord as maximal coordinate
macro_rules! contains {
    ($(#[$doc:meta])*
     impl $a:ident, ($x:expr, $y:expr, $z:expr), $parent:ty) => {
        $(#[$doc])*
        pub type $a = Succ<$parent>;
        impl System for $a {
            fn name() -> &'static str { stringify!($a) }
            fn size() -> (u8, u8, u8) {
                let (px, py, pz) = <$parent as System>::size();
                ($x + px, $y + py, $z + pz)
            }
        }
    }
}

// Sections, Chunks, and Regions

contains!{
/// a Section contains 16 blocks in each direction
impl Section,(4, 4, 4), Block}

// a Chunk contains 2^4==16 Sections in the Y axis
contains!{
/// a Chunk contains 16 Sections in the Y axis
impl Chunk, (0, 4, 0), Section}

// a Region contains 2^5==32 Chunks in the X and Z axis
contains!{
/// a Region contains 32 Chunks in the X and Z axis
impl Region, (5, 0, 5), Chunk}

// World is also special, it contains infinity regions
/// a World contains an infinite amount of chunks in the xz plane
pub type World = Succ<Region>;
impl System for World {
    fn name() -> &'static str { "World" }
    fn size() -> (u8, u8, u8) { panic!("infinity") }
}

// A: Contained<B> is true only if A is a subelement of (is contained in) B at some point
// e.g. Section: Contained<Region>
/// Represents the idea that one coordinate type and containe another
///
/// For example, a Chunk contains a Block
pub trait Contained<M> {}

// Block is Contained in Succ<M> (forall M); everything contains Block
impl<M> Contained<Succ<M>> for Block {}

// if N: Contained<M>, then Thing After N is contained in Thing After M
impl<M, N: Contained<M>> Contained<Succ<M>> for Succ<N> {}

// now we get to use our coordinate types! Here's an Actual Coordinate
// giving the location of an El inside a In
/// A three-dimensional coordinate of some type
///
/// The `El` type parameter is the coordinate type, and `In` is the system in which this coordinate
/// type exists
///
/// The best way to construct a `Coord` is to use the [`coord!`] macro.
///
/// [`coord!`]: ../macro.coord!.html
#[derive(Clone, Copy)]
pub struct Coord<El, In> {
    /// Positive X faces east
    pub x: i64,

    /// Positive Y faces up
    pub y: i64,

    /// Positive Z faces south
    pub z: i64,
    pub phantom: PhantomData<(El, In)>
}

// macro to make constructing coordinates less verbose
// coord!(x, y, z) or coord!(El, In, x, y, z) both work.
/// A macro to make constructing coordinates less verbose.
///
/// You could just use the constructor for [`Coord`], but this macro is generally easier and
/// clearer.
///
/// # Examples
///
/// ```ignore
/// // A Block coordinate, within a chunk
/// coord!{Block, Chunk, 12, 40, 0}
/// ```
///
/// You can omit the types and let the type system figure it out for you:
///
/// ```ignore
/// coord!{0, 1, 2}
/// ```
///
/// [`Coord`]: coords/struct.Coord.html
#[macro_export]
macro_rules! coord {
    ($x:expr, $y:expr, $z:expr) => {
        Coord {
            x: $x,
            y: $y,
            z: $z,
            phantom: PhantomData
        }
    };
    ($from:ty, $to:ty, $x:expr, $y:expr, $z:expr) => {
        Coord::<$from, $to> {
            x: $x,
            y: $y,
            z: $z,
            phantom: PhantomData
        }
    };
}

// nice formatter for coordinates, using the macro representation
impl<El: System + Contained<In>, In: System> Debug for Coord<El, In> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        formatter.write_str(format!("coord!({}, {}, {:?}, {:?}, {:?})", El::name(), In::name(), self.x, self.y, self.z).as_ref())
    }
}

// join and split!
impl<El: Contained<In> + System, In: System> Coord<El, In> {
    // take an A-in-B coordinate, and add on a B-in-C coordinate
    // to create an A-in-C coordinate
    /// Given a A-in-B coordinate, add a B-in-C coordinate to create an A-in-C coordinate.
    ///
    /// For example, given an in-chunk coordinate of `(13, 64, 12)` and a block coordinate of (2,
    /// -2), what is the global block coordinate?  This is essentially the opposite of the example
    /// given for [`split`]
    ///
    /// ```
    /// # #[macro_use] extern crate liboverviewer;
    /// # use liboverviewer::coords::*;
    /// # use std::marker::PhantomData;
    /// # fn main() {
    /// let a = coord!{Block, Chunk, 13, 64, 12};
    /// let b = coord!{Chunk, World, 2, 0, -2}; // unused coords must be zero
    /// let c: Coord<Block, World> = a.join(b);
    ///
    /// assert_eq!(c.x, 45);
    /// assert_eq!(c.y, 64);
    /// assert_eq!(c.z, -20);
    /// # }
    /// ```
    ///
    /// [`split`]: #method.split
    pub fn join<End>(self, other: Coord<In, End>) -> Coord<El, End>
        where El: Contained<End>, In: Contained<End>, End: System
    {
        let (ox, oy, oz) = (other.x, other.y, other.z);
        let (x, y, z) = (self.x, self.y, self.z);
        let (osizex, osizey, osizez) = <In as System>::size();
        let (sizex, sizey, sizez) = <El as System>::size();
        coord!(x + (ox << (osizex - sizex)),
               y + (oy << (osizey - sizey)),
               z + (oz << (osizez - sizez)))
    }
    
    // split an A-in-C coordinate into (A-in-B, B-in-C) for any B
    // use like: let (a_in_b, b_in_c) = coord.split::<B>()
    /// Split this coordinate into two components
    ///
    /// # Example
    ///
    /// Given a global block coordinate, find the chunk that contains this block, and block
    /// coordinates within that chunk
    ///
    /// ```
    /// # #[macro_use] extern crate liboverviewer;
    /// # use liboverviewer::coords::*;
    /// # use std::marker::PhantomData;
    /// # fn main() {
    /// let block = coord!{Block, World, 45, 64, -20};
    /// let (a, b): (Coord<Block, Chunk>, Coord<Chunk, World>) = block.split();
    /// // `a` is the block coords within the chunk
    /// // `b` is the chunk coords within the world
    /// assert_eq!(a.x, 13);
    /// assert_eq!(a.y, 64);
    /// assert_eq!(a.z, 12);
    ///
    /// assert_eq!(b.x, 2);
    /// assert_eq!(b.z, -2);
    /// # }
    /// ```
    ///
    /// A less verbose way of the above would be:
    ///
    /// ```
    /// # #[macro_use] extern crate liboverviewer;
    /// # use liboverviewer::coords::*;
    /// # use std::marker::PhantomData;
    /// # fn main() {
    /// let block = coord!{Block, World, 45, 64, -20};
    /// let (a, b) =  block.split::<Chunk>();
    /// # assert_eq!(a.x, 13);
    /// # assert_eq!(a.y, 64);
    /// # assert_eq!(a.z, 12);
    /// # assert_eq!(b.x, 2);
    /// # assert_eq!(b.z, -2);
    /// # }
    /// ```
    pub fn split<Mid>(self) -> (Coord<El, Mid>, Coord<Mid, In>)
        where El: Contained<Mid>, Mid: System + Contained<In>
    {
        let (x, y, z) = (self.x, self.y, self.z);
        let (osizex, osizey, osizez) = <Mid as System>::size();
        let (sizex, sizey, sizez) = <El as System>::size();
        let a = coord!(x & ((1 << (osizex - sizex)) - 1),
                       y & ((1 << (osizey - sizey)) - 1),
                       z & ((1 << (osizez - sizez)) - 1));
        let b = coord!(x >> (osizex - sizex), y >> (osizey - sizey), z >> (osizez - sizez));
        (a, b)
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use std::marker::PhantomData;


    #[test]
    fn test_coord_types() {
        {
            let block = coord!(Block, World, 5, 68, 0);
            let (inchunk, chunk): (Coord<Block, Chunk>, Coord<Chunk, World>) = block.split();
            assert_eq!(inchunk.x, 5);
            assert_eq!(inchunk.y, 68);
            assert_eq!(inchunk.z, 0);

            assert_eq!(chunk.x, 0);
            assert_eq!(chunk.z, 0);

            let joined: Coord<Block, World> = inchunk.join(chunk);
            assert_eq!(joined.x, 5);
            assert_eq!(joined.y, 68);
            assert_eq!(joined.z, 0);
        }

        {
            let block = coord!(Block, World, 31, 79, 31);
            let (inchunk, chunk): (Coord<Block, Chunk>, Coord<Chunk, World>) = block.split();
            assert_eq!(chunk.x, 1);
            assert_eq!(chunk.z, 1);

            assert_eq!(inchunk.x, 15);
            assert_eq!(inchunk.y, 79);
            assert_eq!(inchunk.z, 15);
        }

        {
            let block = coord!(-1, 63, -2);
            let (inchunk, chunk): (Coord<Block, Chunk>, Coord<Chunk, World>) = block.split();
            assert_eq!(chunk.x, -1);
            assert_eq!(chunk.z, -1);

            assert_eq!(inchunk.x, 15);
            assert_eq!(inchunk.y, 63);
            assert_eq!(inchunk.z, 14);
        }
        {
            let chunk = coord!(Chunk, World, 30, 4, -3);
            let (_, region): (Coord<Chunk, Region>, Coord<Region, World>)  = chunk.split();
            assert_eq!(region.x, 0);
            assert_eq!(region.z, -1);
        }
        {
            let chunk = coord!(Chunk, World, 70, 16, -30);
            let (_, region): (Coord<Chunk, Region>, Coord<Region, World>)  = chunk.split();
            assert_eq!(region.x, 2);
            assert_eq!(region.z, -1);
        }
    }
}

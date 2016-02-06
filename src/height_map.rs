extern crate image;
extern crate liboverviewer;
extern crate rio;

use liboverviewer::world::*;
use liboverviewer::coords;

use std::fs::File;
use std::path::Path;
use std::collections::HashMap;

fn main() {
    let fs = rio::Native::new("/");
    let rset = Regionset::new(&fs, "/storage/home/achin/.minecraft/saves/hmap/region").unwrap();

    let mut chunk_cache: HashMap<(i64, i64), Chunk> = HashMap::new();

    let imgx = 1024;
    let imgy = 1024;
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);
    let mut cur_x = 0;

    for (x, z, pixel) in imgbuf.enumerate_pixels_mut() {
        if z != cur_x {
            cur_x = z;
            //println!("{}", z);
        }
        let block_coord = coords::Coord::<coords::Block, coords::World>::new(x as i64 - 512, 0, z as i64 - 512);
        let (block_in_chunk, chunk_in_world) = block_coord.split::<coords::Chunk>();

        if !rset.chunk_exists(chunk_in_world) {
            continue;
        }
        let chunk: &Chunk = chunk_cache.entry((chunk_in_world.x, chunk_in_world.z)).or_insert_with(|| {
            rset.get_chunk(chunk_in_world).unwrap()
        });

        let map = chunk.get_heightmap();

        let h = map.get((block_in_chunk.x + (block_in_chunk.z*16)) as usize).unwrap();
        *pixel = image::Luma([255u8 - *h as u8 ]);

    }
    let ref mut fout = File::create(&Path::new("hmap.png")).unwrap();

    // We must indicate the image’s color type and what format to save as
    let _ = image::ImageLuma8(imgbuf).save(fout, image::PNG);


}

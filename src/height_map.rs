extern crate image;
#[macro_use]
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

    let imgx = 512;
    let imgy = 512;
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);
    let mut cur_x = 0;

    // produce an imagemap for a single region, where each block column is 1 pixel.  so 512 by 512
    for chunk_x in 0..32 {
        for chunk_z in 0..32 {
            let chunk_in_region = coord!(chunk_x, 0, chunk_z);
            if let Some(chunk) = rset.get_chunk(chunk_in_region) {
                let map = chunk.get_heightmap();

                for block_x in 0..16 {
                    for block_z in 0..16 {
                        let block_in_chunk =
                            coord!{coords::Block, coords::Chunk, block_x, 0, block_z};
                        let h = map.get((block_x + (block_z * 16)) as usize).unwrap();

                        let block_in_region = block_in_chunk.join(chunk_in_region);
                        assert!(block_in_region.x >= 0);
                        assert!(block_in_region.z >= 0);
                        // *pixel = image::Luma([255u8 - *h as u8 ]);
                        imgbuf.put_pixel(block_in_region.x as u32,
                                         block_in_region.z as u32,
                                         image::Luma([255u8 - *h as u8]));
                    }
                }
            }

        }
    }


    let ref mut fout = File::create(&Path::new("hmap.png")).unwrap();

    // We must indicate the image’s color type and what format to save as
    let _ = image::ImageLuma8(imgbuf).save(fout, image::PNG);


}

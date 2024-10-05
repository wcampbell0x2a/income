use std::env;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use income::{Image, UBI_EC_HDR_MAGIC};
use log::{info, trace};

pub fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();

    // read UBI Block type image starting with `UBI#` (EcHdr)
    let mut reader = std::fs::File::options().read(true).open(&args[1]).unwrap();
    let file_len = reader.metadata().unwrap().len();

    // find block size
    let mut block_size = 0;
    for n in 10..20 {
        reader.seek(SeekFrom::Start(1 << n)).unwrap();
        let mut buf = [0; 4];
        reader.read_exact(&mut buf).unwrap();
        if buf == UBI_EC_HDR_MAGIC {
            block_size = 1 << n;
            break;
        }
    }
    trace!("using block size: {:02x?}", block_size);

    let image = Image::read(&mut reader, file_len, block_size);

    // UbiBlocks
    info!(
        "{} named volumes, {} physical volumes, blocksize={block_size:#x?}",
        image.vtable.len(),
        image.map.len()
    );
    for v in &image.vtable {
        let mut file_write = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(v.1.name().unwrap().to_str().unwrap())
            .unwrap();

        image.read_volume(&mut reader, &mut file_write, block_size, v)
    }
}

use std::env;
use std::io::{Read, Seek, SeekFrom};

use income::{Image, UBI_EC_HDR_MAGIC};

pub fn main() {
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

    let image = Image::read(&mut reader, file_len, block_size);

    // UbiBlocks
    println!(
        "{} named volumes, {} physical volumes, blocksize={block_size:#x?}",
        image.vtable.len(),
        image.map.len()
    );

    image.extract(&mut reader, block_size);
}

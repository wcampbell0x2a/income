use std::fs;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use income::{Image, UBI_EC_HDR_MAGIC};
use log::{info, trace};

mod logger;

#[derive(Parser, Debug)]
#[clap(author, version, about = "Extract volumes from UBI image")]
struct Args {
    /// Path to the UBI image
    imagepath: PathBuf,
}

fn main() -> Result<()> {
    logger::init();
    let args = Args::parse();

    // read UBI Block type image starting with `UBI#` (EcHdr)
    let filepath = args.imagepath;
    let mut reader = std::fs::File::options()
        .read(true)
        .open(&filepath)
        .context(format!("Failed to read file: {}", filepath.display()))?;
    let file_len = reader.metadata().unwrap().len();

    // find block size
    let mut block_size = 0;
    for n in 10..20 {
        reader.seek(SeekFrom::Start(1 << n)).unwrap();
        let mut buf = [0; 4];
        reader.read_exact(&mut buf).context("Could not read bytes to find block size")?;
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
        let name = v.1.name().unwrap().into_string().unwrap();
        let filepath =
            Path::new("ubi-root").join(format!("img-{}_vol-{name}.ubifs", image.ec.image_seq));
        let _ = fs::create_dir_all(filepath.parent().unwrap());

        let mut file_write =
            File::options().write(true).create(true).truncate(true).open(&filepath).unwrap();

        image.read_volume(&mut reader, &mut file_write, block_size, v);
        info!("wrote: {}", filepath.display());
    }

    Ok(())
}

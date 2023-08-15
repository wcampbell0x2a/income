#![feature(int_roundings)]

use std::collections::HashMap;

use std::env;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

use deku::prelude::*;
use income::{EcHdr, VidHdr, VolType, VtblRecord, UBI_EC_HDR_MAGIC, VTBL_VOLID};

pub fn main() {
    let args: Vec<String> = env::args().collect();

    // read UBI Block type image starting with `UBI#` (EcHdr)
    let mut file = std::fs::File::options().read(true).open(&args[1]).unwrap();
    let file_len = file.metadata().unwrap().len();

    let mut block_size = 0; // or lebsize
                            // find block size
    for n in 10..20 {
        file.seek(SeekFrom::Start(1 << n)).unwrap();
        let mut buf = [0; 4];
        file.read_exact(&mut buf).unwrap();
        if buf == UBI_EC_HDR_MAGIC {
            block_size = 1 << n;
            break;
        }
    }

    let maxlebs = file_len.div_floor(block_size);

    // scanblocks()
    let mut map = HashMap::<u32, Vec<u64>>::new();
    for lnum in 0..maxlebs {
        // Ec
        file.seek(SeekFrom::Start(lnum * block_size + 0)).unwrap();
        let mut container = Container::new(file.try_clone().unwrap());
        let ec = EcHdr::from_reader(&mut container, ()).unwrap();

        // Vid
        file.seek(SeekFrom::Start(
            lnum * block_size + u64::from(ec.vid_hdr_offset),
        ))
        .unwrap();
        let mut container = Container::new(file.try_clone().unwrap());
        let vid = VidHdr::from_reader(&mut container, ()).unwrap();

        if let Some(x) = map.get_mut(&vid.vol_id) {
            x.push(lnum);
        } else {
            map.insert(vid.vol_id, vec![lnum]);
        }
    }

    // read the volume table
    let lnum = map[&VTBL_VOLID][0]; // TODO: i guess the first entry?
    let start_of_volume = lnum * block_size;
    file.seek(SeekFrom::Start(start_of_volume)).unwrap();
    let mut container = Container::new(file.try_clone().unwrap());
    let ec = EcHdr::from_reader(&mut container, ()).unwrap();

    file.seek(SeekFrom::Start(
        lnum * block_size + u64::from(ec.vid_hdr_offset),
    ))
    .unwrap();
    let mut container = Container::new(file.try_clone().unwrap());
    let vid = VidHdr::from_reader(&mut container, ()).unwrap();
    assert_eq!(vid.lnum as u64, lnum);

    let mut vtable = vec![];
    file.seek(SeekFrom::Start(start_of_volume + ec.data_offset as u64))
        .unwrap();

    // TODO: I need to figure this out...
    //
    // You'll see this. with data_offset at 0x80, we still need to read something(???) until 0x12c
    // where a VtblRecord starts.
    //
    //   10   │ │00000080│ 00 00 00 00 00 00 00 00 ┊ 00 00 00 00 00 00 00 00 │⋄⋄⋄⋄⋄⋄⋄⋄┊⋄⋄⋄⋄⋄⋄⋄⋄│
    //   11   │ │*       │                         ┊                         │        ┊        │
    //   12   │ │00000120│ 00 00 00 00 00 00 00 00 ┊ f1 16 c3 6b 00 00 00 01 │⋄⋄⋄⋄⋄⋄⋄⋄┊×•×k⋄⋄⋄•│
    //   13   │ │00000130│ 00 00 00 01 00 00 00 00 ┊ 02 00 00 05 61 70 70 6c │⋄⋄⋄•⋄⋄⋄⋄┊•⋄⋄•appl│
    //   14   │ │00000140│ 65 00 00 00 00 00 00 00 ┊ 00 00 00 00 00 00 00 00 │e⋄⋄⋄⋄⋄⋄⋄┊⋄⋄⋄⋄⋄⋄⋄⋄│
    //   15   │ │00000150│ 00 00 00 00 00 00 00 00 ┊ 00 00 00 00 00 00 00 00 │⋄⋄⋄⋄⋄⋄⋄⋄┊⋄⋄⋄⋄⋄⋄⋄⋄│

    let end_of_volume = start_of_volume + block_size;
    let mut n = 0;
    while file.stream_position().unwrap() < end_of_volume - VtblRecord::SIZE as u64 {
        let mut container = Container::new(file.try_clone().unwrap());
        let save_before_position = file.stream_position().unwrap();
        match VtblRecord::from_reader(&mut container, ()) {
            Ok(vid) => {
                vtable.push((n, vid));
            }
            Err(_) => {
                // rewind
                file.seek(SeekFrom::Start(
                    save_before_position + VtblRecord::SIZE as u64,
                ))
                .unwrap();
            }
        }
        n += 1;
    }

    // UbiBlocks
    println!(
        "{} named volumes, {} physical volumes, blocksize={block_size:#x?}",
        vtable.len(),
        map.len()
    );

    for (volume, v) in vtable {
        println!("Extracting volume: {:?}", v.name().unwrap());

        let extract_map = &map[&volume];
        let mut file_write = File::options()
            .write(true)
            .create(true)
            .open(v.name().unwrap().to_str().unwrap())
            .unwrap();

        for lnum in extract_map {
            let seek = SeekFrom::Start(lnum * block_size);
            // TODO: we already read the ec, cache it
            file.seek(seek).unwrap();
            let mut container = Container::new(file.try_clone().unwrap());
            let ec = EcHdr::from_reader(&mut container, ()).unwrap();

            file.seek(SeekFrom::Start(
                lnum * block_size + u64::from(ec.vid_hdr_offset),
            ))
            .unwrap();
            let mut container = Container::new(file.try_clone().unwrap());
            let vid = VidHdr::from_reader(&mut container, ()).unwrap();

            match vid.vol_type {
                VolType::Static => {
                    file.seek(SeekFrom::Start(lnum * block_size + ec.data_offset as u64))
                        .unwrap();
                    let mut buf = vec![0; vid.data_size as usize];
                    // change to io::copy
                    file.read_exact(&mut buf).unwrap();
                    file_write.write(&buf).unwrap();
                }
                VolType::Dynamic => {
                    file.seek(SeekFrom::Start(lnum * block_size + ec.data_offset as u64))
                        .unwrap();
                    let mut buf = vec![0; block_size as usize - ec.data_offset as usize];
                    // change to io::copy
                    file.read_exact(&mut buf).unwrap();
                    file_write.write(&buf).unwrap();
                }
            }
        }
    }
}

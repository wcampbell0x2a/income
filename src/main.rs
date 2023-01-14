#![feature(cursor_remaining)]

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::io::{Seek, SeekFrom, Write};

use deku::DekuContainerRead;
use income::{EcHdr, VidHdr, VtblRecord};

pub fn main() {
    let block_size = 0x2_0000;

    let mut volume_name = vec![];
    let mut volumes = HashMap::new();

    let fs_bytes = std::fs::read("1294.ubi").unwrap();
    let mut start = 0;

    for _ in 0..(fs_bytes.len() / block_size) {
        let mut bytes = std::io::Cursor::new(&fs_bytes[start..][..block_size]);
        let (_, ec_header) = EcHdr::from_bytes((bytes.get_ref(), 0)).unwrap();

        bytes
            .seek(SeekFrom::Start(ec_header.vid_hdr_offset.into()))
            .unwrap();
        let (_, vid_header) = VidHdr::from_bytes((bytes.remaining_slice(), 0)).unwrap();

        if vid_header.vol_id == 0x7fffefff {
            bytes
                .seek(SeekFrom::Start(ec_header.data_offset.into()))
                .unwrap();
            let mut rest = bytes.remaining_slice();
            for (i, _) in (0..128).enumerate() {
                let (l_rest, v_record) = VtblRecord::from_bytes((rest, 0)).unwrap();
                let name = v_record.name[..v_record.name_len as usize + 1].to_vec();
                volume_name.push(CString::from_vec_with_nul(name).unwrap());
                rest = l_rest.0;
                if v_record.crc == 0xf116c36b {
                    break;
                }
                println!("{v_record:02x?}");
            }
        } else {
            // TODO: data size is only used for crc?
            //if vid_header.data_size != 0 {
            //    let data = &bytes.remaining_slice()[0x800..][..vid_header.data_size as usize];
            //    println!("BYTES: {:02x?}", data.len());
            //    let bytes = volumes.entry(vid_header.vol_id).or_insert(vec![]);
            //    bytes.write_all(data);
            //} else {
            let data = &bytes.remaining_slice()[0x800..];
            let bytes = volumes.entry(vid_header.vol_id).or_insert(vec![]);
            bytes.write_all(data).unwrap();
            //}
        }
        start += block_size;
    }

    for (i, (_, volume)) in volumes.iter().enumerate() {
        let name = volume_name[i].clone().into_string().unwrap();
        println!("wrote: {name}");
        std::fs::write(name, volume).unwrap();
    }
}

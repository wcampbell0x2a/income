#![feature(int_roundings)]

use std::{
    collections::HashMap,
    ffi::CString,
    io::{Read, Seek, SeekFrom, Write},
};

use deku::prelude::*;
use log::{info, trace};

pub const VTBL_VOLID: u32 = 0x7fffefff;
pub const UBI_EC_HDR_MAGIC: &[u8] = b"UBI#";
pub const UBI_VID_HDR_MAGIC: &[u8] = b"UBI!";

/// Erase Counter Header
///
/// See <kernel>/drivers/mtd/ubi/ubi-media.h
#[derive(Debug, DekuRead, DekuWrite, PartialEq, Eq, Clone, Hash)]
#[deku(endian = "big")]
pub struct EcHdr {
    /// magic string `UBI#`
    #[deku(assert_eq = "UBI_EC_HDR_MAGIC")]
    pub magic: [u8; 4],
    /// UBI version meant to accept this image
    pub version: u8,
    /// Reserved for future use, assert zeros
    pub padding1: [u8; 3],
    /// Erase counter
    pub ec: u64,
    /// Where the VID header starts
    pub vid_hdr_offset: u32,
    /// Where user data starts
    pub data_offset: u32,
    /// Image sequence number
    pub image_seq: u32,
    /// Reserved for future use, assert zeros
    pub padding2: [u8; 32],
    /// EC header crc32 checksum
    pub hdr_crc: u32,
}

/// Volume Identifier Header
/// See <kernel>/drivers/mtd/ubi/ubi-media.h
#[derive(Debug, DekuRead, DekuWrite, PartialEq, Eq)]
#[deku(endian = "big")]
pub struct VidHdr {
    /// Magic String `UBI!`
    #[deku(assert_eq = "UBI_VID_HDR_MAGIC")]
    pub magic: [u8; 4],
    /// UBI version meant to accept this image
    #[deku(assert_eq = "1")]
    pub version: u8,
    pub vol_type: VolType,
    /// If this is a copied PEB b/c of wear leveling
    pub copy_flag: u8,
    /// Compatibility of this volume
    pub compat: u8,
    /// ID of this volume
    pub vol_id: u32,
    /// LEB number
    pub lnum: u32,
    /// Reversed for future use, assert zeros
    pub padding1: [u8; 4],
    /// How many bytes of data this contains (static types only)
    pub data_size: u32,
    /// Total number of used LEBs in this volume
    pub used_ebs: u32,
    /// How many bytes at the end of LEB are not used
    pub data_pad: u32,
    /// CRC32 checksum of data, static type only
    pub data_crc: u32,
    /// Reserved for future use, assert zeros
    pub padding2: [u8; 4],
    /// Sequence number
    pub sqnum: u64,
    /// Reserved for future use, assert zeros
    pub padding3: [u8; 12],
    /// VID header CRC32 checksum
    pub hdr_crc: u32,
}

const UBI_VOL_NAME_MAX: usize = 127;

#[derive(Debug, DekuRead, DekuWrite, PartialEq, Eq, Hash)]
#[deku(ctx = "endian: deku::ctx::Endian", endian = "endian")]
#[deku(id_type = "u8")]
pub enum VolType {
    /// Volume can be resized
    Dynamic = 1,
    /// Volume can not be resized
    Static = 2,
}

/// See <kernel>/drivers/mtd/ubi/ubi-media.h
#[derive(Debug, DekuRead, DekuWrite, PartialEq, Eq, Hash)]
#[deku(endian = "big")]
pub struct VtblRecord {
    /// How many PEBs reserved for this volume
    pub reserved_pebs: u32,
    /// Volume alignment
    pub alignment: u32,
    /// Number of unused bytes at end of PEB
    pub data_pad: u32,
    pub vol_type: VolType,
    /// Vol update started but not finished
    pub upd_marker: u8,
    #[deku(assert = "*name_len < UBI_VOL_NAME_MAX as u16")]
    pub name_len: u16,
    pub name: [u8; UBI_VOL_NAME_MAX + 1],
    /// Volume flags
    pub flags: u8,
    /// Reserved for future use, assert zeros
    pub padding: [u8; 23],
    /// Vol record crc32 checksum
    pub crc: u32,
}

impl VtblRecord {
    pub const SIZE: usize = 4 * 4 + UBI_VOL_NAME_MAX + 1 + 24 + 4;

    pub fn name(&self) -> Option<CString> {
        if self.name_len == 0 {
            None
        } else {
            let val = CString::from_vec_with_nul(self.name[..self.name_len as usize + 1].to_vec())
                .unwrap();
            Some(val)
        }
    }
}

const UBI_FM_MAX_BLOCKS: usize = 32;

/// See <kernel>/drivers/mtd/ubi/ubi-media.h
#[derive(Debug, DekuRead, DekuWrite, PartialEq, Eq, Hash)]
#[deku(endian = "big")]
pub struct FmSb {
    pub magic: u32,
    pub version: u8,
    pub padding1: [u8; 3],
    pub data_crc: u32,
    pub used_blocks: u32,
    pub block_loc: [u8; UBI_FM_MAX_BLOCKS],
    pub block_ec: [u8; UBI_FM_MAX_BLOCKS],
    pub sqnum: u64,
    pub padding2: [u8; 32],
}

const UBI_FM_MAX_POOL_SIZE: usize = 256;

/// See <kernel>/drivers/mtd/ubi/ubi-media.h
#[derive(Debug, DekuRead, DekuWrite, PartialEq, Eq, Hash)]
#[deku(endian = "big")]
pub struct FmScanPool {
    pub magic: u32,
    pub size: u16,
    pub pebs: [u32; UBI_FM_MAX_POOL_SIZE],
    pub padding: [u32; 4],
}

#[derive(Debug, DekuRead, DekuWrite, PartialEq, Eq, Hash)]
#[deku(endian = "big")]
pub struct FmEc {
    pub magic: u32,
    pub ec: u32,
}

#[derive(Debug, DekuRead, DekuWrite, PartialEq, Eq, Hash)]
#[deku(endian = "big")]
pub struct FmVolHdr {
    pub magic: u32,
    pub vol_id: u32,
    pub padding1: [u8; 3],
    pub data_pad: u32,
    pub used_ebs: u32,
    pub last_eb_bytes: u32,
    pub padding2: [u8; 8],
}

#[derive(Debug, DekuRead, DekuWrite, PartialEq, Eq, Hash)]
#[deku(endian = "big")]
pub struct FmEba {
    pub magic: u32,
    pub reserved_pebs: u32,
    #[deku(count = "*reserved_pebs")]
    pub pnum: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Image {
    pub vtable: Vec<(u32, VtblRecord)>,
    /// (vol_id, Vec<lnum>)
    pub map: HashMap<u32, Vec<u64>>,
}

impl Image {
    pub fn read<RS: Read + Seek>(reader: &mut RS, file_len: u64, block_size: u64) -> Self {
        let maxlebs = file_len.div_floor(block_size);

        // scanblocks()
        let mut map = HashMap::<u32, Vec<u64>>::new();
        for lnum in 0..maxlebs {
            // Ec
            reader.seek(SeekFrom::Start(lnum * block_size)).unwrap();
            let ec = EcHdr::from_reader((reader, 0)).unwrap().1;

            // Vid
            reader
                .seek(SeekFrom::Start(
                    lnum * block_size + u64::from(ec.vid_hdr_offset),
                ))
                .unwrap();
            let vid = VidHdr::from_reader((reader, 0)).unwrap().1;

            if let Some(x) = map.get_mut(&vid.vol_id) {
                x.push(lnum);
            } else {
                map.insert(vid.vol_id, vec![lnum]);
            }
        }

        // read the volume table
        let lnum = map[&VTBL_VOLID][0]; // TODO: i guess the first entry?
        let start_of_volume = lnum * block_size;
        reader.seek(SeekFrom::Start(start_of_volume)).unwrap();
        let ec = EcHdr::from_reader((reader, 0)).unwrap().1;
        trace!("ec: {ec:02x?}");

        reader
            .seek(SeekFrom::Start(
                lnum * block_size + u64::from(ec.vid_hdr_offset),
            ))
            .unwrap();
        let vid = VidHdr::from_reader((reader, 0)).unwrap().1;
        assert_eq!(vid.lnum as u64, lnum);
        trace!("vid: {vid:02x?}");

        let mut vtable = vec![];
        reader
            .seek(SeekFrom::Start(start_of_volume + ec.data_offset as u64))
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
        while reader.stream_position().unwrap() < end_of_volume - VtblRecord::SIZE as u64 {
            let save_before_position = reader.stream_position().unwrap();
            match VtblRecord::from_reader((reader, 0)) {
                Ok((_, vtbl)) => {
                    trace!("vtbl: {vtbl:02x?}");
                    vtable.push((n, vtbl));
                }
                Err(_) => {
                    // rewind
                    reader
                        .seek(SeekFrom::Start(
                            save_before_position + VtblRecord::SIZE as u64,
                        ))
                        .unwrap();
                }
            }
            n += 1;
        }

        Self { vtable, map }
    }

    /// Read and write volume data
    pub fn read_volume<RS: Read + Seek, W: Write>(
        &self,
        reader: &mut RS,
        writer: &mut W,
        block_size: u64,
        volume: &(u32, VtblRecord),
    ) {
        let (volume_id, volume) = volume;
        info!("Extracting volume: {:?}", volume.name().unwrap());

        let extract_map = &self.map[volume_id];
        for lnum in extract_map {
            let ec_seek = SeekFrom::Start(lnum * block_size);
            // TODO: we already read the ec, cache it
            reader.seek(ec_seek).unwrap();
            let ec = EcHdr::from_reader((reader, 0)).unwrap().1;

            let vid_seek = lnum * block_size + u64::from(ec.vid_hdr_offset);
            reader.seek(SeekFrom::Start(vid_seek)).unwrap();
            let vid = VidHdr::from_reader((reader, 0)).unwrap().1;

            let vol_seek = lnum * block_size + ec.data_offset as u64;
            reader.seek(SeekFrom::Start(vol_seek)).unwrap();
            match vid.vol_type {
                VolType::Static => {
                    let mut buf = vec![0; vid.data_size as usize];
                    // change to io::copy
                    reader.read_exact(&mut buf).unwrap();
                    writer.write_all(&buf).unwrap();
                }
                VolType::Dynamic => {
                    let mut buf = vec![0; block_size as usize - ec.data_offset as usize];
                    // change to io::copy
                    reader.read_exact(&mut buf).unwrap();
                    writer.write_all(&buf).unwrap();
                }
            }
        }
    }
}

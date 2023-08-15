use std::ffi::CString;

use deku::prelude::*;

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
#[deku(type = "u8")]
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

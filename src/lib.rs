use deku::prelude::*;

pub const UBI_EC_HDR_MAGIC: &[u8] = b"UBI#";
pub const UBI_VID_HDR_MAGIC: &[u8] = b"UBI!";

/// Erase Counter Header
///
/// See <kernel>/drivers/mtd/ubi/ubi-media.h
#[derive(Debug, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct EcHdr {
    #[deku(assert_eq = "UBI_EC_HDR_MAGIC")]
    pub magic: [u8; 4],
    pub version: u8,
    pub padding1: [u8; 3],
    pub ec: u64,
    pub vid_hdr_offset: u32,
    pub data_offset: u32,
    pub image_seq: u32,
    pub padding2: [u8; 32],
    pub hdr_crc: u32,
}

/// Volume Identifier Header
/// See <kernel>/drivers/mtd/ubi/ubi-media.h
#[derive(Debug, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct VidHdr {
    #[deku(assert_eq = "UBI_VID_HDR_MAGIC")]
    pub magic: [u8; 4],
    #[deku(assert_eq = "1")]
    pub version: u8,
    pub vol_type: u8,
    pub copy_flag: u8,
    pub compat: u8,
    pub vol_id: u32,
    pub lnum: u32,
    pub padding1: [u8; 4],
    pub data_size: u32,
    pub used_ebs: u32,
    pub data_pad: u32,
    pub data_crc: u32,
    pub padding2: [u8; 4],
    pub sqnum: u64,
    pub padding3: [u8; 12],
    pub hdr_crc: u32,
}

const UBI_VOL_NAME_MAX: usize = 127;

/// See <kernel>/drivers/mtd/ubi/ubi-media.h
#[derive(Debug, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct VtblRecord {
    pub reserved_pebs: u32,
    pub alignment: u32,
    pub data_pad: u32,
    pub vol_type: u8,
    pub upd_marker: u8,
    pub name_len: u16,
    pub name: [u8; UBI_VOL_NAME_MAX + 1],
    pub flags: u8,
    pub padding: [u8; 23],
    pub crc: u32,
}

const UBI_FM_MAX_BLOCKS: usize = 32;

/// See <kernel>/drivers/mtd/ubi/ubi-media.h
#[derive(DekuRead, DekuWrite)]
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
#[derive(DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct FmScanPool {
    pub magic: u32,
    pub size: u16,
    pub pebs: [u32; UBI_FM_MAX_POOL_SIZE],
    pub padding: [u32; 4],
}

#[derive(DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct FmEc {
    pub magic: u32,
    pub ec: u32,
}

#[derive(DekuRead, DekuWrite)]
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

#[derive(DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct FmEba {
    pub magic: u32,
    pub reserved_pebs: u32,
    #[deku(count = "*reserved_pebs")]
    pub pnum: Vec<u8>,
}

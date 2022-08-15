use binrw::{BinRead, BinWrite};

#[derive(BinRead, BinWrite, Debug)]
#[brw(magic(b"AR\0\0"))]
pub struct ArfHeader {
    pub file_count: u32,
    pub xor_key: [u8; 4],
    pub directory_offset: u32,
}

#[derive(BinRead, BinWrite, Debug)]
pub struct ArfFileDirectoryEntry {
    pub id: u16,
    pub unknown: u16,
    pub offset: u32,
    pub length: u32,
    pub file_xor_key: u8,
    pub padding: [u8; 3],
}

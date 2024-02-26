/// The block that is written to a file
/// ----------------------------------------------------------------------------------------------------------------------------------------
/// |           Header(8 bytes)                   |   Data Section (var length)   |               Index               |  Footer (4 bytes)  |
/// ----------------------------------------------------------------------------------------------------------------------------------------
/// |Block size (4 bytes) | Number of entries (4B)| Entry | Entry | Entry | Entry | Offset | Offset | Offset | Offset |  Checksum (4 bytes)|
/// -----------------------------------------------------------------------------------------------------------------------------------------

/// The entry struct
/// ---------------------------------------------------------------
/// | key length (4 bytes) | key | value length (4 bytes) | value |
//  ---------------------------------------------------------------

/// The offset struct
/// -------------------
/// | Offset (4 bytes)|
/// -------------------

const BLOCKSIZE: usize = 16 * 1024; //  on a mac PAGESIZE comes out to be 16kb
const HEADER_SIZE: usize = 8;
const FOOTER_SIZE: usize = 4;
pub const ENTRY_SIZE_AVAILABLE: usize = BLOCKSIZE - HEADER_SIZE - FOOTER_SIZE; //  the space available for entries
pub struct Block {
    data: Vec<u8>,
    offsets: Vec<u32>,
    num_of_entries: u32,
}

impl Block {
    pub fn new() -> Block {
        let data = Vec::new();
        let offsets = Vec::new();
        Block {
            data,
            offsets,
            num_of_entries: 0,
        }
    }

    pub fn add(&mut self, key: &str, value: &str) -> bool {
        let key_bytes = key.as_bytes();
        let value_bytes = value.as_bytes();
        let key_length = key_bytes.len() as u32;
        println!("Key length {}", key_length);
        let value_length = value_bytes.len() as u32;
        println!("Value length {}", value_length);
        let entry_size = 4 + key_length as usize + 4 + value_length as usize; //  28 bytes

        let size_so_far = self.data.len() + self.offsets.len() * 4;
        let offset_size = 4;
        let size_required_for_entry = offset_size + entry_size;
        println!(
            "Size so far: {}, size_required: {}, ENTRY_SIZE_AVAILABLE: {}",
            size_so_far, size_required_for_entry, ENTRY_SIZE_AVAILABLE
        );

        if size_so_far + size_required_for_entry > ENTRY_SIZE_AVAILABLE {
            //  cannot write to this block as the block is full
            return false;
        }

        let offset = size_so_far as u32;
        self.offsets.push(offset);
        self.num_of_entries += 1;
        self.data.extend_from_slice(&key_length.to_be_bytes());
        self.data.extend_from_slice(&key_bytes);
        self.data.extend_from_slice(&value_length.to_le_bytes());

        true
    }
}

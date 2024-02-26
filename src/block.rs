/// The block that is written to a file
/// -----------------------------------------------------------------------------------------------------------------------------------------------------------------
/// |           Header(8 bytes)                   |   Data Section (var length)   |               Index               |                 Footer (8 bytes)            |
/// -----------------------------------------------------------------------------------------------------------------------------------------------------------------
/// |Block size (4 bytes) | Number of entries (4B)| Entry | Entry | Entry | Entry | Offset | Offset | Offset | Offset |  Index offset (4 bytes) | Checksum (4 bytes)|
/// -----------------------------------------------------------------------------------------------------------------------------------------------------------------

/// The entry struct
/// ---------------------------------------------------------------
/// | key length (4 bytes) | key | value length (4 bytes) | value |
//  ---------------------------------------------------------------
use std::fmt::Error;

/// The offset struct
/// -------------------
/// | Offset (4 bytes)|
/// -------------------

const BLOCKSIZE: usize = 16 * 1024; //  on a mac PAGESIZE comes out to be 16kb
const HEADER_SIZE: usize = 8;
const FOOTER_SIZE: usize = 4;
pub const SIZE_AVAILABLE: usize = BLOCKSIZE - HEADER_SIZE - FOOTER_SIZE; //  the space available for entries + index
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

    fn calculate_checksum(&self, encoded_data: &Vec<u8>) -> u32 {
        let mut sum: u32 = 0;
        for byte in encoded_data {
            sum += *byte as u32;
        }
        sum
    }

    pub fn add(&mut self, key: &str, value: &str) -> bool {
        let key_bytes = key.as_bytes();
        let value_bytes = value.as_bytes();
        let key_length = key_bytes.len() as u32;
        let value_length = value_bytes.len() as u32;
        let entry_size = 4 + key_length as usize + 4 + value_length as usize; //  28 bytes

        let size_so_far = self.data.len() + self.offsets.len() * 4;
        let offset_size = 4;
        let size_required_for_entry = offset_size + entry_size;

        if size_so_far + size_required_for_entry > SIZE_AVAILABLE {
            //  cannot write to this block as the block is full
            return false;
        }

        let offset = self.data.len() as u32;
        self.offsets.push(offset);
        self.num_of_entries += 1;
        self.data.extend_from_slice(&key_length.to_le_bytes());
        self.data.extend_from_slice(&key_bytes);
        self.data.extend_from_slice(&value_length.to_le_bytes());
        self.data.extend_from_slice(&value_bytes);

        true
    }

    fn encode(&self) -> Vec<u8> {
        let mut encoded_data = Vec::new();

        //  encode the header
        let block_size = BLOCKSIZE as u32; //  placeholder
        encoded_data.extend_from_slice(&block_size.to_le_bytes());
        encoded_data.extend_from_slice(&self.num_of_entries.to_le_bytes());

        //  encode the entries
        encoded_data.extend(&self.data);

        let index_offset = encoded_data.len() as u32;

        //  encode the offsets
        for offset in &self.offsets {
            encoded_data.extend_from_slice(&offset.to_le_bytes());
        }

        //  calculate the block size and replace the first 4 bytes with this value
        let block_size = encoded_data.len() as u32 + 8; //  header + data + offsets + footer
        encoded_data[0] = block_size as u8;
        encoded_data[1] = (block_size >> 8) as u8;
        encoded_data[2] = (block_size >> 16) as u8;
        encoded_data[3] = (block_size >> 24) as u8;

        //  encode the footer
        encoded_data.extend_from_slice(&index_offset.to_le_bytes());
        let checksum = self.calculate_checksum(&encoded_data);
        encoded_data.extend_from_slice(&checksum.to_le_bytes());
        encoded_data
    }

    fn decode(&self, data: Vec<u8>) -> Result<Block, Error> {
        //  calculate the checksum
        let data_slice_without_checksum = &data[..data.len() - 4].to_vec();
        let calculated_checksum = self.calculate_checksum(data_slice_without_checksum);

        //  decode the header
        let block_size_offset = 0;
        let num_entries_offset = 4;
        let block_size = u32::from_le_bytes([
            data[block_size_offset],
            data[block_size_offset + 1],
            data[block_size_offset + 2],
            data[block_size_offset + 3],
        ]);
        let num_of_entries = u32::from_le_bytes([
            data[num_entries_offset],
            data[num_entries_offset + 1],
            data[num_entries_offset + 2],
            data[num_entries_offset + 3],
        ]);

        //  decode the footer
        let footer_offset = block_size as usize - 8;
        let index_offset_offset = footer_offset;
        let checksum_offset = footer_offset + 4;
        let checksum = u32::from_le_bytes([
            data[checksum_offset],
            data[checksum_offset + 1],
            data[checksum_offset + 2],
            data[checksum_offset + 3],
        ]);
        let index_offset = u32::from_le_bytes([
            data[index_offset_offset],
            data[index_offset_offset + 1],
            data[index_offset_offset + 2],
            data[index_offset_offset + 3],
        ]);
        assert_eq!(calculated_checksum, checksum);

        //  read the offsets
        let mut offsets = Vec::new();
        let mut data_offset = index_offset as usize;
        while data_offset < footer_offset {
            let offset = u32::from_le_bytes([
                data[data_offset],
                data[data_offset + 1],
                data[data_offset + 2],
                data[data_offset + 3],
            ]);
            offsets.push(offset);
            data_offset += 4;
        }

        //  read the data
        let mut decoded_data: Vec<u8> = Vec::new();
        for offset in offsets.clone() {
            let data_offset = HEADER_SIZE + offset as usize;
            let key_length = u32::from_le_bytes([
                data[data_offset as usize],
                data[data_offset as usize + 1],
                data[data_offset as usize + 2],
                data[data_offset as usize + 3],
            ]);
            let key =
                &data[data_offset as usize + 4..data_offset as usize + 4 + key_length as usize];
            let value_length = u32::from_le_bytes([
                data[data_offset as usize + 4 + key_length as usize],
                data[data_offset as usize + 4 + key_length as usize + 1],
                data[data_offset as usize + 4 + key_length as usize + 2],
                data[data_offset as usize + 4 + key_length as usize + 3],
            ]);
            let value = &data[data_offset as usize + 4 + key_length as usize + 4
                ..data_offset as usize + 4 + key_length as usize + 4 + value_length as usize];
            decoded_data.extend_from_slice(&key_length.to_le_bytes());
            decoded_data.extend_from_slice(&key);
            decoded_data.extend_from_slice(&value_length.to_le_bytes());
            decoded_data.extend_from_slice(&value);
        }

        let mut block = Block::new();
        block.data = decoded_data;
        block.offsets = offsets;
        block.num_of_entries = num_of_entries;
        Ok(block)
    }
}

#[cfg(test)]
use crate::test_rig::generate_key_value_pairs;
#[test]
fn check_max_entries_in_block() {
    let max_num_of_entries = SIZE_AVAILABLE / 32; //  32 bytes is the size of each entry when generated by the test rig
    let entries = generate_key_value_pairs(max_num_of_entries);
    let mut block = Block::new();
    for entry in entries {
        assert_eq!(block.add(&entry.0, &entry.1), true);
    }
    assert_eq!(block.add("key_0", "value_0"), false);
}

#[test]
fn check_encoding_decoding() {
    let num_of_entries = 500;
    let entries = generate_key_value_pairs(num_of_entries);
    let mut block = Block::new();
    for entry in entries {
        block.add(&entry.0, &entry.1);
    }
    let encoded_data = block.encode();
    let decoded_data = Block::decode(&block, encoded_data).unwrap();
    assert_eq!(block.num_of_entries, decoded_data.num_of_entries);

    //  check that the data is the same
    for i in 0..block.data.len() {
        assert_eq!(block.data[i], decoded_data.data[i]);
    }
}

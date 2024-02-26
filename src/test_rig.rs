use crate::block::{Block, ENTRY_SIZE_AVAILABLE};

pub fn generate_key_value_pairs() {
    let mut block = Block::new();
    let total_possible_entries = ENTRY_SIZE_AVAILABLE / 21;
    println!("Total possible entries {}", total_possible_entries);
    for i in 0..total_possible_entries {
        let key: &str = &format!("key_{:<05}", i);
        let value: &str = &format!("value_{:<05}", i);
        assert_eq!(block.add(key, value), true);
    }
    //  should not be able to add any more key value pairs
    assert_eq!(block.add("key_00000", "value_00000"), false);
}

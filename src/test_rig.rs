pub fn generate_key_value_pairs(num_of_entries: usize) -> Vec<(String, String)> {
    let mut vec: Vec<(String, String)> = Vec::new();
    for i in 0..num_of_entries {
        let key: &str = &format!("key_{:<05}", i);
        let value: &str = &format!("value_{:<05}", i);
        vec.push((key.to_string(), value.to_string()));
    }
    vec
}

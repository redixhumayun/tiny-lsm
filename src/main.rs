use test_rig::generate_key_value_pairs;

pub mod block;
mod test_rig;

fn main() {
    let mut entries = generate_key_value_pairs(2);
}

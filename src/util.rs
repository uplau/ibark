pub fn hash_hex_string<T: std::hash::Hash>(item: T) -> String {
    use std::{collections::hash_map::DefaultHasher, hash::Hasher};

    let mut hasher = DefaultHasher::new();
    item.hash(&mut hasher);
    let ret = hasher.finish();
    format!("{:X}", ret)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use rand::Rng;

    pub fn random_string(len: usize) -> String {
        rand::thread_rng()
            .sample_iter(rand::distributions::Alphanumeric)
            .take(len)
            .map(char::from)
            .collect()
    }

    pub fn random_number(min: u32, max: u32) -> u32 {
        let mut rng = rand::thread_rng();
        rng.gen_range(min..=max)
    }

    #[test]
    fn test_hash_hex_string() {
        dbg!(hash_hex_string(random_number(0, 100) as usize));
    }

    #[test]
    fn test_random() {
        dbg!(random_string(random_number(0, 100) as usize));
    }
}

use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

pub const DEPULICATION_BUFFER_SIZE: usize = 30;

pub struct Filter {
    pub last_elements: [u64; DEPULICATION_BUFFER_SIZE], // vector of hashes for deduplication
    pub iterator: usize,                                // keeps track of the oldest element
}

impl Filter {
    pub fn new() -> Filter {
        Filter {
            last_elements: [0; DEPULICATION_BUFFER_SIZE],
            iterator: 0,
        }
    }

    pub async fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }
}

// checks if the given telegram hash is contained in the filter class
pub async fn deduplicate(filter: &mut Filter, telegram_hash: u64) -> bool {
    // checks if the given telegram is already in the buffer
    let contained = filter.last_elements.contains(&telegram_hash);

    if contained {
        return true;
    }

    // updates the buffer adding the new telegram
    let index = filter.iterator;
    filter.last_elements[index] = telegram_hash;
    filter.iterator = (filter.iterator + 1) % DEPULICATION_BUFFER_SIZE;

    contained
}


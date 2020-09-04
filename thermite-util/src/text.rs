use rand::{
    Rng,
    thread_rng,
    distributions::Alphanumeric
};

use std::{
    iter,
    collections::HashSet,
};


pub fn random_alphanum(length: usize) -> String {
    let mut rng = thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(length)
        .collect()
}

pub fn repeat_string(src: &str, count: usize) -> String {
    iter::repeat(src).take(count).collect::<String>()
}

pub fn generate_id(count: usize, existing: &HashSet<String>) -> String {
    let mut new_id = random_alphanum(count);
    while existing.contains(&new_id) {
        new_id = random_alphanum(count);
    }
    new_id
}
use rand::{
    Rng,
    thread_rng,
    distributions::Alphanumeric
};

use std::iter;


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
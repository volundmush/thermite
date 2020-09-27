use rand::{
    Rng,
    thread_rng,
    distributions::Alphanumeric
};

use std::{
    iter,
    collections::HashSet,
    rc::Rc,
    ops::Range,
    hash::Hash
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


#[derive(Default, Debug)]
pub struct StringInterner {
    storage: String,
    index: Vec<Range<usize>>,
}

impl StringInterner {
    pub fn get(&self, idx: usize) -> &str {
        // this will PANIC if a string doesn't exist...
        &self.storage[self.index.get(idx).unwrap().clone()]
    }

    fn intern(&mut self, src: &str) -> usize {
        // First we scan the string for a matching sub-string. if found, we index it and all is well
        let r =
        if let Some(pos) = self.storage.find(src) {
            pos..pos+src.len()
        } else {
            // no substring was found, so we'll append.
            self.storage.push_str(src);
            self.storage.len()-src.len()..self.storage.len()
        };
        self.index.push(r);
        self.index.len()-1
    }

    pub fn get_or_intern(&mut self, src: &str) -> usize {
        // interns or retrieves a string as necessary.
        if let Some(i) = self.contains(src) {
            return i
        }
        self.intern(src)
    }

    pub fn contains(&self, src: &str) -> Option<usize> {
        for (i, n) in self.index.iter().enumerate() {
            if self.storage[n.clone()] == *src {
                return Some(i)
            }
        }
        None
    }

    // You better be damned sure that these idx keys actually are in this interner for the search
    // functions.
    pub fn exact_match(&self, idx: &[usize], pat: &str) -> Option<usize> {
        for i in idx {
            if self.get(*i) == pat {
                return Some(*i)
            }
        }
        None
    }

    pub fn ci_exact_match(&self, idx: &[usize], pat: &str) -> Option<usize> {
        for i in idx {
            if self.get(*i).to_uppercase() == pat.to_uppercase() {
                return Some(*i)
            }
        }
        None
    }

    // these attempt to sort the matches alphanuerically then by length, shortest possible match first.
    pub fn ci_partial_match(&self, idx: &[usize], pat: &str) -> Vec<usize> {
        let mut out: Vec<usize> = Default::default();
        let upper = pat.to_uppercase();
        for i in idx {
            if self.get(*i).to_uppercase().starts_with(&upper) {
                out.push(*i)
            }
        }
        out.sort_by(|a, b| self.get(*a).len().cmp(&self.get(*b).len()));
        out
    }

    pub fn partial_match(&self, idx: &[usize], pat: &str) -> Vec<usize> {
        let mut out: Vec<usize> = Default::default();
        for i in idx {
            if self.get(*i).starts_with(pat) {
                out.push(*i)
            }
        }
        out.sort_by(|a, b| self.get(*a).len().cmp(&self.get(*b).len()));
        out
    }
}
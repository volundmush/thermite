use std::{
    ops::Range,
};

pub fn range_from_idx(idx: Vec<usize>) -> Vec<Range<usize>> {
    let mut out: Vec<Range<usize>> = Default::default();

    for (i, rng) in idx.as_slice().windows(2).enumerate() {
        if i > 0 {
            out.push(rng[0]+1..rng[1])
        } else {
            out.push(rng[0]..rng[1])
        }
        
    }
    out
}

pub fn split_action_list(text: &str) -> Vec<Range<usize>> {
    let mut idx: Vec<usize> = Default::default();
    let mut escaped = false;
    let mut depth: u16 = 0;
    let length = text.len();

    idx.push(0);

    for (i, c) in text.chars().enumerate() {
        if escaped {
            escaped = false;
        } else {
            match c {
                '\\' => {
                    escaped = true;
                },
                '{' => {
                    depth += 1;
                },
                '}' => {
                    depth -= 1;
                },
                ';' => {
                    if depth == 0 && i != length {
                        // We will be adding the final spot as an index point
                        // anyways.
                        idx.push(i)
                    }
                },
                _ => {}
            }
            
        }
    }
    idx.push(length);

    range_from_idx(idx)
}

pub fn split_argument_list(text: &str) -> Vec<Range<usize>> {

    let mut escaped = false;
    let mut paren_depth: u16 = 0;
    let mut square_depth: u16 = 0;
    let length = text.len();
    let mut idx: Vec<usize> = Default::default();
    idx.push(0);

    for (i, c) in text.chars().enumerate() {
        if escaped {
            escaped = false;
        } else {
            match c {
                '\\' => {
                    escaped = true;
                },
                '[' => {
                    square_depth += 1;
                },
                ']' => {
                    square_depth -= 1;
                },
                '(' => {
                    paren_depth += 1;
                },
                ')' => {
                    paren_depth -= 1;
                }
                ',' => {
                    if square_depth == 0 && paren_depth == 0 && i != length {
                        // We will be adding the final spot as an index point
                        // anyways.
                        idx.push(i)
                    }
                },
                _ => {}
            }
            
        }
    }
    idx.push(length);

    range_from_idx(idx)
}

pub fn identify_function_squares(text: &str) -> Vec<Range<usize>> {
    let mut idx: Vec<usize> = Default::default();
    let mut escaped = false;
    let mut square_start: usize = 0;
    let mut paren_depth: u16 = 0;
    let mut square_depth: u16 = 0;

    for (i, c) in text.chars().enumerate() {
        if escaped {
            escaped = false;
        } else {
            match c {
                '\\' => {
                    escaped = true;
                },
                '[' => {
                    if square_depth == 0 {
                        square_start = i;
                    }
                    square_depth += 1;
                },
                ']' => {
                    if square_depth == 1 {
                        idx.push(square_start);
                        idx.push(i+1);
                    }
                    square_depth -= 1;
                },
                '(' => {
                    paren_depth += 1;
                },
                ')' => {
                    paren_depth -= 1;
                }
                _ => {}
            }
        }
    }

    let mut out: Vec<Range<usize>> = Default::default();

    for section in idx.as_slice().chunks(2) {
        out.push(section[0]..section[1])
    }
    out
}
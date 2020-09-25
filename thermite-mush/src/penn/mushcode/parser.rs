use std::{
    ops::Range,
};
use regex::{Regex, Match};

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

pub fn identify_function_squares(text: &str) -> Option<Vec<Range<usize>>> {
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

    if idx.len() == 0 {
        return None
    }
    let mut out: Vec<Range<usize>> = Default::default();

    for section in idx.as_slice().chunks(2) {
        out.push(section[0]..section[1])
    }
    Some(out)
}

pub fn find_end_function(text: &str) -> Option<usize> {
    // called by eval_str to determine the end position of a function at the beginning of a string.
    let mut entered = false;
    let mut paren_depth: u16 = 0;
    let mut square_depth: u16 = 0;
    let mut escaped = false;

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
                    if !entered {
                        entered = true;
                    }
                    paren_depth += 1;
                },
                ')' => {
                    if entered && paren_depth == 1 {
                        return Some(i)
                    }
                    paren_depth -= 1;
                }
                _ => {}
            }
        }
    }
    None
}

pub fn eval_squares(text: &str) -> String {
    // This is given arbitrary text to check for []-nested sections to recursively parse.

    if let Some(ranges) = identify_function_squares(text) {
        let mut out = String::from(text);
        // We will perform eval-replacements in reverse to avoid screwing up the offsets.
        for r in ranges {
            let section = &text[r.clone()];
            if section.len() > 2 {
                // Chop off the enclosing []...
                let internal = &section[1..section.len()-1];
                out.replace_range(r, eval_squares(internal).as_str());
            } else {
                // Since this is just an empty [] we replace it with nothing.
                out.replace_range(r, "");
            }
        }
        eval_str(out.as_str())
    } else {
        // Since there are no enclosed [] sections, we will just perform normal evaluation.
        eval_str(text)
    }
}

pub fn eval_substitutions(text: &str) -> String {
    // currently does nothing.
    return String::from(text);
}

pub fn eval_function(name: &str, text: &str) -> String {
    let arguments_raw = &text[name.len()+1..text.len()-1];
    let split_args_ranges = split_argument_list(arguments_raw);
    let mut parsed_args: Vec<String> = Default::default();
    for r in split_args_ranges {
        parsed_args.push(eval_squares(&arguments_raw[r]));
    }
    format!("(FUNCTION CALL: {} - {:?})", name, parsed_args)
}

pub fn eval_str(text: &str) -> String {
    // This is called after no nested-square-bracket sections remain to parse on the text.

    // Later I will move this regex outside the function so it's not constantly compiled.
    let mut re = Regex::new(r"^(?P<funcname>\w+)\(").unwrap();

    // if this string begins with an unbroken alphanumeric + _ sequence followed by a parentheses,
    // it is a function.
    if let Some(found) = re.find(text) {
        if let Some(end) = find_end_function(text) {
            // This definitely begins with a function.
            let (func, rest) = text.split_at(end+1);
            let name = &func[found.start()..found.end()-1];
            return format!("{}{}", eval_function(name, func), eval_substitutions(rest))
        } else {
            // Although it looked like it was a function, it's not terminated.
            eval_substitutions(text)
        }
    } else {
        // There's no functions here, so we'll just perform substitution evaluation.
        eval_substitutions(text)
    }
}
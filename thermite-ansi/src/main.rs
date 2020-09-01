use textwrap;

fn main() {
    let mut disp = String::from("\x1b[31mTHIS TEXT IS SUPER DUPER\x1b[32m RED!\x1b[33m WHATCHA GONNA DO ABOUT IT?\x1b[0m");
    let filled = textwrap::fill(&disp, 10);
    println!("{:?}", filled);
    println!("{}", filled);
}
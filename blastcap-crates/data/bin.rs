use std::path::PathBuf;

fn main() {
    let info = PathBuf::from(std::env::args().nth(1).unwrap());
    let info = data::load(&info).unwrap();
    println!("{info:#?}");
}

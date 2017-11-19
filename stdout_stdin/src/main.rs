use std::io::*;
use std::str::*;

fn read<T: FromStr>() -> Option<T> {
    let stdin = stdin();
    let s = stdin
        .bytes()
        .map(|c| c.unwrap() as char)
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| !c.is_whitespace())
        .collect::<String>();
    s.parse::<T>().ok()
}

fn main() {
    let mut acc: u32 = 0;
    while let Some(n) = read() {
        acc += n;
    }
    assert_eq!(acc, 30000000);
}

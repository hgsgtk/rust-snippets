use std::io::*;
use std::str::*;

fn read<T: FromStr>(s: &mut StdinLock) -> Option<T> {
    let s = s.by_ref()
        .bytes()
        .map(|c| c.unwrap() as char)
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| !c.is_whitespace())
        .collect::<String>();
    s.parse::<T>().ok()
}

fn main() {
    let s = stdin();
    let mut s = s.lock();
    let s = &mut s;
    let mut acc: u32 = 0;
    while let Some(n) = read(s) {
        acc += n;
    }
    assert_eq!(30_000_000, acc);
}

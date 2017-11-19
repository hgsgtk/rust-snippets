fn main() {
    let x = "Sort string.";
    let mut chars: Vec<_> = x.chars().collect();
    chars.sort();
    let y: String = chars.into_iter().collect();
    println!("{}", y);
}

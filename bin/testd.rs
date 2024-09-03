fn main() {
    let n = std::env::args().nth(1).unwrap().parse::<u64>().unwrap();
    let ms = std::env::args().nth(2).unwrap().parse::<u64>().unwrap();
    let mut i: u64 = 0;
    while i < n {
        println!("Line {}/{}", i, n);
        i += 1;
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
}

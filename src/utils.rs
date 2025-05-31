pub fn print_num(title: &'static str, num: usize) {
    println!("{}", title);
    println!("Base 10:               {}", num);
    println!("Base 2 (binary):       {:b}", num);
    println!("Base 8 (octal):        {:o}", num);
    println!("Base 16 (hexadecimal): {:x}", num);
    if num != 0 {
        println!("Boolean (bit):         1");
    } else {
        println!("Boolean (bit):         0");
    }
    println!();
}

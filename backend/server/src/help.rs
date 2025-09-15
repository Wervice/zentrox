use std::process::exit;

pub fn print() {
    println!("Zentrox");
    println!("--help:\t\tPrint this help.");
    println!("--docs <Path | None>:\t\tGenerate OpenAPI docs.");
    exit(0)
}

extern crate save_parser;

use std::env;
use std::process;

use save_parser::Config;


fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });
    if let Err(e) = save_parser::run(config) {
        println!("Application error: {}", e);
        process::exit(1);
    }
//    bench_parse_and_dump();

}

fn bench_parse_and_dump() {
    use std::time::{Instant};
    let conf = Config {
        filename: String::from("big-file.txt"),
        output_file: String::from("example.json"),
    };
    let now = Instant::now();
    let result = save_parser::run(conf);
    println!("{}", now.elapsed().subsec_nanos() as f64 / 1000000000.0);
}

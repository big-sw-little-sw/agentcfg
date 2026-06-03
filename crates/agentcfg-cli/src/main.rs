use agentcfg_core::greet;
use std::env;

fn main() {
    let name = env::args().nth(1).unwrap_or_else(|| "world".into());
    println!("{}", greet(&name));
}

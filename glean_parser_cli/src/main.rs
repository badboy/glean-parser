use std::env;

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();

    let file = &args[0];
    let data = glean_parser::validate(file).unwrap();
    println!("{:?}", data);

    //glean_parser::run();
}

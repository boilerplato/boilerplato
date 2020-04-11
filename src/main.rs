extern crate boilerplato;
use clap::{App, Arg};

fn main() {
    let matches = App::new("boilerplato")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Rousan Ali <rousanali786@gmail.com>")
        .about("A CLI to generate app's source code from a template")
        .arg(Arg::with_name("file").value_name("PATH").index(1).required(true))
        .get_matches();

    let file_path = matches.value_of("file").unwrap();
    let file_size = match rust_cli_boilerplate::get_file_size(file_path) {
        Ok(file_size) => file_size,
        Err(_) => {
            eprintln!("error: File does not exist or can't access");
            return;
        }
    };

    println!("File size: {} B", file_size);
}

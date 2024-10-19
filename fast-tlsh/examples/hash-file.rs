use std::env;

use tlsh::GeneratorOrIOError;
use tlsh::FuzzyHashType;

fn main() {
    type TlshType = tlsh::hashes::Normal;
    let width = TlshType::LEN_IN_STR;
    for filename in env::args().skip(1) {
        let result = match tlsh::hash_file_for::<TlshType, _>(&filename) {
            Ok(x) => x.to_string(),
            Err(x) => String::from(match x {
                GeneratorOrIOError::GeneratorError(_) => "TNULL",
                GeneratorOrIOError::IOError(_) => "IOERR",
            }),
        };
        println!("{result:width$} {filename}");
    }
}

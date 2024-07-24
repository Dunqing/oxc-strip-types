// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_strip_types --example strip_types`
// or `just watch "run -p oxc_strip_types --example strip_types"`

use std::{env, path::Path};

use oxc_strip_types::{StripTypes, StripTypesOptions};

fn main() {
    let name = env::args().nth(1).unwrap_or_else(|| "test.ts".to_string());
    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path).expect("{name} not found");

    println!("Original:\n");
    println!("{source_text}\n");

    let options = StripTypesOptions { replace_with_space: false };
    let ret = StripTypes::new(source_text.clone(), name, options).strip();

    if !ret.errors.is_empty() {
        println!("Strip types failed:\n");
        for error in ret.errors {
            let error = error.with_source_code(source_text.clone());
            println!("{error:?}");
        }
        return;
    }

    println!("Strip types succeeded:\n");
    println!("{}", ret.code);
}

//! PROTOTYPE for #1015, never merge: run main's real selection over each file
//! named on the command line, and print one line per test:
//! `<path>\t<function>\t<selected|refused>\t<reasons joined by "; ">`.

use std::path::Path;

fn main() {
    for arg in std::env::args().skip(1) {
        let path = Path::new(&arg);
        let module = match purba_parsing::parse(path) {
            Ok(module) => module,
            Err(_) => {
                println!("{arg}\t-\tparse-error\t");
                continue;
            }
        };
        let (tests, refused) = purba_selection::select(path, &module);
        for test in &tests {
            println!("{arg}\t{}\tselected\t", test.function());
        }
        for one in &refused {
            println!("{arg}\t{}\trefused\t{one}", one.test().function());
        }
    }
}

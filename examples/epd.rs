use edid::parse;

use nom::Err::{Incomplete, Error, Failure};

fn main() {

    let d = include_bytes!("../testdata/card0-eDP-1");

    match parse(d) {
        Ok((_, parsed)) => {
            println!("parsed: {:#?}", parsed);
        },
        Err(Incomplete(_needed)) => { 
            panic!("Incomplete");
         },
        Err(Error(e)) | Err(Failure(e)) => { 
            panic!("{}", format!("{:?}", e));
         }
    }
}
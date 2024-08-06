use edidr::parse;

use nom::Err::{Incomplete, Error, Failure};

fn main() {

    let d = include_bytes!("../testdata/card0-HDMI-1.bin");

    match parse(d) {
        Ok((_, parsed)) => {
            println!("parsed: {:#?}", parsed);
        },
        Err(Incomplete(_needed)) => { 
            panic!("Incomplete: {:#?}", _needed);
         },
        Err(Error(e)) | Err(Failure(e)) => { 
            panic!("{}", format!("{:?}", e));
         }
    }

    let d = include_bytes!("../testdata/card0-HDMI-2.bin");

    match parse(d) {
        Ok((_, parsed)) => {
            println!("parsed: {:#?}", parsed);
        },
        Err(Incomplete(_needed)) => { 
            panic!("Incomplete: {:#?}", _needed);
         },
        Err(Error(e)) | Err(Failure(e)) => { 
            panic!("{}", format!("{:?}", e));
         }
    }
}
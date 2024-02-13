#[cfg(test)]

use crate::Location;



#[test]
fn test_loc() {
    let loc: Location = match Location::parse("chr1:100000-100t00") {
        Ok(loc)=>loc,
        Err(err)=>panic!("{}", err)
    };

    println!("aha {} {} {}", loc.chr, loc.start, loc.end);

}
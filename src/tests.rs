#[cfg(test)]

use crate::Location;



#[test]
fn test_loc() {
    let loc: Location = Location::parse("chr1:100000-100t00");

    println!("aha {} {} {}", loc.chr, loc.start, loc.end);

}
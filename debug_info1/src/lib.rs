#[macro_use]
extern crate debug_info;
// use debug_info::debug_println;


pub fn aa() {
    let r: Option<usize> = None;
    let x = match r {
        Some(r) => (),
        None => debug_println!("xxx"),
    };
    // debug_println!("xxx");
}
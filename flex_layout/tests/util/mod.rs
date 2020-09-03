use layout::tree::LayoutR;

pub fn print(count: &mut usize, id: usize, layout: &LayoutR) {
    *count += 1;
    debug_println!("result: {:?} {:?} {:?}", *count, id, layout);
}

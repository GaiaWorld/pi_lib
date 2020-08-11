use layout::tree::LayoutR;

pub fn print(count: &mut usize, id: usize, layout: &LayoutR) {
    *count += 1;
    println!("result: {:?} {:?} {:?}", *count, id, layout);
}

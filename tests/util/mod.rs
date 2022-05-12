use layout::tree::LayoutR;

pub fn print(count: &mut usize, id: usize, layout: &LayoutR) {
    *count += 1;
   unsafe{debugit::debugit!("result: {:?} {:?} {:?}", *count, id, layout);
}

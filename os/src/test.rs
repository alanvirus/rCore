use std::cell::RefCell;
static A: RefCell<i32> = RefCell::new(3);
fn main() {
    *A.borrow_mut() = 4;
    println!("{}", A.borrow());
}
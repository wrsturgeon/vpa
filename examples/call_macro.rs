use vpa::call;

fn main() {
    // Take a look inside:
    println!("{:#?}", call!(|x: ()| x))
}

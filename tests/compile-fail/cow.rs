#![feature(plugin)]
#![plugin(clippy)]

#![deny(got_milk)]
fn main() {
    let x : String; //~ERROR Use a &'static string instead
    
    x = "Hallo".to_owned();
    
    format!("{}", x);
}

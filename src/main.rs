extern crate kodak;
use kodak::*;

fn main() {
    let img1 = Image::load_png(String::from("test.png")).unwrap()
        .crop(Region::from_top_left(Dim::square(70))).unwrap()
        .overlay(Image::blank_with_colour(Dim::square(50), Colour::WHITE), Loc {x: 60, y: 60});
    img1.save_png(String::from("out.png"));
}

# kodak

Kodak is a Rust crate meant for manipulating images. It makes use of composition APIs, commonly seen in functional languages.

Some typical Kodak code might look like this:

```rs
extern crate kodak;
use kodak::*;

// Add a white border around an image.
let border_width = 15;

let img = Image::load_png(String::from("assets/olle_ma.png"))
	.unwrap();
let bordered = Image::blank_with_colour(
	img.get_dimensions().expand(2 * border_width),
	Colour::WHITE)
	.overlay(img, Loc { x: border_width, y: border_width });
bordered.save_png(String::from("border_img.png"));

```

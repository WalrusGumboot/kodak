#![crate_name = "kodak"]
#![deny(
    missing_docs,
    missing_copy_implementations,
    missing_debug_implementations,
    unsafe_code,
    unstable_features,
    trivial_casts,
    trivial_numeric_casts
)]

//! Kodak is a crate for image creation and manipulation.
//! It aims to be easy to use, fast and well-documented.
//!
//! Kodak makes use of chaining API methods, like seen in functional programming languages.
//! A typical piece of Kodak code might look like this:
//!
//! ```
//! // Add a white border around an image.
//! extern crate kodak;
//! use kodak::*;
//!
//! let border_width = 16;
//!
//! let src_img = Image::load_png("assets/olle_voader.png").unwrap();
//! let new_img = Image::blank(src_img.get_dimensions().expand(2 * border_width))
//!     .fill(Colour::WHITE)
//!     .overlay(src_img, Loc { x: border_width, y: border_width });
//! new_img.save_png("assets/olle_koader.png");
//! ```

extern crate png;
use std::ops::Add;

/// This struct is used to indicate locations on an image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Loc {
    /// The x-coordinate of the location.
    pub x: u32,
    /// The y-coordinate of the location.
    pub y: u32,
}

impl Loc {
    /// Returns the location from a one-dimensional index given dimensions.
    pub fn from_index(idx: usize, dimension: Dim) -> Self {
        // TODO: rewrite this so that images larger than 2^16 by 2^16 work too
        let i: u32 = idx
            .try_into()
            .expect("An index too large to fit into a u32 was encountered.");
        Loc {
            x: i % dimension.w,
            y: i / dimension.w,
        }
    }

    /// Returns the one-dimensional index from a location given dimensions.
    pub fn as_index(&self, dimension: Dim) -> usize {
        (self.x + self.y * dimension.w).try_into().unwrap()
    }

    /// Checks if a location falls inside of a region.
    ///
    /// Note that this function is left-inclusive, but right-exclusive.
    pub fn inside_region(&self, region: Region) -> bool {
        let x = self.x;
        let y = self.y;

        let cx = region.l.x;
        let cy = region.l.y;
        let w = region.d.w;
        let h = region.d.h;

        x >= cx && x < cx + w && y >= cy && y < cy + h
    }
}

impl Add<Dim> for Loc {
    type Output = Self;
    fn add(self, rhs: Dim) -> Self::Output {
        Loc {
            x: self.x + rhs.w,
            y: self.y + rhs.h,
        }
    }
}

impl Add for Loc {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Loc {
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}

/// This struct is used to indicate dimensions of an image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Dim {
    /// The width aspect of this Dimension.
    pub w: u32,
    /// The height aspect of this Dimension.
    pub h: u32,
}

impl Dim {
    /// Creates a square dimension.
    pub fn square(side: u32) -> Self { Dim { w: side, h: side } }

    /// Expands a dimension with a given amount.
    pub fn expand(self, amount: u32) -> Self {
        Dim { w: self.w + amount, h: self.h + amount }
    }
}

/// This struct is used to indicate a region, specified by a top-left Loc and a Dim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    /// The top left corner of the Region.
    pub l: Loc,
    /// The dimensions of the Region.
    pub d: Dim,
}

impl Region {
    /// Makes a region which starts in the top left corner.
    pub fn from_top_left( dim: Dim ) -> Self { Region { l: Loc{ x: 0, y: 0 } , d: dim } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// A struct to represent colours
///
/// Note that it is assumed that all colours are three-channel, 8 bit per pixel and in sRGB colour space.
/// It is outside of the scope of this crate to support colour representations that differ from this.
pub struct Colour {
    /// The red channel of the colour.
    pub r: u8,
    /// The green channel of the colour.
    pub g: u8,
    /// The blue channel of the colour.
    pub b: u8,
}

impl Colour {
    /// The colour black.
    pub const BLACK: Colour = Colour { r: 0, g: 0, b: 0 };
    /// The colour white.
    pub const WHITE: Colour = Colour {
        r: 255,
        g: 255,
        b: 255,
    };

    /// Creates a Colour from a vector of three `u8`'s.
    pub fn from_vec(v: Vec<u8>) -> Self {
        assert_eq!(v.len(), 3, "Three u8's should be passed to Colour::from_vec().");
        Colour { r: v[0], g: v[1], b: v[2] }
    }

    /// Creates a `Vec<u8>` from a Colour.
    pub fn to_vec(&self) -> Vec<u8> {
        vec![self.r, self.g, self.b]
    }
}

/// The Image struct is at the heart of Kodak. You'll be using functions on this 95% of the time.
///
/// Note that the maximum image size is 2^32 - 1 by 2^32 - 1 pixels. This limit was chosen because it is also the maximum of the PNG format.
#[derive(Debug, Clone)]
pub struct Image {
    /// The width of the image in pixels.
    width: u32,
    /// The height of the image in pixels.
    height: u32,
    /// A vector containing all pixels one-dimensionally.
    pixels: Vec<Colour>,
}

// The following impl block defines constructing functions for Images.
impl Image {
    /// Creates a new blank image.
    ///
    /// The default colour used is black.
    ///
    /// # Arguments
    ///
    /// * `dimension` - the dimensions of the image to be created
    ///
    /// # Returns
    ///
    /// The newly created image.
    pub fn blank(dimension: Dim) -> Self {
        let width = dimension.w;
        let height = dimension.h;

        Image {
            width,
            height,
            pixels: vec![Colour::BLACK; (width * height).try_into().unwrap()],
        }
    }

    /// Creates a new blank image with the specified colour.
    ///
    /// This function is more efficient than `Image::blank().fill()`.
    pub fn blank_with_colour(dimension: Dim, colour: Colour) -> Self {
        let width = dimension.w;
        let height = dimension.h;

        Image {
            width,
            height,
            pixels: vec![colour; (width * height).try_into().unwrap()],
        }
    }

    /// Loads a PNG image as an Image struct.
    ///
    /// This returns an `Err` if the PNG could not be decoded properly.
    ///
    /// # Panics
    ///
    /// * if the specified file could not be opened,
    /// * if the PNG file could not be read into a buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// let img = Image::load_png("assets/olle_ma.png").unwrap();
    /// ```
    pub fn load_png(file_name: String) -> Result<Self, png::DecodingError> {
        use std::fs::File;

        let decoder = png::Decoder::new(File::open(file_name).unwrap());
        let mut reader = decoder.read_info()?;
        let mut buf = vec![0u8; reader.output_buffer_size()];
        reader.next_frame(&mut buf).unwrap();
        let info = reader.info();

        let mut pixels_iterator = buf.iter().peekable();
        let mut pixels: Vec<Colour> = Vec::new();

        while pixels_iterator.peek().is_some() {
            pixels.push(Colour::from_vec(
                pixels_iterator.by_ref().cloned().take(3).collect()
            ));
        }

        Ok(Image { width: info.width, height: info.height, pixels })
    }

    /// Saves an Image as a PNG file.
    pub fn save_png(&self, file_name: String) {
        use std::fs::File;
        use std::io::BufWriter;

        let mut encoder = png::Encoder::new(
            BufWriter::new(File::create(file_name).unwrap()),
            self.width, self.height
        );

        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header().unwrap();

        let pixel_data: Vec<u8> = self.pixels.iter().flat_map(|c| c.to_vec()).collect();
        writer.write_image_data(&pixel_data[..]).unwrap();
    }
}

// The following impl block defines functions that give information about Images.
impl Image {
    /// Returns the dimensions of the image.
    pub fn get_dimensions(&self) -> Dim {
        Dim {
            w: self.width,
            h: self.height,
        }
    }
    /// Returns the entire image as a region.
    pub fn as_region(&self) -> Region {
        Region {
            l: Loc { x: 0, y: 0 },
            d: self.get_dimensions(),
        }
    }
    /// Tries to look up the colour of a specific pixel; returns an Err<&str>
    /// if the location is out of bounds and an Ok<Colour> if not.
    pub fn get_pixel(&self, loc: Loc) -> Result<Colour, &'static str> {
        if !loc.inside_region(self.as_region()) {
            return Err("The specified location falls outside of the image.");
        }

        Ok(self.pixels[loc.as_index(self.get_dimensions())])
    }
}

// The following impl block defines modifying functions for Images.
impl Image {
    /// Fill the entire image with a given colour.
    ///
    /// # Arguments
    ///
    /// * `colour`: the colour to fill with.
    ///
    /// # Examples
    ///
    /// ```
    /// let img = Image::blank(30, 20).fill(Colour::WHITE);
    /// assert_eq!(img.pixels[0], Colour::WHITE);
    /// ```
    pub fn fill(self, colour: Colour) -> Image {
        let new_pixels = vec![colour; self.pixels.len()];
        Image {
            pixels: new_pixels,
            ..self
        }
    }

    /// Fills a region.
    pub fn fill_region(self, region: Region, colour: Colour) -> Image {
        let new_pixels = self
            .pixels
            .clone()
            .iter_mut()
            .enumerate()
            .map(|c| {
                if Loc::from_index(c.0, self.get_dimensions()).inside_region(region) {
                    colour
                } else {
                    *c.1 // pass through
                }
            })
            .collect::<Vec<_>>();
        Image {
            pixels: new_pixels,
            ..self
        }
    }

    /// Crop a region out of the image and return it. This method is mainly used internally and panics; the safer `crop()` should be used instead.
    ///
    /// The corner from which to crop is assumed to be the top left corner.
    ///
    /// # Panics
    ///
    /// * if the corner from which to crop is outside of the image,
    /// * if the cropped image would reach outside of the image (by specifying new dimensions which are too large).
    pub fn crop_unclamped(self, region: Region) -> Image {
        let new_width = region.d.w;
        let new_height = region.d.h;

        assert!(
            region.l.inside_region(self.as_region()),
            "The corner from which to crop is outside of the image."
        );
        assert!((region.l + region.d).inside_region(self.as_region()));

        Image {
            width: new_width,
            height: new_height,
            pixels: self
                .pixels
                .iter()
                .enumerate()
                .filter(|x| Loc::from_index(x.0, self.get_dimensions()).inside_region(region))
                .map(|x| *x.1)
                .collect(),
        }
    }

    /// Crop a region out of the image and return it. This method (unlike `crop_unclamped()`) will adjust the
    /// size of the region if it is too big. It will return an `Err<&str>` if the corner to begin with falls outside of the image.
    ///
    /// # Arguments
    ///
    /// * `region` - the region to crop out (_inclusive_).
    ///
    /// # Examples
    ///
    /// ```
    /// let img = Image::blank( Dim { w: 20, h: 30 } )
    ///     .crop( Region { l: Loc { x: 10, y: 10 }, d: Dim { w: 10, h: 10 } } )
    ///     .unwrap();
    /// assert_eq!(img.width, 10);
    /// ```
    pub fn crop(self, region: Region) -> Result<Image, &'static str> {
        if !region.l.inside_region(self.as_region()) {
            return Err("The corner from which to crop falls outside of the image.");
        }

        if !(region.l + region.d).inside_region(self.as_region()) {
            // We clamp the area to be cropped.
            let new_width = self.width - region.d.w;
            let new_height = self.height - region.d.h;

            return Ok(self.crop_unclamped(Region {
                d: Dim {
                    w: new_width,
                    h: new_height,
                },
                ..region
            }));
        }

        Ok(self.crop_unclamped(region))
    }

    /// Overlays a given Image on top of this Image, at the specified location.
    /// This function will not care if the other image is too big to fit on top of the original.
    pub fn overlay(self, other: Image, offset: Loc) -> Self {
        let crop_dims = Dim { w: self.width - offset.x, h: self.height - offset.y };
        let cropped = other.crop( Region::from_top_left(crop_dims)).unwrap();
        println!("The cropped image is {} by {}", cropped.width, cropped.height);

        let mut working_copy = self.pixels.clone();
        for p in cropped.pixels.iter().enumerate() {
            let loc_on_other = Loc::from_index(p.0, cropped.get_dimensions());
            let loc_on_original = loc_on_other + offset;
            working_copy[loc_on_original.as_index(self.get_dimensions())] = *p.1;
        }

        Image { pixels: working_copy, ..self }
    }
}

#[cfg(test)]
mod kodak_tests {
    use super::*;

    #[test]
    fn loc_in_region() {
        let location1 = Loc { x: 10, y: 10 };
        let location2 = Loc { x: 10, y: 11 };
        assert!(location1.inside_region(Region {
            l: Loc { x: 0, y: 0 },
            d: Dim { w: 20, h: 20 }
        }));

        assert!(location2.inside_region(Region {
            l: Loc { x: 0, y: 0 },
            d: Dim { w: 20, h: 20 }
        }));
    }

    #[test]
    fn fill_region() {
        let img = Image::blank(Dim { w: 20, h: 10 }).fill_region(
            Region {
                l: Loc { x: 10, y: 0 },
                d: Dim { w: 10, h: 10 },
            },
            Colour::WHITE,
        );

        assert_eq!(img.get_pixel(Loc { x: 5, y: 5 }).unwrap(), Colour::BLACK);
        assert_eq!(img.get_pixel(Loc { x: 15, y: 5 }).unwrap(), Colour::WHITE);
    }

    #[test]
    fn crop() {
        let img = Image::load_png(String::from("test.png")).unwrap();
        assert_eq!(img.crop(Region::from_top_left(Dim::square(100))).unwrap().width, 100);
    }

    #[test]
    fn overlay_non_out_of_bounds() {
        let original = Image::blank_with_colour(Dim::square(10), Colour::WHITE);
        let overlay = Image::blank(Dim::square(5));

        let result = original.overlay(overlay, Loc { x: 0, y: 0 });

        assert_eq!(result.get_pixel(Loc {x: 0, y: 0}).unwrap(), Colour::BLACK);
        assert_eq!(result.get_pixel(Loc {x: 4, y: 0}).unwrap(), Colour::BLACK);
        assert_eq!(result.get_pixel(Loc {x: 0, y: 4}).unwrap(), Colour::BLACK);
        assert_eq!(result.get_pixel(Loc {x: 4, y: 4}).unwrap(), Colour::BLACK);
        assert_eq!(result.get_pixel(Loc {x: 0, y: 5}).unwrap(), Colour::WHITE);
        assert_eq!(result.get_pixel(Loc {x: 5, y: 0}).unwrap(), Colour::WHITE);
    }
}

//! A picture as Demysto holds one: fitted for every Run, and the original for
//! the Run that asks again.
//!
//! The clipboard hands over pixels rather than a file — raw RGBA8 with a width
//! and a height — so this encodes as well as fits, and never decodes anything.
//! ADR-0017 fixes both halves of the shape: every Run sends the fitted picture,
//! there is no setting governing it, and the original is held so that the offer
//! to ask again at full resolution is real rather than a promise to re-capture
//! something the clipboard no longer holds.

use std::sync::Arc;

use base64::Engine;
use image::{imageops::FilterType, ExtendedColorType, ImageEncoder, RgbaImage};

/// The longest side a picture is fitted to before a Run sends it, in pixels.
///
/// A number rather than a principle, and recorded as one in `docs/spec/0002`:
/// if photographs turn out to matter more than screenshots, this and the format
/// beside it are the same one line.
pub(crate) const CEILING: u32 = 1568;

/// What a data URL carrying a picture opens with. PNG rather than JPEG because
/// the case that matters most is text on a screenshot, and that is the case
/// JPEG's artefacts land on.
const DATA_URL: &str = "data:image/png;base64,";

/// What a picture is fitted with. Lanczos over anything cheaper for the reason
/// PNG is chosen over JPEG: eight-point text in a screenshot is what a Model is
/// being asked to read, and a nearest-neighbour reduction of it is not text any
/// more.
const FILTER: FilterType = FilterType::Lanczos3;

/// One encoding of a picture: the bytes that travel, and how large the picture
/// they carry is.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Png {
    bytes: Vec<u8>,
    width: u32,
    height: u32,
}

/// The picture an image Selection is about, held both ways.
///
/// A picture already within the ceiling is fitted and original at once, and is
/// stored once: the two handles are the same allocation, and everything that
/// counts what is held counts it once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picture {
    fitted: Arc<Png>,
    original: Arc<Png>,
}

impl Png {
    /// What a request carries: the picture as a data URL, which is the shape
    /// the contract's `image_url` part takes.
    pub(crate) fn url(&self) -> String {
        let mut url = String::with_capacity(DATA_URL.len() + self.bytes.len().div_ceil(3) * 4);
        url.push_str(DATA_URL);
        base64::engine::general_purpose::STANDARD.encode_string(&self.bytes, &mut url);

        url
    }

    /// What this encoding weighs, which is what the button offering to send it
    /// says on it.
    pub(crate) fn weight(&self) -> u64 {
        self.bytes.len() as u64
    }
}

impl Picture {
    /// The picture the clipboard handed over: raw RGBA8, with the size to read
    /// it by.
    ///
    /// Fails rather than guesses where the buffer is not the size the width and
    /// the height claim: a clipboard that says one thing and holds another is
    /// not a picture Demysto can send, and cropping it to fit would be sending
    /// something nobody copied.
    /// `None` where the buffer is not the size the width and the height claim,
    /// and where what arrived could not be encoded at all. Neither is a
    /// sentence: what the user is told about a picture Demysto cannot use is
    /// one message in the catalogue, said by `capture`.
    pub(crate) fn taken(width: u32, height: u32, rgba: &[u8]) -> Option<Self> {
        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|pixels| pixels.checked_mul(4))?;

        if width == 0 || height == 0 || expected != rgba.len() {
            return None;
        }

        let raw = RgbaImage::from_raw(width, height, rgba.to_vec())
            .expect("a buffer the size the dimensions claim is an image");

        let original = Arc::new(encoded(&raw)?);

        let fitted = match fitting(width, height) {
            // Already within the ceiling, so there is nothing to fit and
            // nothing to store twice.
            None => Arc::clone(&original),
            Some((width, height)) => Arc::new(encoded(&image::imageops::resize(
                &raw, width, height, FILTER,
            ))?),
        };

        Some(Self { fitted, original })
    }

    /// What a Run sends, and what a window shows: the picture fitted to the
    /// ceiling.
    ///
    /// One method for both, because it is one picture: showing the original
    /// would cross megabytes to draw something a few hundred pixels wide, and
    /// sending it is what the button is for.
    pub(crate) fn fitted(&self) -> &Arc<Png> {
        &self.fitted
    }

    /// What a Run sends once somebody has asked again at the original
    /// resolution, and every Turn after that one.
    pub(crate) fn original(&self) -> &Arc<Png> {
        &self.original
    }

    /// Whether fitting took anything off this picture at all.
    ///
    /// What the offer to ask again turns on: where the two are one allocation
    /// there is nothing the original would say that the fitted one did not,
    /// and a button that sends a byte-identical request is a button that
    /// charges for nothing.
    pub(crate) fn was_fitted(&self) -> bool {
        !Arc::ptr_eq(&self.fitted, &self.original)
    }

    /// How large the picture is, which is what the windows say where there is
    /// no text to quote.
    ///
    /// No catalogue message, unusually, and for a reason: digits and a
    /// multiplication sign say the same thing in every language Demysto speaks.
    pub(crate) fn dimensions(&self) -> String {
        format!("{} × {}", self.original.width, self.original.height)
    }

    /// What asking again at the original resolution would send, so that the
    /// price is written on the button rather than in a warning over it.
    pub(crate) fn original_weight(&self) -> u64 {
        self.original.weight()
    }

    /// What holding this picture costs, counted once where the two handles are
    /// one allocation.
    pub(crate) fn weight_held(&self) -> u64 {
        match self.was_fitted() {
            true => self.fitted.weight() + self.original.weight(),
            false => self.original.weight(),
        }
    }
}

/// What the windows are told about a picture: how large it is, and nothing
/// else.
///
/// The bytes are asked for rather than carried, for the reason the whole of a
/// text Selection is: this is on the Capture the Palette is handed and on the
/// Conversation that crosses the bridge every time a Turn begins or ends, and a
/// data URL on either would cross megabytes to say what a thumbnail needs once.
///
/// The size crosses already written out rather than as two numbers, because
/// written out is the only form anything wants it in — and because a window
/// handed the numbers hands them to Fluent, which formats a number the way a
/// quantity is formatted and makes 2000 pixels read as "2,000".
impl serde::Serialize for Picture {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;

        let mut picture = serializer.serialize_struct("Picture", 1)?;
        picture.serialize_field("dimensions", &self.dimensions())?;
        picture.end()
    }
}

/// The size a picture over the ceiling is fitted to, or `None` for one already
/// within it — nothing is ever enlarged.
fn fitting(width: u32, height: u32) -> Option<(u32, u32)> {
    let longest = width.max(height);

    if longest <= CEILING {
        return None;
    }

    let scale = f64::from(CEILING) / f64::from(longest);

    Some((scaled(width, scale), scaled(height, scale)))
}

/// One side, scaled — and never to nothing: a panorama fitted by its width can
/// arrive at less than half a pixel of height, and a picture no pixels high is
/// not one anything can encode.
fn scaled(side: u32, by: f64) -> u32 {
    ((f64::from(side) * by).round() as u32).max(1)
}

fn encoded(raw: &RgbaImage) -> Option<Png> {
    let mut bytes = Vec::new();

    image::codecs::png::PngEncoder::new(&mut bytes)
        .write_image(raw, raw.width(), raw.height(), ExtendedColorType::Rgba8)
        .ok()?;

    Some(Png {
        bytes,
        width: raw.width(),
        height: raw.height(),
    })
}

#[cfg(test)]
mod tests {
    //! Fitting, tested here rather than at the facade: what a Capture produces
    //! is asserted through the seam alongside everything else, and these are
    //! about the arithmetic under it — which no Run reaches in enough shapes to
    //! cover.

    use super::*;

    /// A picture of that size, in the shape the clipboard hands one over.
    fn taken(width: u32, height: u32) -> Picture {
        let rgba = vec![0x7f; (width as usize) * (height as usize) * 4];

        Picture::taken(width, height, &rgba).expect("that is a picture")
    }

    /// What a Run would send, by size.
    fn sent(picture: &Picture) -> (u32, u32) {
        let png = picture.fitted();

        (png.width, png.height)
    }

    #[test]
    fn a_picture_over_the_ceiling_is_fitted_to_it() {
        assert_eq!(sent(&taken(3000, 2000)), (1568, 1045));
    }

    #[test]
    fn fitting_keeps_the_shape_of_the_picture() {
        let (width, height) = sent(&taken(3000, 2000));

        let before = 3000.0 / 2000.0;
        let after = f64::from(width) / f64::from(height);

        assert!((before - after).abs() < 0.01, "{before} became {after}");
    }

    #[test]
    fn a_picture_under_the_ceiling_is_sent_as_it_is() {
        assert_eq!(sent(&taken(800, 600)), (800, 600));
    }

    #[test]
    fn a_picture_exactly_on_the_ceiling_is_sent_as_it_is() {
        assert_eq!(sent(&taken(CEILING, 400)), (CEILING, 400));
    }

    #[test]
    fn a_picture_within_the_ceiling_is_stored_once() {
        let picture = taken(800, 600);

        assert!(
            Arc::ptr_eq(picture.fitted(), picture.original()),
            "the fitted picture and the original should be the same allocation"
        );
        assert!(!picture.was_fitted(), "nothing was taken off it");
        assert_eq!(picture.weight_held(), picture.original_weight());
    }

    #[test]
    fn a_very_wide_picture_is_fitted_by_its_longest_side() {
        // Fitted by the width, and the height is what is left of it — which is
        // small, and must not be nothing.
        assert_eq!(sent(&taken(4000, 200)), (1568, 78));
    }

    #[test]
    fn a_picture_far_wider_than_it_is_tall_keeps_a_pixel_of_height() {
        assert_eq!(sent(&taken(4000, 1)), (1568, 1));
    }

    #[test]
    fn a_picture_over_the_ceiling_holds_both_of_itself() {
        let picture = taken(3000, 2000);

        assert!(picture.was_fitted());
        assert_eq!(
            picture.weight_held(),
            picture.fitted().weight() + picture.original_weight()
        );
    }

    #[test]
    fn what_travels_is_a_png_in_a_data_url() {
        let url = taken(8, 8).fitted().url();

        assert!(url.starts_with("data:image/png;base64,"), "{url}");

        let bytes = base64::engine::general_purpose::STANDARD
            .decode(url.trim_start_matches(DATA_URL))
            .expect("a data URL carries base64");

        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n", "that is not a PNG");
    }

    #[test]
    fn the_dimensions_are_the_originals_rather_than_the_fitted_ones() {
        // What the list of Conversations shows is the picture the user copied,
        // not what Demysto made of it to send.
        assert_eq!(taken(3000, 2000).dimensions(), "3000 × 2000");
    }

    #[test]
    fn a_buffer_that_is_not_the_size_it_claims_is_not_a_picture() {
        assert!(Picture::taken(100, 100, &[0; 16]).is_none());
    }

    #[test]
    fn a_picture_of_no_size_at_all_is_not_one() {
        assert!(Picture::taken(0, 0, &[]).is_none());
    }
}

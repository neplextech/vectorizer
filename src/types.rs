use crate::config::{ColorMode, Config, Hierarchical, Preset};
use napi::{bindgen_prelude::Buffer, Either, Result};
use visioncortex::{Color, ColorImage, PathSimplifyMode};

/// Path simplification strategy used while fitting vector paths.
#[napi(js_name = "PathSimplifyMode")]
#[derive(Clone)]
pub enum JsPathSimplifyMode {
  /// Do not simplify paths.
  None,
  /// Simplify paths into polygons.
  Polygon,
  /// Fit paths as splines.
  Spline,
}

/// Vectorization configuration.
#[derive(Clone)]
#[napi(object, js_name = "Config")]
pub struct JsConfig {
  /// True color image or binary image (black and white)
  pub color_mode: ColorMode,
  /// Hierarchial clustering or non-stacked. Only applicable to color images.
  pub hierarchical: Hierarchical,
  /// Discard patches smaller than X pixels in size (cleaner)
  pub filter_speckle: i32,
  /// The number of significant bits to use in an RGB channel (more accurate)
  pub color_precision: i32,
  /// The color difference between gradient layers (less layers)
  pub layer_difference: i32,
  /// Curve fitting mode
  pub mode: JsPathSimplifyMode,
  /// Minimum momentary angle (degree) to be considered a corner (smoother)
  pub corner_threshold: i32,
  /// Perform iterative subdivide smooth until all segments are shorter than this length
  pub length_threshold: f64,
  /// The maximum number of iterations to perform
  pub max_iterations: i32,
  /// Minimum angle displacement (degree) to splice a spline (less accurate)
  pub splice_threshold: i32,
  /// Number of decimal places to use in path string
  pub path_precision: Option<u32>,
  /// Random color search attempts after the reserved key colors are exhausted
  pub unused_color_iterations: Option<u32>,
  /// Transparent boundary fraction required before keying is applied
  pub keying_threshold: Option<f64>,
  /// Maximum cluster bounds treated as a small circle in spline mode
  pub small_circle: Option<i32>,
}

/// Dimensions for raw RGBA pixel buffers.
#[derive(Clone)]
#[napi(object)]
pub struct RawDataConfig {
  /// Raw image width in pixels.
  pub width: i32,
  /// Raw image height in pixels.
  pub height: i32,
}

/// Decoded RGBA image data.
#[napi(object, js_name = "ImageData")]
pub struct JsImageData {
  /// Image width in pixels.
  pub width: u32,
  /// Image height in pixels.
  pub height: u32,
  /// RGBA pixel data with four bytes per pixel.
  pub pixels: Buffer,
}

/// RGBA color value.
#[derive(Clone, PartialEq, Eq)]
#[napi(object, js_name = "Color")]
pub struct JsColor {
  /// Red channel from 0 to 255.
  pub r: u32,
  /// Green channel from 0 to 255.
  pub g: u32,
  /// Blue channel from 0 to 255.
  pub b: u32,
  /// Optional alpha channel from 0 to 255.
  pub a: Option<u32>,
}

/// Internal helper options.
#[derive(Clone)]
#[napi(object, js_name = "InternalOptions")]
pub struct JsInternalOptions {
  /// Random color search attempts after reserved key colors are exhausted.
  pub unused_color_iterations: Option<u32>,
}

pub(crate) fn resolve_config(config: Option<Either<JsConfig, Preset>>) -> Config {
  match config {
    Some(Either::A(config)) => {
      let defaults = Config::default();
      Config {
        color_mode: match config.color_mode {
          ColorMode::Color => ColorMode::Color,
          ColorMode::Binary => ColorMode::Binary,
        },
        hierarchical: match config.hierarchical {
          Hierarchical::Stacked => Hierarchical::Stacked,
          Hierarchical::Cutout => Hierarchical::Cutout,
        },
        filter_speckle: config.filter_speckle as usize,
        color_precision: config.color_precision,
        layer_difference: config.layer_difference,
        mode: match config.mode {
          JsPathSimplifyMode::None => PathSimplifyMode::None,
          JsPathSimplifyMode::Polygon => PathSimplifyMode::Polygon,
          JsPathSimplifyMode::Spline => PathSimplifyMode::Spline,
        },
        corner_threshold: config.corner_threshold,
        length_threshold: config.length_threshold,
        max_iterations: config.max_iterations as usize,
        splice_threshold: config.splice_threshold,
        path_precision: config.path_precision,
        unused_color_iterations: config
          .unused_color_iterations
          .map(|value| value as usize)
          .unwrap_or(defaults.unused_color_iterations),
        keying_threshold: config
          .keying_threshold
          .map(|value| value as f32)
          .unwrap_or(defaults.keying_threshold),
        small_circle: config.small_circle.unwrap_or(defaults.small_circle),
      }
    }
    Some(Either::B(preset)) => Config::from_preset(preset),
    None => Config::default(),
  }
}

pub(crate) fn color_image_to_js(img: ColorImage) -> JsImageData {
  JsImageData {
    width: img.width as u32,
    height: img.height as u32,
    pixels: Buffer::from(img.pixels),
  }
}

pub(crate) fn color_image_from_js(img: JsImageData) -> Result<ColorImage> {
  let width = img.width as usize;
  let height = img.height as usize;
  let pixels = img.pixels.as_ref().to_vec();
  let expected_len = width
    .checked_mul(height)
    .and_then(|size| size.checked_mul(4))
    .ok_or_else(|| {
      napi::Error::new(
        napi::Status::InvalidArg,
        "Image dimensions are too large".to_string(),
      )
    })?;

  if pixels.len() != expected_len {
    return Err(napi::Error::new(
      napi::Status::InvalidArg,
      format!(
        "Image pixel length {} does not match width * height * 4 {}",
        pixels.len(),
        expected_len
      ),
    ));
  }

  Ok(ColorImage {
    pixels,
    width,
    height,
  })
}

pub(crate) fn color_from_js(color: JsColor) -> Result<Color> {
  Ok(Color::new_rgba(
    channel_from_js(color.r, "r")?,
    channel_from_js(color.g, "g")?,
    channel_from_js(color.b, "b")?,
    channel_from_js(color.a.unwrap_or(255), "a")?,
  ))
}

pub(crate) fn color_to_js(color: Color) -> JsColor {
  JsColor {
    r: color.r as u32,
    g: color.g as u32,
    b: color.b as u32,
    a: Some(color.a as u32),
  }
}

fn channel_from_js(value: u32, name: &str) -> Result<u8> {
  u8::try_from(value).map_err(|_| {
    napi::Error::new(
      napi::Status::InvalidArg,
      format!("Color channel {} must be between 0 and 255", name),
    )
  })
}

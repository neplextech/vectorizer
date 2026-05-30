#![deny(clippy::all)]

use config::{ColorMode, Config, Hierarchical, Preset};
use converter::{
  convert_image_to_svg, convert_image_to_svg_chunks, js_color_exists_in_image,
  js_find_unused_color_in_image, js_read_image,
};
use napi::{
  bindgen_prelude::{AsyncTask, Buffer, Unknown},
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
  Either, Result, Status, Task,
};
use std::panic;
use svg::SvgFile;
use visioncortex::{Color, ColorImage, PathSimplifyMode};

type JsChunkCallback =
  ThreadsafeFunction<(String, f64), Unknown<'static>, (String, f64), Status, false>;

#[macro_use]
extern crate napi_derive;

pub mod config;
pub mod converter;
pub mod svg;

#[napi(js_name = "PathSimplifyMode")]
#[derive(Clone)]
pub enum JsPathSimplifyMode {
  None,
  Polygon,
  Spline,
}

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

#[derive(Clone)]
#[napi(object)]
pub struct RawDataConfig {
  pub width: i32,
  pub height: i32,
}

#[napi(object, js_name = "ImageData")]
pub struct JsImageData {
  pub width: u32,
  pub height: u32,
  pub pixels: Buffer,
}

#[derive(Clone, PartialEq, Eq)]
#[napi(object, js_name = "Color")]
pub struct JsColor {
  pub r: u32,
  pub g: u32,
  pub b: u32,
  pub a: Option<u32>,
}

#[derive(Clone)]
#[napi(object, js_name = "InternalOptions")]
pub struct JsInternalOptions {
  pub unused_color_iterations: Option<u32>,
}

pub struct VectorizeTask {
  data: Buffer,
  config: Option<Either<JsConfig, Preset>>,
  args: Option<RawDataConfig>,
}

pub struct VectorizeCallbackTask {
  data: Buffer,
  config: Option<Either<JsConfig, Preset>>,
  args: Option<RawDataConfig>,
  callback: JsChunkCallback,
}

#[napi]
impl Task for VectorizeTask {
  type Output = String;
  type JsValue = String;

  fn compute(&mut self) -> Result<Self::Output> {
    let res = vectorize_inner(self.data.as_ref(), self.config.clone(), self.args.clone());
    res
  }

  fn resolve(&mut self, _env: napi::Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output)
  }
}

#[napi]
impl Task for VectorizeCallbackTask {
  type Output = ();
  type JsValue = ();

  fn compute(&mut self) -> Result<Self::Output> {
    vectorize_inner_chunks(
      self.data.as_ref(),
      self.config.clone(),
      self.args.clone(),
      &self.callback,
    )
  }

  fn resolve(&mut self, _env: napi::Env, _output: Self::Output) -> Result<Self::JsValue> {
    Ok(())
  }
}

#[napi(catch_unwind)]
pub fn vectorize(
  source: Buffer,
  config: Option<Either<JsConfig, Preset>>,
) -> AsyncTask<VectorizeTask> {
  AsyncTask::new(VectorizeTask {
    data: source,
    config,
    args: None,
  })
}

#[napi(catch_unwind)]
pub fn vectorize_raw(
  source: Buffer,
  args: RawDataConfig,
  config: Option<Either<JsConfig, Preset>>,
) -> AsyncTask<VectorizeTask> {
  AsyncTask::new(VectorizeTask {
    data: source,
    config,
    args: Some(args),
  })
}

#[napi(catch_unwind)]
pub fn vectorize_sync(source: Buffer, config: Option<Either<JsConfig, Preset>>) -> Result<String> {
  vectorize_inner(source.as_ref(), config, None)
}

#[napi(catch_unwind)]
pub fn vectorize_raw_sync(
  source: Buffer,
  args: RawDataConfig,
  config: Option<Either<JsConfig, Preset>>,
) -> Result<String> {
  vectorize_inner(source.as_ref(), config, Some(args))
}

#[napi(
  catch_unwind,
  js_name = "vectorizeToCallback",
  ts_args_type = "source: Buffer, config: Config | Preset | undefined | null, callback: ([string, number]) => void"
)]
pub fn js_vectorize_to_callback(
  source: Buffer,
  config: Option<Either<JsConfig, Preset>>,
  callback: JsChunkCallback,
) -> AsyncTask<VectorizeCallbackTask> {
  AsyncTask::new(VectorizeCallbackTask {
    data: source,
    config,
    args: None,
    callback,
  })
}

#[napi(
  catch_unwind,
  js_name = "vectorizeRawToCallback",
  ts_args_type = "source: Buffer, args: RawDataConfig, config: Config | Preset | undefined | null, callback: ([string, number]) => void"
)]
pub fn js_vectorize_raw_to_callback(
  source: Buffer,
  args: RawDataConfig,
  config: Option<Either<JsConfig, Preset>>,
  callback: JsChunkCallback,
) -> AsyncTask<VectorizeCallbackTask> {
  AsyncTask::new(VectorizeCallbackTask {
    data: source,
    config,
    args: Some(args),
    callback,
  })
}

#[napi(catch_unwind, js_name = "readImage")]
pub fn js_read_image_from_js(source: Buffer, args: Option<RawDataConfig>) -> Result<JsImageData> {
  let img = js_read_image(source.as_ref(), args).map_err(to_napi_error)?;

  Ok(color_image_to_js(img))
}

#[napi(catch_unwind, js_name = "colorExistsInImage")]
pub fn js_color_exists_in_image_from_js(img: JsImageData, color: JsColor) -> Result<bool> {
  let img = color_image_from_js(img)?;
  let color = color_from_js(color)?;

  Ok(js_color_exists_in_image(&img, color))
}

#[napi(catch_unwind, js_name = "findUnusedColorInImage")]
pub fn js_find_unused_color_in_image_from_js(
  img: JsImageData,
  options: Option<JsInternalOptions>,
) -> Result<JsColor> {
  let img = color_image_from_js(img)?;
  let unused_color_iterations = options
    .and_then(|options| options.unused_color_iterations)
    .unwrap_or(6) as usize;
  let color =
    js_find_unused_color_in_image(&img, unused_color_iterations).map_err(to_napi_error)?;

  Ok(color_to_js(color))
}

#[napi(js_name = "SvgFile")]
pub struct JsSvgFile {
  inner: SvgFile,
}

#[napi]
impl JsSvgFile {
  #[napi(constructor)]
  pub fn new(width: u32, height: u32, path_precision: Option<u32>) -> Self {
    Self {
      inner: SvgFile::new(width as usize, height as usize, path_precision),
    }
  }

  #[napi(getter)]
  pub fn width(&self) -> u32 {
    self.inner.width as u32
  }

  #[napi(getter)]
  pub fn height(&self) -> u32 {
    self.inner.height as u32
  }

  #[napi(getter)]
  pub fn path_precision(&self) -> Option<u32> {
    self.inner.path_precision
  }

  #[napi(getter)]
  pub fn path_count(&self) -> u32 {
    self.inner.paths.len() as u32
  }

  #[napi(js_name = "toString")]
  pub fn js_to_string(&self) -> Result<String> {
    self.inner.to_string().map_err(|e| {
      napi::Error::new(
        napi::Status::GenericFailure,
        format!("Error: {:?}", e).as_str(),
      )
    })
  }
}

fn vectorize_inner(
  source: &[u8],
  config: Option<Either<JsConfig, Preset>>,
  raw_args: Option<RawDataConfig>,
) -> Result<String> {
  panic::set_hook(Box::new(|_info| {}));

  let result =
    panic::catch_unwind(|| convert_image_to_svg(source, resolve_config(config), raw_args));

  let result = match result {
    Ok(res) => res,
    Err(_) => Err(napi::Error::new(
      napi::Status::GenericFailure,
      "Unknown error occurred",
    ))?,
  };

  let svg = result.map_err(|e| {
    napi::Error::new(
      napi::Status::GenericFailure,
      format!("Error: {:?}", e).as_str(),
    )
  })?;

  Ok(svg)
}

fn vectorize_inner_chunks(
  source: &[u8],
  config: Option<Either<JsConfig, Preset>>,
  raw_args: Option<RawDataConfig>,
  callback: &JsChunkCallback,
) -> Result<()> {
  panic::set_hook(Box::new(|_info| {}));

  let result = panic::catch_unwind(|| {
    convert_image_to_svg_chunks(
      source,
      resolve_config(config),
      raw_args,
      |chunk, progress| {
        let status = callback.call((chunk, progress), ThreadsafeFunctionCallMode::Blocking);
        if status == Status::Ok {
          Ok(())
        } else {
          Err(format!("chunk callback failed with status {}", status))
        }
      },
    )
  });

  let result = match result {
    Ok(res) => res,
    Err(_) => Err(napi::Error::new(
      napi::Status::GenericFailure,
      "Unknown error occurred",
    ))?,
  };

  result.map_err(to_napi_error)?;

  Ok(())
}

fn resolve_config(config: Option<Either<JsConfig, Preset>>) -> Config {
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

fn color_image_to_js(img: ColorImage) -> JsImageData {
  JsImageData {
    width: img.width as u32,
    height: img.height as u32,
    pixels: Buffer::from(img.pixels),
  }
}

fn color_image_from_js(img: JsImageData) -> Result<ColorImage> {
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

fn color_from_js(color: JsColor) -> Result<Color> {
  Ok(Color::new_rgba(
    channel_from_js(color.r, "r")?,
    channel_from_js(color.g, "g")?,
    channel_from_js(color.b, "b")?,
    channel_from_js(color.a.unwrap_or(255), "a")?,
  ))
}

fn color_to_js(color: Color) -> JsColor {
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

fn to_napi_error(error: String) -> napi::Error {
  napi::Error::new(napi::Status::GenericFailure, error)
}

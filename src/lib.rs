#![deny(clippy::all)]

use config::{ColorMode, Config, Hierarchical, Preset};
use converter::convert_image_to_svg;
use napi::{
  bindgen_prelude::{AsyncTask, Buffer},
  Either, Result, Task,
};
use std::panic;
use visioncortex::PathSimplifyMode;

#[macro_use]
extern crate napi_derive;

pub mod config;
pub mod converter;
pub mod svg;

#[napi(js_name = "PathSimplifyMode")]
pub enum JsPathSimplifyMode {
  None,
  Polygon,
  Spline,
}

#[derive(Clone)]
#[napi(object, js_name = "Config")]
pub struct JsConfig {
  pub color_mode: ColorMode,
  pub hierarchical: Hierarchical,
  pub filter_speckle: i32,
  pub color_precision: i32,
  pub layer_difference: i32,
  pub mode: JsPathSimplifyMode,
  pub corner_threshold: i32,
  pub length_threshold: f64,
  pub max_iterations: i32,
  pub splice_threshold: i32,
  pub path_precision: Option<u32>,
}

#[derive(Clone)]
#[napi(object)]
pub struct RawDataConfig {
  pub width: i32,
  pub height: i32,
}

pub struct VectorizeTask {
  data: Buffer,
  config: Option<Either<JsConfig, Preset>>,
  args: Option<RawDataConfig>,
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

fn create_config_with_preset(preset: Preset) -> Config {
  Config::from_preset(preset)
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

fn resolve_config(config: Option<Either<JsConfig, Preset>>) -> Config {
  match config {
    Some(Either::A(config)) => Config {
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
    },
    Some(Either::B(preset)) => create_config_with_preset(preset),
    None => Config::default(),
  }
}

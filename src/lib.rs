#![deny(clippy::all)]

use napi::{
  bindgen_prelude::{AsyncTask, Buffer},
  Result, Task,
};
use visioncortex::PathSimplifyMode as VcPathSimplifyMode;
use vtracer;

#[macro_use]
extern crate napi_derive;

#[napi]
pub enum ColorMode {
  Color,
  Binary,
}

#[napi]
pub enum Hierarchical {
  Stacked,
  Cutout,
}

#[napi]
pub enum PathSimplifyMode {
  None,
  Polygon,
  Spline,
}

#[napi]
pub enum Preset {
  Bw,
  Poster,
  Photo,
}

#[derive(Clone)]
#[napi(object)]
pub struct Config {
  pub color_mode: ColorMode,
  pub hierarchical: Hierarchical,
  pub filter_speckle: i32,
  pub color_precision: i32,
  pub layer_difference: i32,
  pub mode: PathSimplifyMode,
  pub corner_threshold: i32,
  pub length_threshold: f64,
  pub max_iterations: i32,
  pub splice_threshold: i32,
  pub path_precision: Option<u32>,
  pub width: i32,
  pub height: i32,
}

pub struct VectorizeTask {
  data: Buffer,
  config: Config,
}

#[napi]
impl Task for VectorizeTask {
  type Output = String;
  type JsValue = String;

  fn compute(&mut self) -> Result<Self::Output> {
    let res = vectorize_inner(self.data.as_ref(), self.config.clone());
    res
  }

  fn resolve(&mut self, _env: napi::Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output)
  }
}

#[napi(catch_unwind)]
pub fn vectorize(source: Buffer, config: Config) -> AsyncTask<VectorizeTask> {
  AsyncTask::new(VectorizeTask {
    data: source,
    config,
  })
}

#[napi(catch_unwind)]
pub fn vectorize_sync(source: Buffer, config: Config) -> Result<String> {
  vectorize_inner(source.as_ref(), config)
}

fn vectorize_inner(source: &[u8], config: Config) -> Result<String> {
  let mut img = vtracer::ColorImage::new_w_h(config.width as usize, config.height as usize);
  img.pixels = source.to_vec();

  let result = vtracer::convert(
    img,
    vtracer::Config {
      color_mode: match config.color_mode {
        ColorMode::Color => vtracer::ColorMode::Color,
        ColorMode::Binary => vtracer::ColorMode::Binary,
      },
      hierarchical: match config.hierarchical {
        Hierarchical::Stacked => vtracer::Hierarchical::Stacked,
        Hierarchical::Cutout => vtracer::Hierarchical::Cutout,
      },
      filter_speckle: config.filter_speckle as usize,
      color_precision: config.color_precision,
      layer_difference: config.layer_difference,
      mode: match config.mode {
        PathSimplifyMode::None => VcPathSimplifyMode::None,
        PathSimplifyMode::Polygon => VcPathSimplifyMode::Polygon,
        PathSimplifyMode::Spline => VcPathSimplifyMode::Spline,
      },
      corner_threshold: config.corner_threshold,
      length_threshold: config.length_threshold,
      max_iterations: config.max_iterations as usize,
      splice_threshold: config.splice_threshold,
      path_precision: config.path_precision,
    },
  );

  let svg = result.map_err(|e| {
    napi::Error::new(
      napi::Status::GenericFailure,
      format!("Error: {:?}", e).as_str(),
    )
  })?;

  Ok(svg.to_string())
}

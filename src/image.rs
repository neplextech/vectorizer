use crate::converter::{js_color_exists_in_image, js_find_unused_color_in_image, js_read_image};
use crate::errors::to_napi_error;
use crate::types::{
  color_from_js, color_image_from_js, color_image_to_js, color_to_js, JsColor, JsImageData,
  JsInternalOptions, RawDataConfig,
};
use napi::{
  bindgen_prelude::{AbortSignal, AsyncTask, Buffer},
  Result, Task,
};
use visioncortex::ColorImage;

pub struct ReadImageTask {
  source: Buffer,
  args: Option<RawDataConfig>,
}

/// Background task used by async image decoding.
#[napi]
impl Task for ReadImageTask {
  type Output = ColorImage;
  type JsValue = JsImageData;

  fn compute(&mut self) -> Result<Self::Output> {
    read_image_inner(self.source.as_ref(), self.args.clone())
  }

  fn resolve(&mut self, _env: napi::Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(color_image_to_js(output))
  }
}

pub struct ColorExistsInImageTask {
  img: JsImageData,
  color: JsColor,
}

/// Background task used by async color existence checks.
#[napi]
impl Task for ColorExistsInImageTask {
  type Output = bool;
  type JsValue = bool;

  fn compute(&mut self) -> Result<Self::Output> {
    color_exists_in_image_inner(
      std::mem::replace(
        &mut self.img,
        JsImageData {
          width: 0,
          height: 0,
          pixels: Buffer::from(Vec::new()),
        },
      ),
      self.color.clone(),
    )
  }

  fn resolve(&mut self, _env: napi::Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output)
  }
}

pub struct FindUnusedColorInImageTask {
  img: JsImageData,
  options: Option<JsInternalOptions>,
}

/// Background task used by async unused color search.
#[napi]
impl Task for FindUnusedColorInImageTask {
  type Output = JsColor;
  type JsValue = JsColor;

  fn compute(&mut self) -> Result<Self::Output> {
    find_unused_color_in_image_inner(
      std::mem::replace(
        &mut self.img,
        JsImageData {
          width: 0,
          height: 0,
          pixels: Buffer::from(Vec::new()),
        },
      ),
      self.options.clone(),
    )
  }

  fn resolve(&mut self, _env: napi::Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output)
  }
}

/// Decode an encoded image buffer or raw RGBA pixel buffer asynchronously.
#[napi(
  catch_unwind,
  js_name = "readImage",
  ts_args_type = "source: Buffer, args?: RawDataConfig | undefined | null, signal?: AbortSignal | undefined | null",
  ts_return_type = "Promise<ImageData>"
)]
pub fn read_image(
  source: Buffer,
  args: Option<RawDataConfig>,
  signal: Option<AbortSignal>,
) -> AsyncTask<ReadImageTask> {
  AsyncTask::with_optional_signal(ReadImageTask { source, args }, signal)
}

/// Decode an encoded image buffer or raw RGBA pixel buffer synchronously.
#[napi(
  catch_unwind,
  js_name = "readImageSync",
  ts_args_type = "source: Buffer, args?: RawDataConfig | undefined | null",
  ts_return_type = "ImageData"
)]
pub fn read_image_sync(source: Buffer, args: Option<RawDataConfig>) -> Result<JsImageData> {
  let img = read_image_inner(source.as_ref(), args)?;

  Ok(color_image_to_js(img))
}

/// Check whether an RGB color exists in decoded image data asynchronously.
#[napi(
  catch_unwind,
  js_name = "colorExistsInImage",
  ts_args_type = "img: ImageData, color: Color, signal?: AbortSignal | undefined | null",
  ts_return_type = "Promise<boolean>"
)]
pub fn color_exists_in_image(
  img: JsImageData,
  color: JsColor,
  signal: Option<AbortSignal>,
) -> AsyncTask<ColorExistsInImageTask> {
  AsyncTask::with_optional_signal(ColorExistsInImageTask { img, color }, signal)
}

/// Check whether an RGB color exists in decoded image data synchronously.
#[napi(
  catch_unwind,
  js_name = "colorExistsInImageSync",
  ts_args_type = "img: ImageData, color: Color",
  ts_return_type = "boolean"
)]
pub fn color_exists_in_image_sync(img: JsImageData, color: JsColor) -> Result<bool> {
  color_exists_in_image_inner(img, color)
}

/// Find an RGB color that does not exist in decoded image data asynchronously.
#[napi(
  catch_unwind,
  js_name = "findUnusedColorInImage",
  ts_args_type = "img: ImageData, options?: InternalOptions | undefined | null, signal?: AbortSignal | undefined | null",
  ts_return_type = "Promise<Color>"
)]
pub fn find_unused_color_in_image(
  img: JsImageData,
  options: Option<JsInternalOptions>,
  signal: Option<AbortSignal>,
) -> AsyncTask<FindUnusedColorInImageTask> {
  AsyncTask::with_optional_signal(FindUnusedColorInImageTask { img, options }, signal)
}

/// Find an RGB color that does not exist in decoded image data synchronously.
#[napi(
  catch_unwind,
  js_name = "findUnusedColorInImageSync",
  ts_args_type = "img: ImageData, options?: InternalOptions | undefined | null",
  ts_return_type = "Color"
)]
pub fn find_unused_color_in_image_sync(
  img: JsImageData,
  options: Option<JsInternalOptions>,
) -> Result<JsColor> {
  find_unused_color_in_image_inner(img, options)
}

fn read_image_inner(source: &[u8], args: Option<RawDataConfig>) -> Result<ColorImage> {
  js_read_image(source, args).map_err(to_napi_error)
}

fn color_exists_in_image_inner(img: JsImageData, color: JsColor) -> Result<bool> {
  let img = color_image_from_js(img)?;
  let color = color_from_js(color)?;

  Ok(js_color_exists_in_image(&img, color))
}

fn find_unused_color_in_image_inner(
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

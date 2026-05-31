use crate::config::Preset;
use crate::converter::{convert_image_to_svg, convert_image_to_svg_chunks};
use crate::errors::to_napi_error;
use crate::types::{resolve_config, JsConfig, RawDataConfig};
use napi::{
  bindgen_prelude::{AbortSignal, AsyncTask, Buffer, FnArgs, Function},
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
  Either, Result, Status, Task,
};
use std::panic;

type JsCallbackTsfn =
  ThreadsafeFunction<FnArgs<(String, f64)>, (), FnArgs<(String, f64)>, Status, false>;

pub struct VectorizeTask {
  data: Buffer,
  config: Option<Either<JsConfig, Preset>>,
  args: Option<RawDataConfig>,
}

/// Background task used by async vectorization APIs.
#[napi]
impl Task for VectorizeTask {
  type Output = String;
  type JsValue = String;

  fn compute(&mut self) -> Result<Self::Output> {
    vectorize_inner(self.data.as_ref(), self.config.clone(), self.args.clone())
  }

  fn resolve(&mut self, _env: napi::Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output)
  }
}

/// Vectorize an encoded image buffer asynchronously.
#[napi(catch_unwind)]
pub fn vectorize(
  source: Buffer,
  config: Option<Either<JsConfig, Preset>>,
  signal: Option<AbortSignal>,
) -> AsyncTask<VectorizeTask> {
  AsyncTask::with_optional_signal(
    VectorizeTask {
      data: source,
      config,
      args: None,
    },
    signal,
  )
}

/// Vectorize a raw RGBA pixel buffer asynchronously.
#[napi(catch_unwind)]
pub fn vectorize_raw(
  source: Buffer,
  args: RawDataConfig,
  config: Option<Either<JsConfig, Preset>>,
  signal: Option<AbortSignal>,
) -> AsyncTask<VectorizeTask> {
  AsyncTask::with_optional_signal(
    VectorizeTask {
      data: source,
      config,
      args: Some(args),
    },
    signal,
  )
}

/// Vectorize an encoded image buffer synchronously.
#[napi(catch_unwind)]
pub fn vectorize_sync(source: Buffer, config: Option<Either<JsConfig, Preset>>) -> Result<String> {
  vectorize_inner(source.as_ref(), config, None)
}

/// Vectorize a raw RGBA pixel buffer synchronously.
#[napi(catch_unwind)]
pub fn vectorize_raw_sync(
  source: Buffer,
  args: RawDataConfig,
  config: Option<Either<JsConfig, Preset>>,
) -> Result<String> {
  vectorize_inner(source.as_ref(), config, Some(args))
}

/// Vectorize an encoded image buffer and emit SVG chunks to a callback.
#[napi(
  catch_unwind,
  js_name = "vectorizeToCallback",
  ts_args_type = "source: Buffer, config: Config | Preset | undefined | null, callback: (chunk: string, progress: number) => void"
)]
pub fn js_vectorize_to_callback(
  source: Buffer,
  config: Option<Either<JsConfig, Preset>>,
  callback: Function<FnArgs<(String, f64)>, ()>,
) -> Result<()> {
  let tsfn = callback.build_threadsafe_function().build()?;

  std::thread::spawn(move || {
    let result = vectorize_inner_chunks(source.as_ref(), config, None, &tsfn);
    if let Err(e) = result {
      eprintln!("Error in vectorize_to_callback: {:?}", e);
    }
  });

  Ok(())
}

/// Vectorize a raw RGBA pixel buffer and emit SVG chunks to a callback.
#[napi(
  catch_unwind,
  js_name = "vectorizeRawToCallback",
  ts_args_type = "source: Buffer, args: RawDataConfig, config: Config | Preset | undefined | null, callback: (chunk: string, progress: number) => void"
)]
pub fn js_vectorize_raw_to_callback(
  source: Buffer,
  args: RawDataConfig,
  config: Option<Either<JsConfig, Preset>>,
  callback: Function<FnArgs<(String, f64)>, ()>,
) -> Result<()> {
  let tsfn = callback.build_threadsafe_function().build()?;

  std::thread::spawn(move || {
    let result = vectorize_inner_chunks(source.as_ref(), config, Some(args), &tsfn);
    if let Err(e) = result {
      eprintln!("Error in vectorize_to_callback: {:?}", e);
    }
  });

  Ok(())
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
  callback: &JsCallbackTsfn,
) -> Result<()> {
  panic::set_hook(Box::new(|_info| {}));

  let result = panic::catch_unwind(|| {
    convert_image_to_svg_chunks(
      source,
      resolve_config(config),
      raw_args,
      |chunk, progress| {
        let status = callback.call(
          FnArgs {
            data: (chunk, progress),
          },
          ThreadsafeFunctionCallMode::NonBlocking,
        );
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

use napi::{
  bindgen_prelude::{AbortSignal, AsyncTask},
  Result, Task,
};
use oxvg_ast::{parse::roxmltree::parse, serialize::Node as _, visitor::Info};
use oxvg_optimiser::Jobs;

/// Built-in optimization presets.
#[napi(js_name = "OptimizePreset")]
#[derive(Clone)]
pub enum JsOptimizePreset {
  /// Run the default OXVG optimizer job set.
  Default,
  /// Run the safer OXVG optimizer job set.
  Safe,
  /// Start with no optimizer jobs.
  None,
}

/// Options for SVG optimization.
#[derive(Clone)]
#[napi(object, js_name = "OptimizeOptions")]
pub struct JsOptimizeOptions {
  /// Built-in optimizer job preset.
  pub preset: Option<JsOptimizePreset>,
  /// SVGO Config["plugins"] compatible job config. When provided, this defines the optimizer job set.
  #[napi(ts_type = "Array<string | { name: string, params?: unknown }>")]
  pub plugins: Option<Vec<serde_json::Value>>,
  /// Optimizer job names to omit. Names can be camelCase, kebab-case, or snake_case.
  pub omit: Option<Vec<String>>,
  /// Run optimization repeatedly until output stops changing or the iteration limit is reached.
  pub multipass: Option<bool>,
  /// Maximum multipass iterations. Defaults to 10 when multipass is enabled.
  pub multipass_iterations: Option<u32>,
}

pub struct OptimizeTask {
  input: String,
  options: Option<JsOptimizeOptions>,
}

/// Background task used by async SVG optimization.
#[napi]
impl Task for OptimizeTask {
  type Output = String;
  type JsValue = String;

  fn compute(&mut self) -> Result<Self::Output> {
    optimize_svg_inner(self.input.clone(), self.options.clone())
  }

  fn resolve(&mut self, _env: napi::Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output)
  }
}

/// Optimize an SVG string asynchronously.
#[napi(catch_unwind)]
pub fn optimize(
  input: String,
  options: Option<JsOptimizeOptions>,
  signal: Option<AbortSignal>,
) -> AsyncTask<OptimizeTask> {
  AsyncTask::with_optional_signal(OptimizeTask { input, options }, signal)
}

/// Optimize an SVG string synchronously.
#[napi(catch_unwind)]
pub fn optimize_sync(input: String, options: Option<JsOptimizeOptions>) -> Result<String> {
  optimize_svg_inner(input, options)
}

fn optimize_svg_inner(input: String, options: Option<JsOptimizeOptions>) -> Result<String> {
  let jobs = build_jobs(options.as_ref())?;

  if options
    .as_ref()
    .and_then(|options| options.multipass)
    .unwrap_or(false)
  {
    optimize_multipass(input, options.as_ref(), &jobs)
  } else {
    optimize_once(&input, &jobs)
  }
}

fn optimize_multipass(
  input: String,
  options: Option<&JsOptimizeOptions>,
  jobs: &Jobs,
) -> Result<String> {
  let iterations = resolve_multipass_iterations(options)?;
  let mut current = input;

  for _ in 0..iterations {
    let next = optimize_once(&current, jobs)?;
    if next == current {
      return Ok(next);
    }
    current = next;
  }

  Ok(current)
}

fn resolve_multipass_iterations(options: Option<&JsOptimizeOptions>) -> Result<u32> {
  let iterations = options
    .and_then(|options| options.multipass_iterations)
    .unwrap_or(10);

  if iterations == 0 {
    return Err(napi::Error::new(
      napi::Status::InvalidArg,
      "multipassIterations must be greater than 0",
    ));
  }

  Ok(iterations)
}

fn optimize_once(input: &str, jobs: &Jobs) -> Result<String> {
  let result = parse(
    input,
    |dom, allocator| -> std::result::Result<String, String> {
      jobs
        .run(dom, &Info::new(allocator))
        .map_err(|e| format!("failed to optimize SVG: {e}"))?;
      dom
        .serialize()
        .map_err(|e| format!("failed to serialize SVG: {e}"))
    },
  )
  .map_err(|e| {
    napi::Error::new(
      napi::Status::GenericFailure,
      format!("failed to optimize SVG: {e}"),
    )
  })?
  .map_err(|e| napi::Error::new(napi::Status::GenericFailure, e))?;

  Ok(result)
}

fn build_jobs(options: Option<&JsOptimizeOptions>) -> Result<Jobs> {
  let mut jobs = if let Some(plugins) = options.and_then(|options| options.plugins.clone()) {
    Jobs::from_svgo_plugin_config(Some(plugins)).map_err(|error| {
      napi::Error::new(
        napi::Status::InvalidArg,
        format!("failed to parse SVGO plugin config: {error}"),
      )
    })?
  } else {
    match options.and_then(|options| options.preset.clone()) {
      Some(JsOptimizePreset::None) => Jobs::none(),
      Some(JsOptimizePreset::Safe) => Jobs::safe(),
      Some(JsOptimizePreset::Default) | None => Jobs::default(),
    }
  };

  if let Some(omit) = options.and_then(|options| options.omit.as_ref()) {
    for job_name in omit {
      jobs.omit(&normalize_job_name(job_name));
    }
  }

  Ok(jobs)
}

fn normalize_job_name(name: &str) -> String {
  let mut normalized = String::with_capacity(name.len());

  for (index, char) in name.chars().enumerate() {
    if char == '-' {
      normalized.push('_');
      continue;
    }

    if char.is_uppercase() {
      if index > 0 {
        normalized.push('_');
      }
      normalized.push(char.to_ascii_lowercase());
      continue;
    }

    normalized.push(char);
  }

  normalized
}

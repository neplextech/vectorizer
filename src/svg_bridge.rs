use crate::svg::SvgFile;
use napi::Result;

/// Mutable SVG builder exposed to JavaScript.
#[napi(js_name = "SvgFile")]
pub struct JsSvgFile {
  inner: SvgFile,
}

/// Methods for the JavaScript SVG builder.
#[napi]
impl JsSvgFile {
  /// Create an empty SVG document.
  #[napi(constructor)]
  pub fn new(width: u32, height: u32, path_precision: Option<u32>) -> Self {
    Self {
      inner: SvgFile::new(width as usize, height as usize, path_precision),
    }
  }

  /// SVG width in pixels.
  #[napi(getter)]
  pub fn width(&self) -> u32 {
    self.inner.width as u32
  }

  /// SVG height in pixels.
  #[napi(getter)]
  pub fn height(&self) -> u32 {
    self.inner.height as u32
  }

  /// Decimal precision used when writing path data.
  #[napi(getter)]
  pub fn path_precision(&self) -> Option<u32> {
    self.inner.path_precision
  }

  /// Number of paths currently stored in the SVG.
  #[napi(getter)]
  pub fn path_count(&self) -> u32 {
    self.inner.paths.len() as u32
  }

  /// Serialize the SVG document to a string.
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

// Based on https://github.com/visioncortex/vtracer/blob/74f2a04a17d8c246d80c439fb162780160a7c3e9/cmdapp/src/converter.rs

use crate::RawDataConfig;

use super::config::{ColorMode, Config, ConverterConfig, Hierarchical};
use super::svg::SvgFile;
use fastrand::Rng;
use image;
use visioncortex::color_clusters::{KeyingAction, Runner, RunnerConfig, HIERARCHICAL_MAX};
use visioncortex::{
  approximate_circle_with_spline, Color, ColorImage, ColorName, CompoundPath, PathSimplifyMode,
};

#[allow(dead_code)]
const NUM_UNUSED_COLOR_ITERATIONS: usize = 6;
/// The fraction of pixels in the top/bottom rows of the image that need to be transparent before
/// the entire image will be keyed.
#[allow(dead_code)]
const KEYING_THRESHOLD: f32 = 0.2;

#[allow(dead_code)]
const SMALL_CIRCLE: i32 = 12;

/// Convert an in-memory image into an in-memory SVG
pub fn convert(img: ColorImage, config: Config) -> Result<SvgFile, String> {
  let config = config.into_converter_config();
  match config.color_mode {
    ColorMode::Color => color_image_to_svg(img, config),
    ColorMode::Binary => binary_image_to_svg(img, config),
  }
}

/// Convert an image file into svg file
pub fn convert_image_to_svg(
  input: &[u8],
  config: Config,
  raw: Option<RawDataConfig>,
) -> Result<String, String> {
  let img = read_image(input, raw)?;
  let svg = convert(img, config)?;
  let str = svg.to_string().map_err(|e| e.to_string())?;
  Ok(str)
}

pub fn convert_image_to_svg_chunks<F>(
  input: &[u8],
  config: Config,
  raw: Option<RawDataConfig>,
  write: F,
) -> Result<(), String>
where
  F: FnMut(String, f64) -> Result<(), String>,
{
  let img = read_image(input, raw)?;
  let svg = convert(img, config)?;
  svg.write_chunks(write)
}

fn color_exists_in_image(img: &ColorImage, color: Color) -> bool {
  img
    .pixels
    .chunks_exact(4)
    .any(|pixel| pixel[0] == color.r && pixel[1] == color.g && pixel[2] == color.b)
}

#[allow(dead_code)]
fn find_unused_color_in_image(img: &ColorImage) -> Result<Color, String> {
  find_unused_color_in_image_with_iterations(img, NUM_UNUSED_COLOR_ITERATIONS)
}

fn find_unused_color_in_image_with_iterations(
  img: &ColorImage,
  unused_color_iterations: usize,
) -> Result<Color, String> {
  let special_colors = IntoIterator::into_iter([
    Color::new(255, 0, 0),
    Color::new(0, 255, 0),
    Color::new(0, 0, 255),
    Color::new(255, 255, 0),
    Color::new(0, 255, 255),
    Color::new(255, 0, 255),
  ]);
  let mut rng = Rng::new();
  let random_colors =
    (0..unused_color_iterations).map(|_| Color::new(rng.u8(..), rng.u8(..), rng.u8(..)));
  for color in special_colors.chain(random_colors) {
    if !color_exists_in_image(img, color) {
      return Ok(color);
    }
  }
  Err(String::from(
    "unable to find unused color in image to use as key",
  ))
}

#[allow(dead_code)]
fn should_key_image(img: &ColorImage) -> bool {
  should_key_image_with_threshold(img, KEYING_THRESHOLD)
}

fn should_key_image_with_threshold(img: &ColorImage, keying_threshold: f32) -> bool {
  if img.width == 0 || img.height == 0 {
    return false;
  }

  // Check for transparency at several scanlines
  let threshold = ((img.width * 2) as f32 * keying_threshold) as usize;
  let mut num_transparent_boundary_pixels = 0;
  let y_positions = [
    0,
    img.height / 4,
    img.height / 2,
    3 * img.height / 4,
    img.height - 1,
  ];
  for y in y_positions {
    let row_start = y * img.width * 4;
    for x in 0..img.width {
      let alpha_index = row_start + x * 4 + 3;
      if img.pixels[alpha_index] == 0 {
        num_transparent_boundary_pixels += 1;
      }
      if num_transparent_boundary_pixels >= threshold {
        return true;
      }
    }
  }

  false
}

fn color_image_to_svg(mut img: ColorImage, config: ConverterConfig) -> Result<SvgFile, String> {
  let width = img.width;
  let height = img.height;

  let key_color = if should_key_image_with_threshold(&img, config.keying_threshold) {
    let key_color =
      find_unused_color_in_image_with_iterations(&img, config.unused_color_iterations)?;
    for y in 0..height {
      for x in 0..width {
        if img.get_pixel(x, y).a == 0 {
          img.set_pixel(x, y, &key_color);
        }
      }
    }
    key_color
  } else {
    // The default color is all zeroes, which is treated by visioncortex as a special value meaning no keying will be applied.
    Color::default()
  };

  let runner = Runner::new(
    RunnerConfig {
      diagonal: config.layer_difference == 0,
      hierarchical: HIERARCHICAL_MAX,
      batch_size: 25600,
      good_min_area: config.filter_speckle_area,
      good_max_area: (width * height),
      is_same_color_a: config.color_precision_loss,
      is_same_color_b: 1,
      deepen_diff: config.layer_difference,
      hollow_neighbours: 1,
      key_color,
      keying_action: if matches!(config.hierarchical, Hierarchical::Cutout) {
        KeyingAction::Keep
      } else {
        KeyingAction::Discard
      },
    },
    img,
  );

  let mut clusters = runner.run();

  match config.hierarchical {
    Hierarchical::Stacked => {}
    Hierarchical::Cutout => {
      let view = clusters.view();
      let image = view.to_color_image();
      let runner = Runner::new(
        RunnerConfig {
          diagonal: false,
          hierarchical: 64,
          batch_size: 25600,
          good_min_area: 0,
          good_max_area: (image.width * image.height),
          is_same_color_a: 0,
          is_same_color_b: 1,
          deepen_diff: 0,
          hollow_neighbours: 0,
          key_color,
          keying_action: KeyingAction::Discard,
        },
        image,
      );
      clusters = runner.run();
    }
  }

  let view = clusters.view();

  let mut svg = SvgFile::new(width, height, config.path_precision);
  for &cluster_index in view.clusters_output.iter().rev() {
    let cluster = view.get_cluster(cluster_index);
    let paths = if matches!(config.mode, PathSimplifyMode::Spline)
      && cluster.rect.width() < config.small_circle
      && cluster.rect.height() < config.small_circle
      && cluster.to_shape(&view).is_circle()
    {
      let mut paths = CompoundPath::new();
      paths.add_spline(approximate_circle_with_spline(
        cluster.rect.left_top(),
        cluster.rect.width(),
      ));
      paths
    } else {
      cluster.to_compound_path(
        &view,
        false,
        config.mode,
        config.corner_threshold,
        config.length_threshold,
        config.max_iterations,
        config.splice_threshold,
      )
    };
    svg.add_path(paths, cluster.residue_color());
  }

  Ok(svg)
}

fn binary_image_to_svg(img: ColorImage, config: ConverterConfig) -> Result<SvgFile, String> {
  let img = img.to_binary_image(|x| x.r < 128);
  let width = img.width;
  let height = img.height;

  let clusters = img.to_clusters(false);

  let mut svg = SvgFile::new(width, height, config.path_precision);
  for i in 0..clusters.len() {
    let cluster = clusters.get_cluster(i);
    if cluster.size() >= config.filter_speckle_area {
      let paths = cluster.to_compound_path(
        config.mode,
        config.corner_threshold,
        config.length_threshold,
        config.max_iterations,
        config.splice_threshold,
      );
      svg.add_path(paths, Color::color(&ColorName::Black));
    }
  }

  Ok(svg)
}

fn read_image(input: &[u8], raw: Option<RawDataConfig>) -> Result<ColorImage, String> {
  match raw {
    Some(raw) => {
      let width = raw.width as usize;
      let height = raw.height as usize;

      if width == 0 || height == 0 {
        return Err(String::from("width and height must be positive"));
      }

      let expected_len = width
        .checked_mul(height)
        .and_then(|size| size.checked_mul(4))
        .ok_or_else(|| String::from("Image dimensions are too large"))?;

      if input.len() != expected_len {
        return Err(format!(
          "pixel data length {} does not match width * height * 4 {}",
          input.len(),
          expected_len
        ));
      }

      let img = ColorImage {
        pixels: input.to_vec(),
        width,
        height,
      };
      Ok(img)
    }
    None => {
      let img = image::load_from_memory(input);
      let img = match img {
        Ok(file) => file.to_rgba8(),
        Err(_) => return Err(String::from("unable to read this image")),
      };

      let (width, height) = (img.width() as usize, img.height() as usize);
      let img = ColorImage {
        pixels: img.as_raw().to_vec(),
        width,
        height,
      };
      Ok(img)
    }
  }
}

pub(crate) fn js_read_image(
  input: &[u8],
  raw: Option<RawDataConfig>,
) -> Result<ColorImage, String> {
  read_image(input, raw)
}

pub(crate) fn js_color_exists_in_image(img: &ColorImage, color: Color) -> bool {
  color_exists_in_image(img, color)
}

pub(crate) fn js_find_unused_color_in_image(
  img: &ColorImage,
  unused_color_iterations: usize,
) -> Result<Color, String> {
  find_unused_color_in_image_with_iterations(img, unused_color_iterations)
}

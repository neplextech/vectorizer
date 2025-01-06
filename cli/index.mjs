#!/usr/bin/env node

import { Command } from 'commander';
import pc from 'picocolors';
import fs from 'node:fs/promises';
import { vectorize, Preset, ColorMode, Hierarchical, PathSimplifyMode } from '../index.js';

const { version } = JSON.parse(await fs.readFile(new URL('../package.json', import.meta.url)));
const program = new Command();

program
  .name('vectorizer')
  .description('Convert raster images to SVG vector graphics')
  .argument('<input>', 'Input image file path')
  .argument('[output]', 'Output SVG file path (defaults to input path with .svg extension)')
  .option('--print', 'Print the SVG output to terminal instead of writing to file')
  .option('-p, --preset <preset>', 'Use preset configuration (bw|poster|photo)', 'photo')
  .option('--color-mode <mode>', 'Color mode (color|binary)', 'color')
  .option('--hierarchical <mode>', 'Hierarchical mode (stacked|cutout)', 'stacked')
  .option('--filter-speckle <pixels>', 'Filter speckles smaller than X pixels', parseInt)
  .option('--color-precision <bits>', 'Color precision in bits', parseInt)
  .option('--layer-difference <value>', 'Color difference between layers', parseFloat)
  .option('--mode <mode>', 'Path simplify mode (none|polygon|spline)', 'spline')
  .option('--corner-threshold <degrees>', 'Corner threshold in degrees', parseFloat)
  .option('--length-threshold <value>', 'Length threshold', parseFloat)
  .option('--max-iterations <count>', 'Maximum iterations', parseInt)
  .option('--splice-threshold <degrees>', 'Splice threshold in degrees', parseFloat)
  .option('--path-precision <decimals>', 'Path precision decimal places', parseInt)
  .version(version, '-v, --version', 'Output the current version');

async function main() {
  try {
    program.parse();

    const [inputPath, outputPath] = program.args;
    const options = program.opts();

    if (options.print && outputPath) {
      console.error(pc.yellow('Warning: Output path is ignored when using --print flag'));
    }

    // Validate input file
    try {
      await fs.access(inputPath);
    } catch {
      console.error(pc.red(`Error: Input file '${inputPath}' does not exist`));
      process.exit(1);
    }

    // Determine output path
    const finalOutputPath = outputPath || inputPath.replace(/\.[^.]+$/, '.svg');

    // Read input file
    const inputBuffer = await fs.readFile(inputPath);

    // Parse preset
    let config = null;
    if (options.preset) {
      const presetMap = { bw: Preset.Bw, poster: Preset.Poster, photo: Preset.Photo };
      const preset = presetMap[options.preset.toLowerCase()];
      if (preset !== undefined) {
        config = preset;
      }
    }

    // If not using preset, build config object
    if (config === null && Object.keys(options).length > 1) {
      config = {
        colorMode: options.colorMode === 'binary' ? ColorMode.Binary : ColorMode.Color,
        hierarchical: options.hierarchical === 'cutout' ? Hierarchical.Cutout : Hierarchical.Stacked,
        filterSpeckle: options.filterSpeckle,
        colorPrecision: options.colorPrecision,
        layerDifference: options.layerDifference,
        mode:
          {
            none: PathSimplifyMode.None,
            polygon: PathSimplifyMode.Polygon,
            spline: PathSimplifyMode.Spline,
          }[options.mode] || PathSimplifyMode.Spline,
        cornerThreshold: options.cornerThreshold,
        lengthThreshold: options.lengthThreshold,
        maxIterations: options.maxIterations,
        spliceThreshold: options.spliceThreshold,
        pathPrecision: options.pathPrecision,
      };

      // Remove undefined values
      Object.keys(config).forEach((key) => config[key] === undefined && delete config[key]);
    }

    console.log(pc.blue(`Converting image ${pc.underline(inputPath)} to svg...`), '\n');

    // Process the image
    const start = performance.now();
    const svg = await vectorize(inputBuffer, config);
    const end = performance.now() - start;

    // Handle output based on print flag
    if (options.print) {
      console.log(pc.green(svg));
      console.log(pc.blue(`Vectorization took ${end.toFixed(2)}ms`));
    } else {
      // Write output file
      await fs.writeFile(finalOutputPath, svg, 'utf8');
      console.log(pc.green(`Successfully converted to ${finalOutputPath} in ${end.toFixed(2)}ms`));
    }
  } catch (error) {
    console.error(pc.red('Error:'), error.message);
    process.exit(1);
  }
}

main();

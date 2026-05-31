#!/usr/bin/env node

import { Command, Option } from 'commander';
import pc from 'picocolors';
import fs from 'node:fs/promises';
import vectorizer from '../index.js';

const {
  ColorMode,
  Hierarchical,
  OptimizePreset,
  PathSimplifyMode,
  Preset,
  isEOF,
  optimize,
  vectorizeRawToCallback,
  vectorizeToCallback,
} = vectorizer;

const { version } = JSON.parse(await fs.readFile(new URL('../package.json', import.meta.url)));
const program = new Command();
const progressBarWidth = 28;

program
  .name('vectorizer')
  .description('Convert raster images to SVG vector graphics')
  .argument('<input>', 'Input image file path')
  .argument('[output]', 'Output SVG file path (defaults to input path with .svg extension)')
  .option('--print', 'Print the SVG output to stdout instead of writing to file')
  .option('-p, --preset <preset>', 'Use preset configuration (bw|poster|photo)', 'photo')
  .option('--raw <size>', 'Read input as raw RGBA pixels with dimensions WIDTHxHEIGHT')
  .option('--color-mode <mode>', 'Color mode (color|binary)', 'color')
  .option('--hierarchical <mode>', 'Hierarchical mode (stacked|cutout)', 'stacked')
  .option('--filter-speckle <pixels>', 'Filter speckles smaller than X pixels', parseInteger)
  .option('--color-precision <bits>', 'Color precision in bits', parseInteger)
  .option('--layer-difference <value>', 'Color difference between layers', parseNumber)
  .option('--mode <mode>', 'Path simplify mode (none|polygon|spline)', 'spline')
  .option('--corner-threshold <degrees>', 'Corner threshold in degrees', parseNumber)
  .option('--length-threshold <value>', 'Length threshold', parseNumber)
  .option('--max-iterations <count>', 'Maximum iterations', parseInteger)
  .option('--splice-threshold <degrees>', 'Splice threshold in degrees', parseNumber)
  .option('--path-precision <decimals>', 'Path precision decimal places', parseInteger)
  .option('--optimize', 'Optimize the generated SVG before output')
  .addOption(
    new Option('--optimize-preset <preset>', 'Optimizer preset (default|safe|none)').choices([
      'default',
      'safe',
      'none',
    ]),
  )
  .option('--optimize-plugin <plugin>', 'SVGO plugin to enable while optimizing. Can be repeated.', collect)
  .option('--optimize-omit <job>', 'Optimizer job to omit. Can be repeated.', collect)
  .option('--multipass', 'Run optimizer repeatedly until output stops changing')
  .option('--multipass-iterations <count>', 'Maximum optimizer multipass iterations', parseInteger)
  .version(version, '-v, --version', 'Output the current version')
  .action(runVectorizeCommand);

program
  .command('optimize')
  .description('Optimize an existing SVG file')
  .argument('<input>', 'Input SVG file path')
  .argument('[output]', 'Output SVG file path (defaults to input path with .optimized.svg suffix)')
  .option('--print', 'Print the optimized SVG to stdout instead of writing to file')
  .addOption(
    new Option('--preset <preset>', 'Optimizer preset (default|safe|none)').choices(['default', 'safe', 'none']),
  )
  .option('--plugin <plugin>', 'SVGO plugin to enable. Can be repeated.', collect)
  .option('--omit <job>', 'Optimizer job to omit. Can be repeated.', collect)
  .option('--multipass', 'Run optimizer repeatedly until output stops changing')
  .option('--multipass-iterations <count>', 'Maximum optimizer multipass iterations', parseInteger)
  .action(runOptimizeCommand);

program.parseAsync().catch((error) => {
  console.error(pc.red('Error:'), error.message);
  process.exit(1);
});

async function runVectorizeCommand(inputPath, outputPath, options, command) {
  if (options.print && outputPath) {
    console.error(pc.yellow('Warning: Output path is ignored when using --print flag'));
  }

  await assertReadable(inputPath, 'Input file');

  const finalOutputPath = outputPath || inputPath.replace(/\.[^.]+$/, '.svg');
  const inputBuffer = await fs.readFile(inputPath);
  const config = buildVectorizeConfig(options, command);
  const rawArgs = options.raw ? parseRawSize(options.raw) : null;

  console.error(pc.blue(`Converting ${pc.underline(inputPath)} to SVG`));

  const vectorizeStart = performance.now();
  const printChunks = options.print && !options.optimize;
  const collectOptions = {
    printChunks,
    showProgress: !options.print,
  };
  const svg = rawArgs
    ? await collectVectorizeChunks(
        (callback) => vectorizeRawToCallback(inputBuffer, rawArgs, config, callback),
        collectOptions,
      )
    : await collectVectorizeChunks((callback) => vectorizeToCallback(inputBuffer, config, callback), collectOptions);
  const vectorizeDuration = performance.now() - vectorizeStart;

  if (printChunks) {
    console.error(pc.blue(`Vectorized in ${vectorizeDuration.toFixed(2)}ms`));
    return;
  }

  const outputSvg = options.optimize
    ? await optimizeWithTiming(svg, buildOptimizeOptions(options, 'optimize'), 'Optimized generated SVG')
    : svg;

  if (options.print) {
    process.stdout.write(outputSvg);
    console.error(pc.blue(`Vectorized in ${vectorizeDuration.toFixed(2)}ms`));
    return;
  }

  await fs.writeFile(finalOutputPath, outputSvg, 'utf8');
  console.error(pc.green(`Wrote ${finalOutputPath}`));
  console.error(pc.blue(`Vectorized in ${vectorizeDuration.toFixed(2)}ms`));
}

async function runOptimizeCommand(inputPath, outputPath, options) {
  if (options.print && outputPath) {
    console.error(pc.yellow('Warning: Output path is ignored when using --print flag'));
  }

  await assertReadable(inputPath, 'Input SVG file');

  const finalOutputPath = outputPath || inputPath.replace(/\.svg$/i, '.optimized.svg');
  const svg = await fs.readFile(inputPath, 'utf8');
  const optimized = await optimizeWithTiming(svg, buildOptimizeOptions(options), `Optimized ${inputPath}`);

  if (options.print) {
    process.stdout.write(optimized);
    return;
  }

  await fs.writeFile(finalOutputPath, optimized, 'utf8');
  console.error(pc.green(`Wrote ${finalOutputPath}`));
}

async function assertReadable(filePath, label) {
  try {
    await fs.access(filePath);
  } catch {
    throw new Error(`${label} '${filePath}' does not exist`);
  }
}

function buildVectorizeConfig(options, command) {
  const hasCustomConfig = [
    'colorMode',
    'hierarchical',
    'filterSpeckle',
    'colorPrecision',
    'layerDifference',
    'mode',
    'cornerThreshold',
    'lengthThreshold',
    'maxIterations',
    'spliceThreshold',
    'pathPrecision',
  ].some((name) => command.getOptionValueSource(name) === 'cli');

  if (!hasCustomConfig) {
    return resolveVectorizePreset(options.preset);
  }

  const config = {
    colorMode: parseColorMode(options.colorMode),
    hierarchical: parseHierarchical(options.hierarchical),
    filterSpeckle: options.filterSpeckle ?? 4,
    colorPrecision: options.colorPrecision ?? 8,
    layerDifference: options.layerDifference ?? 16,
    mode: parsePathMode(options.mode),
    cornerThreshold: options.cornerThreshold ?? 60,
    lengthThreshold: options.lengthThreshold ?? 4,
    maxIterations: options.maxIterations ?? 10,
    spliceThreshold: options.spliceThreshold ?? 45,
    pathPrecision: options.pathPrecision ?? 2,
  };

  Object.keys(config).forEach((key) => config[key] === undefined && delete config[key]);
  return config;
}

function buildOptimizeOptions(options, prefix = '') {
  const preset = prefix ? options[`${prefix}Preset`] : options.preset;
  const plugins = prefix ? options[`${prefix}Plugin`] : options.plugin;
  const omit = prefix ? options[`${prefix}Omit`] : options.omit;
  const optimizeOptions = {};

  if (preset) {
    optimizeOptions.preset = parseOptimizePreset(preset);
  }

  if (plugins?.length) {
    optimizeOptions.plugins = plugins.map(parsePluginConfig);
  }

  if (omit?.length) {
    optimizeOptions.omit = omit;
  }

  if (options.multipass) {
    optimizeOptions.multipass = true;
  }

  if (options.multipassIterations !== undefined) {
    optimizeOptions.multipassIterations = options.multipassIterations;
  }

  return optimizeOptions;
}

async function optimizeWithTiming(svg, options, label) {
  const start = performance.now();
  const optimized = await optimize(svg, options);
  const duration = performance.now() - start;
  const delta = svg.length - optimized.length;

  console.error(pc.green(`${label} in ${duration.toFixed(2)}ms`));
  console.error(pc.dim(`Optimization length: ${svg.length} -> ${optimized.length} (${formatDelta(delta)} bytes)`));

  return optimized;
}

function collectVectorizeChunks(start, { printChunks = false, showProgress = true } = {}) {
  const chunks = printChunks ? null : [];
  let chunkCount = 0;
  const begin = performance.now();

  return new Promise((resolve, reject) => {
    try {
      start((chunk, progress) => {
        chunkCount += 1;
        if (printChunks) {
          process.stdout.write(chunk);
        } else {
          chunks.push(chunk);
        }

        if (showProgress) {
          renderProgressBar(progress, chunkCount, begin);
        }

        if (isEOF(chunk, progress)) {
          if (showProgress) {
            process.stderr.write('\n');
          }
          resolve(printChunks ? '' : chunks.join(''));
        }
      });
    } catch (error) {
      reject(error);
    }
  });
}

function renderProgressBar(progress, chunkCount, begin) {
  const elapsedMilliSeconds = Math.max(performance.now() - begin, 0.001);
  const chunksPerMilliSecond = chunkCount / elapsedMilliSeconds;
  const filledLength = Math.round((progress / 100) * progressBarWidth);
  const emptyLength = progressBarWidth - filledLength;
  const bar = `${'#'.repeat(filledLength)}${'-'.repeat(emptyLength)}`;

  process.stderr.write(
    `\rProgress: [${bar}] ${progress.toFixed(2)}% | ${chunkCount} chunks | ${chunksPerMilliSecond.toFixed(2)} chunks/ms`,
  );
}

function resolveVectorizePreset(value) {
  const preset = { bw: Preset.Bw, poster: Preset.Poster, photo: Preset.Photo }[value.toLowerCase()];
  if (preset === undefined) {
    throw new Error(`Unknown preset '${value}'. Expected bw, poster, or photo`);
  }
  return preset;
}

function parseOptimizePreset(value) {
  return {
    default: OptimizePreset.Default,
    safe: OptimizePreset.Safe,
    none: OptimizePreset.None,
  }[value];
}

function parseColorMode(value) {
  if (value === 'binary') return ColorMode.Binary;
  if (value === 'color') return ColorMode.Color;
  throw new Error(`Unknown color mode '${value}'. Expected color or binary`);
}

function parseHierarchical(value) {
  if (value === 'cutout') return Hierarchical.Cutout;
  if (value === 'stacked') return Hierarchical.Stacked;
  throw new Error(`Unknown hierarchical mode '${value}'. Expected stacked or cutout`);
}

function parsePathMode(value) {
  const mode = {
    none: PathSimplifyMode.None,
    polygon: PathSimplifyMode.Polygon,
    spline: PathSimplifyMode.Spline,
  }[value];

  if (mode === undefined) {
    throw new Error(`Unknown path mode '${value}'. Expected none, polygon, or spline`);
  }

  return mode;
}

function parseRawSize(value) {
  const match = /^(\d+)x(\d+)$/i.exec(value);
  if (!match) {
    throw new Error(`Invalid raw size '${value}'. Expected WIDTHxHEIGHT`);
  }

  return {
    width: Number(match[1]),
    height: Number(match[2]),
  };
}

function parsePluginConfig(value) {
  const trimmed = value.trim();

  if (trimmed.startsWith('{')) {
    return JSON.parse(trimmed);
  }

  return trimmed;
}

function parseInteger(value) {
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed)) {
    throw new Error(`Expected integer, received '${value}'`);
  }
  return parsed;
}

function parseNumber(value) {
  const parsed = Number.parseFloat(value);
  if (!Number.isFinite(parsed)) {
    throw new Error(`Expected number, received '${value}'`);
  }
  return parsed;
}

function collect(value, previous = []) {
  previous.push(value);
  return previous;
}

function formatDelta(delta) {
  if (delta > 0) return `-${delta}`;
  if (delta < 0) return `+${Math.abs(delta)}`;
  return '0';
}

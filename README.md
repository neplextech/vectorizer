# @neplex/vectorizer

Node.js library to convert raster images to svg using [VTracer](https://github.com/visioncortex/vtracer), with time complexity of `O(n)`.

## CLI

```bash
npx @neplex/vectorizer ./raster.png ./vector.svg
```

Optimize while vectorizing:

```bash
npx @neplex/vectorizer ./raster.png ./vector.svg --optimize --multipass --multipass-iterations 5
```

Optimize an existing SVG:

```bash
npx @neplex/vectorizer optimize ./vector.svg ./vector.optimized.svg --plugin preset-default
```

Use `--help` to see all available options.

## Installation

```bash
npm install @neplex/vectorizer
```

## Usage

```js
import { vectorize, ColorMode, Hierarchical, PathSimplifyMode } from '@neplex/vectorizer';
import { readFile, writeFile } from 'node:fs/promises';

const src = await readFile('./raster.png');

const svg = await vectorize(src, {
  colorMode: ColorMode.Color,
  colorPrecision: 6,
  filterSpeckle: 4,
  spliceThreshold: 45,
  cornerThreshold: 60,
  hierarchical: Hierarchical.Stacked,
  mode: PathSimplifyMode.Spline,
  layerDifference: 5,
  lengthThreshold: 5,
  maxIterations: 2,
  pathPrecision: 5,
});

console.log(svg); // <svg>...</svg>
await writeFile('./vector.svg', svg);
```

The generated SVG string is generally quite large, so it's recommended to optimize it before use. See the [SVG Optimization section](#svg-optimization) for more details. If you want to use a synchronous API, you can use `vectorizeSync` instead. See the [API section](#api) for more details on available functions and options.

## Benchmark

```js
yarn bench
clk: ~5.32 GHz
cpu: Intel(R) Core(TM) i7-14700K
runtime: node 24.14.1 (x64-win32)

benchmark                          avg (min … max) p75 / p99    (min … top 1%)
-------------------------------------------------- -------------------------------
@neplex/vectorizer raw sync         529.82 µs/iter 533.70 µs        ▅█▅
                           (517.00 µs … 607.00 µs) 552.90 µs   ▅██▆▇████▄
                           (  1.65 kb … 168.67 kb)   1.94 kb ▃███████████▆▄▃▂▁▂▂▂▁

@neplex/vectorizer encoded sync     549.19 µs/iter 552.70 µs       ▃█▆▇
                           (534.80 µs … 597.10 µs) 573.50 µs    ▃▆▆████▆▄
                           (  1.60 kb …  74.57 kb)   1.70 kb ▂▆██████████▆▄▂▂▂▂▂▁▁

@neplex/vectorizer encoded async    585.94 µs/iter 586.00 µs   ▇█
                           (561.70 µs … 745.10 µs) 691.10 µs   ██
                           (  2.27 kb …  76.16 kb)   2.34 kb ▂▅███▅▃▂▂▁▁▁▁▁▁▂▁▁▁▁▁

@neplex/vectorizer encoded callback 684.02 µs/iter 698.80 µs    ▃▅▃▃█▂
                           (632.30 µs … 844.90 µs) 770.60 µs  ▂▅██████▇▃
                           (  4.27 kb …  44.40 kb)   4.48 kb ▂████████████▇▅▆▆▅▃▃▂

@neplex/vectorizer raw async        565.45 µs/iter 565.20 µs  ▂█
                           (548.70 µs … 706.00 µs) 646.00 µs  ██▃
                           (  2.33 kb … 186.42 kb)   2.58 kb ▂███▅▃▃▂▂▁▂▁▂▁▁▂▁▁▁▁▁

@neplex/vectorizer raw callback     625.91 µs/iter 636.20 µs   ▃█▇
                           (591.60 µs … 779.20 µs) 704.80 µs   ███▇
                           (  3.98 kb … 374.13 kb)   4.62 kb ▂████████▄▅▆▅▆▄▄▃▁▂▁▂

imagetracerjs                         2.41 ms/iter   2.42 ms   █▇▂
                               (2.33 ms … 2.93 ms)   2.73 ms  ████▂
                           (  1.14 mb …   6.92 mb)   4.52 mb ▂█████▃▃▃▃▂▂▂▂▁▂▁▁▁▁▁

potrace trace                         3.59 ms/iter   4.45 ms      █▅
                               (1.70 ms … 7.43 ms)   6.36 ms  ▆▃█▂██▃ ▃▅▂  ▄
                           (736.00  b …  11.91 mb)   2.27 mb ▃██████████████▇▃█▁▆▃

summary
  @neplex/vectorizer raw sync
   1.04x faster than @neplex/vectorizer encoded sync
   1.07x faster than @neplex/vectorizer raw async
   1.11x faster than @neplex/vectorizer encoded async
   1.18x faster than @neplex/vectorizer raw callback
   1.29x faster than @neplex/vectorizer encoded callback
   4.54x faster than imagetracerjs
   6.78x faster than potrace trace
```

See [benchmark](https://github.com/neplextech/vectorizer/blob/main/benchmark/bench.ts) for more details.

## API

### `vectorize(data: Buffer, config?: Config | Preset, signal?: AbortSignal): Promise<string>`

Takes an image buffer and returns a promise that resolves to an SVG string.

### `vectorizeSync(data: Buffer, config?: Config | Preset): string`

Takes an image buffer and returns an SVG string synchronously.

### `vectorizeRaw(data: Buffer, args: RawDataConfig, config?: Config | Preset, signal?: AbortSignal): Promise<string>`

Takes a raw pixel data buffer and returns a promise that resolves to an SVG string.

### `vectorizeRawSync(data: Buffer, args: RawDataConfig, config?: Config | Preset): string`

Takes a raw pixel data buffer and returns an SVG string synchronously.

### `vectorizeToCallback(data: Buffer, config: Config | Preset | null | undefined, callback: (chunk: string, progress: number) => void): void`

Vectorizes an image buffer and emits SVG chunks to the callback. `progress` is a number from 0 to 100 indicating how much of the vectorization has been written.

### `vectorizeRawToCallback(data: Buffer, args: RawDataConfig, config: Config | Preset | null | undefined, callback: (chunk: string, progress: number) => void): void`

Vectorizes raw pixel data and emits SVG chunks to the callback. `progress` is a number from 0 to 100.

### `readImage(data: Buffer, args?: RawDataConfig, signal?: AbortSignal): Promise<ImageData>`

Decodes an encoded image buffer, or a raw RGBA buffer when `args` is provided, and resolves image width, height, and pixels.

### `readImageSync(data: Buffer, args?: RawDataConfig): ImageData`

Decodes an encoded image buffer, or a raw RGBA buffer when `args` is provided, synchronously.

### `colorExistsInImage(image: ImageData, color: Color, signal?: AbortSignal): Promise<boolean>`

Checks asynchronously whether a decoded image contains the given RGB color.

### `colorExistsInImageSync(image: ImageData, color: Color): boolean`

Checks synchronously whether a decoded image contains the given RGB color.

### `findUnusedColorInImage(image: ImageData, options?: InternalOptions, signal?: AbortSignal): Promise<Color>`

Finds asynchronously a color that does not exist in decoded image data.

### `findUnusedColorInImageSync(image: ImageData, options?: InternalOptions): Color`

Finds synchronously a color that does not exist in decoded image data.

### `optimize(svg: string, options?: OptimizeOptions, signal?: AbortSignal): Promise<string>`

Optimizes an SVG string asynchronously. Supports the same options as `optimizeSync`.

### `optimizeSync(svg: string, options?: OptimizeOptions): string`

Optimizes an SVG string synchronously.

`OptimizeOptions` supports:

- `preset?: OptimizePreset`: `Default`, `Safe`, or `None`.
- `plugins?: Array<string | { name: string, params?: unknown }>`: SVGO `Config["plugins"]` compatible job config. When provided, this defines the optimizer job set.
- `omit?: string[]`: optimizer job names to skip.
- `multipass?: boolean`: run optimization until output stops changing or the iteration limit is reached.
- `multipassIterations?: number`: maximum multipass iterations. Defaults to `10`.

### `isEOF(chunk: string, progress: number): boolean`

Determines if the given chunk data indicates the end of the SVG output. Useful for callback-based vectorization to know when the final chunk has been received. Note that EOF here refers to the final chunk of svg, which is `</svg>\n`.

```js
const { promise, resolve } = Promise.withResolvers<void>();

vectorizeToCallback(src, config, (chunk, progress) => {
  if (isEOF(chunk, progress)) {
    resolve();
  }
});

await promise; // block until all the SVG chunks have been received
console.log('SVG output complete!');
```

## SVG Optimization

The raw output from `vectorize` can be significantly reduced in size using the built-in optimizer powered by [oxvg_optimiser](https://github.com/noahbald/oxvg):

```js
import { optimize, vectorize } from '@neplex/vectorizer';
import { readFile, writeFile } from 'node:fs/promises';

const src = await readFile('./input.png');
const svg = await vectorize(src);
const optimized = await optimize(
  svg,
  /* optional config */ {
    plugins: ['preset-default', { name: 'removeTitle' }], // supports svgo plugin config
    multipass: true, // enable multipass optimization
    multipassIterations: 5, // limit multipass iterations to 5
  },
);

await writeFile('./output.svg', optimized);
```

> Note that the optimizer included in this library is a thin wrapper around `oxvg_optimiser` and does not intend to provide all the features of `oxvg`. For advanced needs, consider using `@oxvg/napi` directly.

## Demo

Generated under the following configuration:

```js
{
    colorMode: ColorMode.Color,
    colorPrecision: 8,
    filterSpeckle: 4,
    spliceThreshold: 45,
    cornerThreshold: 60,
    hierarchical: Hierarchical.Stacked,
    mode: PathSimplifyMode.Spline,
    layerDifference: 6,
    lengthThreshold: 4,
    maxIterations: 2
}
```

| Raster Image (PNG Input)                                                                                                                                                                                                         | Vector Image (Generated SVG)                                                                                                                                                                                                           |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ![Raster Image](https://raw.githubusercontent.com/neplextech/vectorizer/main/example/anime-girl.png)<br/>[CC-BY-SA 3.0](https://creativecommons.org/licenses/by/3.0) by [Niabot](https://commons.wikimedia.org/wiki/User:Niabot) | ![Vector Image](https://raw.githubusercontent.com/neplextech/vectorizer/main/example/result-optimized.svg)<br/>[CC-BY-SA 3.0](https://creativecommons.org/licenses/by/3.0) by [Niabot](https://commons.wikimedia.org/wiki/User:Niabot) |

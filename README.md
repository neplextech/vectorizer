# @neplex/vectorizer

Node.js library to convert raster images to svg using [VTracer](https://github.com/visioncortex/vtracer), with time complexity of `O(n)`.

## CLI

```bash
npx @neplex/vectorizer ./raster.png ./vector.svg
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

If you want to use a synchronous API, you can use `vectorizeSync` instead.

## Benchmark

```js
yarn bench
clk: ~5.30 GHz
cpu: Intel(R) Core(TM) i7-14700K
runtime: node 24.14.1 (x64-win32)

benchmark                       avg (min … max) p75 / p99    (min … top 1%)
----------------------------------------------- -------------------------------
@neplex/vectorizer raw sync      532.14 µs/iter 533.70 µs    ▂▇█▃▃
                        (524.20 µs … 590.90 µs) 550.90 µs   ▂█████
                        (  1.65 kb … 168.67 kb)   1.94 kb ▂▄███████▇▅▃▄▂▁▁▂▁▁▁▁

@neplex/vectorizer encoded sync  552.92 µs/iter 554.70 µs      █▄▂
                        (544.90 µs … 613.00 µs) 569.60 µs    ▅████▆▃
                        (  1.60 kb …  74.57 kb)   1.70 kb ▂▄████████▇▅▃▃▂▂▁▁▁▁▁

@neplex/vectorizer encoded async 617.69 µs/iter 650.90 µs  █▆
                        (572.30 µs … 785.10 µs) 728.20 µs  ██▄▂
                        (  2.27 kb …  76.16 kb)   2.35 kb ▅████▆▅▄▄▅▄▅██▅▃▂▁▂▁▂

@neplex/vectorizer raw async     583.75 µs/iter 593.00 µs   █▃
                        (551.50 µs … 721.00 µs) 677.90 µs  ████▄
                        (  2.33 kb …  76.23 kb)   2.40 kb ▅█████▇▆▆▅▃▂▂▂▅▄▃▂▁▁▁

@neplex/vectorizer raw callback  601.52 µs/iter 604.30 µs  █
                          (561.30 µs … 1.18 ms)   1.11 ms ▇█
                        (  5.02 kb … 217.06 kb)   5.48 kb ███▄▂▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁

imagetracerjs                      2.37 ms/iter   2.36 ms  █
                            (2.32 ms … 2.92 ms)   2.65 ms  █▇
                        (  1.17 mb …   6.89 mb)   4.52 mb ▆███▄▂▂▁▁▁▂▂▂▂▂▁▁▁▁▁▁

potrace trace                      3.83 ms/iter   4.72 ms     ▃█  ▅▇
                            (1.82 ms … 7.31 ms)   6.73 ms   ▆ ██  ██▂ ▇█
                        (175.78 kb …  12.23 mb)   2.34 mb ▄▇████▇▄███▄██▇▅▄▇▂▁▂

summary
  @neplex/vectorizer raw sync
   1.04x faster than @neplex/vectorizer encoded sync
   1.1x faster than @neplex/vectorizer raw async
   1.13x faster than @neplex/vectorizer raw callback
   1.16x faster than @neplex/vectorizer encoded async
   4.45x faster than imagetracerjs
   7.19x faster than potrace trace
```

See [benchmark](https://github.com/neplextech/vectorizer/blob/main/benchmark/bench.ts) for more details.

## API

### `vectorize(data: Buffer, config?: Config | Preset): Promise<string>`

Takes an image buffer and returns a promise that resolves to an SVG string.

### `vectorizeSync(data: Buffer, config?: Config | Preset): string`

Takes an image buffer and returns an SVG string synchronously.

### `vectorizeRaw(data: Buffer, args: RawDataConfig, config?: Config | Preset): Promise<string>`

Takes a raw pixel data buffer and returns a promise that resolves to an SVG string.

### `vectorizeRawSync(data: Buffer, args: RawDataConfig, config?: Config | Preset): string`

Takes a raw pixel data buffer and returns an SVG string synchronously.

### `vectorizeToCallback(data: Buffer, config: Config | Preset | null | undefined, callback: ([chunk, progress]) => void): Promise<void>`

Vectorizes an image buffer and emits SVG chunks to the callback. `progress` is a number from 0–100 indicating how much of the vectorization has been written.

### `vectorizeRawToCallback(data: Buffer, args: RawDataConfig, config: Config | Preset | null | undefined, callback: ([chunk, progress]) => void): Promise<void>`

Vectorizes raw pixel data and emits SVG chunks to the callback. `progress` is a number from 0–100.

### `isEOF(data: [string, number]): boolean`

Determines if the given chunk data indicates the end of the SVG output. Useful for callback-based vectorization to know when the final chunk has been received. Note that EOF here refers to the final chunk of svg, which is `</svg>\n`.

```js
await vectorizeToCallback(src, config, (chunk) => {
  if (isEOF(chunk)) {
    console.log('SVG output complete!');
  }
});
```

## SVG Optimization

The raw output from `vectorize` can be significantly reduced in size using optimizers like [`@oxvg/napi`](https://www.npmjs.com/package/@oxvg/napi):

```js
import { vectorize } from '@neplex/vectorizer';
import { optimise } from '@oxvg/napi';
import { readFile, writeFile } from 'node:fs/promises';

const src = await readFile('./input.png');
const svg = await vectorize(src);
const optimized = optimise(svg);

await writeFile('./output.svg', optimized);
```

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

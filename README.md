# @neplex/vectorizer

Node.js library to convert raster images to svg using [VTracer](https://github.com/visioncortex/vtracer), with time complexity of `O(n)`.

## Benchmark

```js
clk: ~5.11 GHz
cpu: Intel(R) Core(TM) i7-14700K
runtime: node 22.12.0 (x64-win32)

benchmark                   avg (min … max) p75   p99    (min … top 1%)
------------------------------------------- -------------------------------
@neplex/vectorizer           543.89 µs/iter 542.50 µs  ▆█
                    (517.20 µs … 778.50 µs) 719.00 µs ▃██▅▂▂▂▂▁▁▁▁▁▁▁▁▁▁▁▁▁
imagetracerjs                  2.54 ms/iter   2.61 ms  ▃█▃▂▄
                        (2.38 ms … 4.09 ms)   2.93 ms ██████████▄▅▄▅▂▂▁▁▁▂▁

summary
  @neplex/vectorizer
   4.67x faster than imagetracerjs
```

See [benchmark](https://github.com/neplextech/vectorizer/blob/main/benchmark/bench.ts) for more details.

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

## API

### `vectorize(data: Buffer, config?: Config | Preset): Promise<string>`

Takes an image buffer and returns a promise that resolves to an SVG string.

### `vectorizeSync(data: Buffer, config?: Config | Preset): string`

Takes an image buffer and returns an SVG string synchronously.

### `vectorizeRaw(data: Buffer, args: RawDataConfig, config?: Config | Preset): Promise<string>`

Takes a raw pixel data buffer and returns a promise that resolves to an SVG string.

### `vectorizeRawSync(data: Buffer, args: RawDataConfig, config?: Config | Preset): string`

Takes a raw pixel data buffer and returns an SVG string synchronously.

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

| Raster Image (PNG Input)                                                                                                                                                                                                         | Vector Image (Generated SVG)                                                                                                                                                                                                 |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ![Raster Image](https://raw.githubusercontent.com/neplextech/vectorizer/main/example/anime-girl.png)<br/>[CC-BY-SA 3.0](https://creativecommons.org/licenses/by/3.0) by [Niabot](https://commons.wikimedia.org/wiki/User:Niabot) | ![Vector Image](https://raw.githubusercontent.com/neplextech/vectorizer/main/example/result.svg)<br/>[CC-BY-SA 3.0](https://creativecommons.org/licenses/by/3.0) by [Niabot](https://commons.wikimedia.org/wiki/User:Niabot) |

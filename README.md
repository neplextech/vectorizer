# @neplex/vectorizer

A simple Node.js library to convert raster images into svg using [VTracer](https://github.com/visioncortex/vtracer), with time complexity of `O(n)`.

## Installation

```bash
npm install @neplex/vectorizer
```

## Usage

```js
import {
  vectorize,
  ColorMode,
  Hierarchial,
  PathSimplifyMode,
} from '@neplex/vectorizer';
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

If you want to use synchronous API, you can use `vectorizeSync` instead.

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

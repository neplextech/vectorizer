# Vectrace

A Node.js library to convert raster images to vector graphics using [VTracer](https://github.com/visioncortex/vtracer).

## Installation

```bash
npm install vectrace
```

## Usage

```js
import { vectorize, ColorMode, Hierarchial, PathSimplifyMode } from 'vectrace';
import { Transformer } from '@napi-rs/image';
import { readFile, writeFile } from 'node:fs/promises';

const src = await readFile('./raster.png');
const pixels = await new Transformer(src).rawPixels();

const svg = await vectorize(pixels, {
  width: IMAGE_WIDTH,
  height: IMAGE_HEIGHT,
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
    maxIterations: 2,
    width: 1052,
    height: 774
}
```

| Raster Image                        | Vector Image                        |
| ----------------------------------- | ----------------------------------- |
| ![Raster Image](example/raster.png) | ![Vector Image](example/vector.svg) |

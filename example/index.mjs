import { Transformer } from '@napi-rs/image'
import { vectorize, ColorMode, Hierarchical, PathSimplifyMode } from '../index.js'
import { readFile, writeFile } from 'node:fs/promises'

const WIDTH = 1052, HEIGHT = 744;
const src = await readFile('./example/anime-girl.png');
const pixels = await new Transformer(src).rawPixels();

const config = {
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
    width: WIDTH,
    height: HEIGHT
};

const result = await vectorize(pixels, config);

await writeFile('./example/result.svg', result);
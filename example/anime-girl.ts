import { vectorize, ColorMode, Hierarchical, PathSimplifyMode } from '../index.js';
import { readFile, writeFile } from 'node:fs/promises';

const src = await readFile('./example/anime-girl.png');

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
};

const begin = performance.now();
const result = await vectorize(src, config);
const end = performance.now();

console.log(`[Anime Girl Vectorization] Time: ${(end - begin).toFixed(2)}ms`);

await writeFile('./example/result.svg', result);

import test from 'ava'
import { writeFile, readFile } from 'node:fs/promises'
import { Transformer } from '@napi-rs/image'
import { ColorMode, vectorize, PathSimplifyMode, Hierarchical } from '../index.js'

const config = {
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
};

const configCircle = { ...config, width: 100, height: 100 };
const configFirefox = {
  ...config,
  filterSpeckle: 14,
  colorPrecision: 8,
  mode: PathSimplifyMode.Polygon,
  width: 432,
  height: 420,
  layerDifference: 0
};

test('should vectorize image (simple)', async (t) => {
  const src = await readFile('./__test__/data/sample.png');
  const pixels = new Transformer(src);
  const result = await vectorize(await pixels.rawPixels(), configCircle);

  await writeFile('./__test__/data/result.svg', result);

  t.pass();
})

test('should vectorize image (hard)', async (t) => {
  const src = await readFile('./__test__/data/firefox-logo.png');
  const pixels = new Transformer(src);
  const result = await vectorize(await pixels.rawPixels(), configFirefox);

  await writeFile('./__test__/data/result-firefox.svg', result);

  t.pass();
})
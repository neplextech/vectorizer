import test from 'ava';
import { writeFile, readFile } from 'node:fs/promises';
import { ColorMode, vectorize, PathSimplifyMode, Hierarchical, Preset, vectorizeRaw } from '../index.js';
import { Transformer } from '@napi-rs/image';

const src = await readFile('./__test__/data/firefox-logo.png');
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
  layerDifference: 0,
};

test('should vectorize image (simple)', async (t) => {
  const src = await readFile('./__test__/data/sample.png');
  const result = await vectorize(src, configCircle);

  await writeFile('./__test__/data/result.svg', result);

  t.pass();
});

test('should vectorize raw pixels data', async (t) => {
  const src = await readFile('./__test__/data/sample.png');
  const raw = await new Transformer(src).rawPixels();
  const result = await vectorizeRaw(
    raw,
    {
      height: 100,
      width: 100,
    },
    configCircle,
  );

  await writeFile('./__test__/data/result-raw.svg', result);

  t.pass();
});

test('should vectorize image', async (t) => {
  const result = await vectorize(src, configFirefox);

  await writeFile('./__test__/data/result-firefox.svg', result);

  t.pass();
});

test('should vectorize image with preset bw', async (t) => {
  const result = await vectorize(src, Preset.Bw);

  await writeFile('./__test__/data/result-bw.svg', result);

  t.pass();
});

test('should vectorize image with preset Photo', async (t) => {
  const result = await vectorize(src, Preset.Photo);

  await writeFile('./__test__/data/result-photo.svg', result);

  t.pass();
});

test('should vectorize image with preset Poster', async (t) => {
  const result = await vectorize(src, Preset.Poster);

  await writeFile('./__test__/data/result-poster.svg', result);

  t.pass();
});

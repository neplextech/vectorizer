import test from 'ava';
import {
  colorExistsInImage,
  findUnusedColorInImage,
  readImage,
  SvgFile,
  vectorizeRawToCallback,
  vectorizeRawSync,
  vectorizeSync,
  PathSimplifyMode,
  ColorMode,
  Hierarchical,
  Preset,
} from '../index';

const redPixel = Buffer.from([255, 0, 0, 255]);
const rawArgs = { width: 1, height: 1 };
const config = {
  colorMode: ColorMode.Color,
  hierarchical: Hierarchical.Stacked,
  filterSpeckle: 0,
  colorPrecision: 8,
  layerDifference: 16,
  mode: PathSimplifyMode.Spline,
  cornerThreshold: 60,
  lengthThreshold: 4,
  maxIterations: 2,
  spliceThreshold: 45,
  pathPrecision: 2,
  unusedColorIterations: 1,
  keyingThreshold: 0.5,
  smallCircle: 12,
};

test('readImage exposes decoded color image data', (t) => {
  const image = readImage(redPixel, rawArgs);

  t.is(image.width, 1);
  t.is(image.height, 1);
  t.deepEqual([...image.pixels], [...redPixel]);
});

test('colorExistsInImage and findUnusedColorInImage expose image color helpers', (t) => {
  const image = readImage(redPixel, rawArgs);

  t.true(colorExistsInImage(image, { r: 255, g: 0, b: 0, a: 255 }));
  t.false(colorExistsInImage(image, { r: 0, g: 255, b: 0, a: 255 }));
  t.deepEqual(findUnusedColorInImage(image, { unusedColorIterations: 0 }), { r: 0, g: 255, b: 0, a: 255 });
});

test('SvgFile can be constructed and stringified from js', (t) => {
  const svg = new SvgFile(10, 20, 2);

  t.is(svg.width, 10);
  t.is(svg.height, 20);
  t.true(svg.toString().includes('<svg'));
});

test('vectorizeRawToCallback emits svg chunks with progress', async (t) => {
  const chunks: string[] = [];
  const progressValues: number[] = [];

  await vectorizeRawToCallback(redPixel, rawArgs, config, ([chunk, progress]) => {
    chunks.push(chunk);
    progressValues.push(progress);
  });

  const svg = chunks.join('');
  t.true(chunks.length > 1);
  t.is(svg, vectorizeRawSync(redPixel, rawArgs, config));
  t.true(progressValues.length > 0);
  t.is(progressValues[progressValues.length - 1], 100);
  // progress is non-decreasing
  for (let i = 1; i < progressValues.length; i++) {
    t.true(progressValues[i] >= progressValues[i - 1]);
  }
});

test('vectorizeSync rejects corrupt encoded images with a readable error', (t) => {
  const error = t.throws(() => vectorizeSync(Buffer.from('not an image')));

  t.regex(error?.message ?? '', /unable to read this image/);
});

test('vectorizeRawSync rejects zero-dimension raw input', (t) => {
  const error = t.throws(() => vectorizeRawSync(Buffer.alloc(0), { width: 0, height: 1 }, config));

  t.regex(error?.message ?? '', /width and height must be positive/);
});

test('vectorizeRawSync rejects raw input with mismatched pixel length', (t) => {
  const error = t.throws(() => vectorizeRawSync(Buffer.from([255, 0, 0]), rawArgs, config));

  t.regex(error?.message ?? '', /pixel data length/i);
});

for (const [name, preset] of [
  ['Bw', Preset.Bw],
  ['Poster', Preset.Poster],
  ['Photo', Preset.Photo],
] as const) {
  test(`vectorizeRawSync handles preset ${name}`, (t) => {
    const svg = vectorizeRawSync(redPixel, rawArgs, preset);

    t.true(svg.includes('<svg'));
  });
}

test('vectorizeRawSync handles Cutout hierarchical mode', (t) => {
  const svg = vectorizeRawSync(redPixel, rawArgs, {
    ...config,
    hierarchical: Hierarchical.Cutout,
  });

  t.true(svg.includes('<svg'));
});

test('findUnusedColorInImage throws when reserved colors are exhausted and random search is disabled', (t) => {
  const pixels = Buffer.from([
    255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255, 0, 255, 255, 255, 255, 0, 255, 255,
  ]);
  const image = readImage(pixels, { width: 6, height: 1 });

  const error = t.throws(() => findUnusedColorInImage(image, { unusedColorIterations: 0 }));

  t.regex(error?.message ?? '', /unable to find unused color/);
});

test('transparent keyed conversion surfaces no-unused-color failures', (t) => {
  const pixels = Buffer.from([
    255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255, 0, 255, 255, 255, 255, 0, 255, 255, 1, 2, 3, 0,
  ]);

  const error = t.throws(() =>
    vectorizeRawSync(pixels, { width: 7, height: 1 }, { ...config, unusedColorIterations: 0, keyingThreshold: 0.01 }),
  );

  t.regex(error?.message ?? '', /unable to find unused color/);
});

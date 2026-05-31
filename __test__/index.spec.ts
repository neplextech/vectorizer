import test from 'ava';
import {
  colorExistsInImage,
  colorExistsInImageSync,
  findUnusedColorInImage,
  findUnusedColorInImageSync,
  readImage,
  readImageSync,
  SvgFile,
  vectorizeRawToCallback,
  vectorizeRawSync,
  vectorizeSync,
  PathSimplifyMode,
  ColorMode,
  Hierarchical,
  Preset,
  optimize,
  optimizeSync,
  OptimizePreset,
  isEOF,
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

test('readImage resolves decoded color image data', async (t) => {
  const imagePromise = readImage(redPixel, rawArgs);

  t.true(imagePromise instanceof Promise);

  const image = await imagePromise;

  t.is(image.width, 1);
  t.is(image.height, 1);
  t.deepEqual([...image.pixels], [...redPixel]);
});

test('image sync helpers expose decoded image and color operations', (t) => {
  const image = readImageSync(redPixel, rawArgs);

  t.is(image.width, 1);
  t.is(image.height, 1);
  t.deepEqual([...image.pixels], [...redPixel]);
  t.true(colorExistsInImageSync(image, { r: 255, g: 0, b: 0, a: 255 }));
  t.false(colorExistsInImageSync(image, { r: 0, g: 255, b: 0, a: 255 }));
  t.deepEqual(findUnusedColorInImageSync(image, { unusedColorIterations: 0 }), { r: 0, g: 255, b: 0, a: 255 });
});

test('colorExistsInImage and findUnusedColorInImage resolve image color helper results', async (t) => {
  const image = await readImage(redPixel, rawArgs);

  const existsPromise = colorExistsInImage(image, { r: 255, g: 0, b: 0, a: 255 });
  const unusedColorPromise = findUnusedColorInImage(image, { unusedColorIterations: 0 });

  t.true(existsPromise instanceof Promise);
  t.true(unusedColorPromise instanceof Promise);

  t.true(await existsPromise);
  t.false(await colorExistsInImage(image, { r: 0, g: 255, b: 0, a: 255 }));
  t.deepEqual(await unusedColorPromise, { r: 0, g: 255, b: 0, a: 255 });
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

  const { promise, resolve } = Promise.withResolvers<void>();

  vectorizeRawToCallback(redPixel, rawArgs, config, (chunk, progress) => {
    chunks.push(chunk);
    progressValues.push(progress);

    if (isEOF(chunk, progress)) {
      resolve();
    }
  });

  await promise;

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
  const image = readImageSync(pixels, { width: 6, height: 1 });

  const error = t.throws(() => findUnusedColorInImageSync(image, { unusedColorIterations: 0 }));

  t.regex(error?.message ?? '', /unable to find unused color/);
});

test('optimizeSync optimizes svg strings', (t) => {
  const svg = '<svg xmlns="http://www.w3.org/2000/svg"><!-- remove me --><g><path d="M0 0"/></g></svg>';

  const optimized = optimizeSync(svg);

  t.true(optimized.includes('<svg'));
  t.false(optimized.includes('remove me'));
});

test('optimize resolves async optimized svg strings', async (t) => {
  const svg = '<svg xmlns="http://www.w3.org/2000/svg"><!-- remove me --><g><path d="M0 0"/></g></svg>';

  const optimized = await optimize(svg);

  t.is(optimized, optimizeSync(svg));
});

test('optimize accepts preset and omitted job options', (t) => {
  const svg = '<svg xmlns="http://www.w3.org/2000/svg"><!-- keep me --><g><path d="M0 0"/></g></svg>';

  const optimized = optimizeSync(svg, {
    preset: OptimizePreset.Safe,
    omit: ['remove_comments'],
  });

  t.true(optimized.includes('keep me'));
});

test('optimize accepts SVGO plugin string config', (t) => {
  const svg = '<svg xmlns="http://www.w3.org/2000/svg"><!-- remove me --><g><path d="M0 0"/></g></svg>';

  const optimized = optimizeSync(svg, {
    preset: OptimizePreset.None,
    plugins: ['removeComments'],
  });

  t.false(optimized.includes('remove me'));
});

test('optimize accepts SVGO plugin object config', (t) => {
  const svg = '<svg xmlns="http://www.w3.org/2000/svg"><title>remove me</title><g><path d="M0 0"/></g></svg>';

  const optimized = optimizeSync(svg, {
    preset: OptimizePreset.None,
    plugins: [{ name: 'removeTitle' }],
  });

  t.false(optimized.includes('remove me'));
});

test('optimize rejects unsupported SVGO plugin config', (t) => {
  const error = t.throws(() =>
    optimizeSync('<svg xmlns="http://www.w3.org/2000/svg"/>', {
      plugins: ['customPlugin'],
    }),
  );

  t.regex(error?.message ?? '', /unknown job/i);
});

test('optimize rejects invalid multipass iteration counts', (t) => {
  const error = t.throws(() =>
    optimizeSync('<svg xmlns="http://www.w3.org/2000/svg"/>', {
      multipass: true,
      multipassIterations: 0,
    }),
  );

  t.regex(error?.message ?? '', /multipassIterations/i);
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

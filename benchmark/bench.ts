import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { run, bench, summary } from 'mitata';
import { vectorizeRawSync } from '../index.js';
// @ts-ignore
import ImageTracer from 'imagetracerjs';
import { Transformer } from '@napi-rs/image';

const data = await readFile(join(import.meta.dirname, 'data', 'sample.png'));
const image = await new Transformer(data).rawPixels();
const imageData = {
  width: 100,
  height: 100,
  data: image,
};

const { data: pixels, ...size } = imageData;

summary(() => {
  bench('@neplex/vectorizer', () => {
    vectorizeRawSync(pixels, size);
  });

  bench('imagetracerjs', () => {
    ImageTracer.imagedataToSVG(imageData);
  });
});

await run();

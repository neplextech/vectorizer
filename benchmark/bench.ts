import { createRequire } from 'node:module';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { run, bench, summary } from 'mitata';
import {
  isEOF,
  vectorize,
  vectorizeRaw,
  vectorizeRawSync,
  vectorizeRawToCallback,
  vectorizeSync,
  vectorizeToCallback,
} from '../index.js';
// @ts-ignore
import ImageTracer from 'imagetracerjs';
import { Transformer } from '@napi-rs/image';

type PotraceModule = {
  trace(input: Buffer | string, callback: (error: Error | null, svg: string) => void): void;
  posterize(
    input: Buffer | string,
    options: { steps: number },
    callback: (error: Error | null, svg: string) => void,
  ): void;
};

const require = createRequire(import.meta.url);
const samplePath = join(import.meta.dirname, 'data', 'sample.png');
const data = await readFile(samplePath);
const image = await new Transformer(data).rawPixels();
const imageData = {
  width: 100,
  height: 100,
  data: image,
};

const { data: pixels, ...size } = imageData;
const optionalPotrace = loadOptionalPotrace();

function loadOptionalPotrace(): PotraceModule | null {
  try {
    return require('potrace') as PotraceModule;
  } catch {
    return null;
  }
}

function traceWithPotrace(potrace: PotraceModule, input: Buffer): Promise<string> {
  return new Promise((resolve, reject) => {
    potrace.trace(input, (error, svg) => {
      if (error) {
        reject(error);
        return;
      }
      resolve(svg);
    });
  });
}

summary(() => {
  bench('@neplex/vectorizer raw sync', () => {
    vectorizeRawSync(pixels, size);
  });

  bench('@neplex/vectorizer encoded sync', () => {
    vectorizeSync(data);
  });

  bench('@neplex/vectorizer encoded async', async () => {
    await vectorize(data);
  });

  bench('@neplex/vectorizer encoded callback', async () => {
    const chunks: string[] = [];
    const { promise, resolve } = Promise.withResolvers<void>();

    vectorizeToCallback(data, null, (chunk, progress) => {
      chunks.push(chunk);

      if (isEOF(chunk, progress)) {
        resolve();
      }
    });

    await promise;

    chunks.join('');
  });

  bench('@neplex/vectorizer raw async', async () => {
    await vectorizeRaw(pixels, size);
  });

  bench('@neplex/vectorizer raw callback', async () => {
    const chunks: string[] = [];
    const { promise, resolve } = Promise.withResolvers<void>();

    vectorizeRawToCallback(pixels, size, null, (chunk, progress) => {
      chunks.push(chunk);

      if (isEOF(chunk, progress)) {
        resolve();
      }
    });

    await promise;

    chunks.join('');
  });

  bench('imagetracerjs', () => {
    ImageTracer.imagedataToSVG(imageData);
  });

  if (optionalPotrace) {
    bench('potrace trace', async () => {
      await traceWithPotrace(optionalPotrace, data);
    });
  } else {
    console.log('Skipping optional potrace benchmark. Install it with `yarn add -D potrace` to include it.');
  }
});

await run();

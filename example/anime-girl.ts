import { ColorMode, Hierarchical, PathSimplifyMode, vectorizeToCallback, isEOF, optimize } from '../index.js';
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

const results: string[] = [];
const begin = performance.now();
const { promise, resolve } = Promise.withResolvers<void>();
const progressBarWidth = 24;
let chunkCount = 0;

function renderProgressBar(progress: number, chunksPerMilliSecond: number) {
  const filledLength = Math.round((progress / 100) * progressBarWidth);
  const emptyLength = progressBarWidth - filledLength;
  const bar = `${'#'.repeat(filledLength)}${'-'.repeat(emptyLength)}`;

  process.stdout.write(
    `\rProgress: [${bar}] ${progress.toFixed(2)}% | Speed: ${chunksPerMilliSecond.toFixed(2)} chunks/ms`,
  );
}

vectorizeToCallback(src, config, (chunk, progress) => {
  chunkCount += 1;
  const elapsedMilliSeconds = Math.max(performance.now() - begin, 0.001);
  const chunksPerMilliSecond = chunkCount / elapsedMilliSeconds;

  renderProgressBar(progress, chunksPerMilliSecond);
  results.push(chunk);
  if (isEOF(chunk, progress)) {
    process.stdout.write('\n');
    resolve();
  }
});
await promise;
console.log(`Total chunks: ${chunkCount}`);
const end = performance.now();
const result = results.join('');

console.log(`[Anime Girl Vectorization] Time: ${(end - begin).toFixed(2)}ms | Length: ${result.length}`);

await writeFile('./example/result.svg', result);

const optimizeBegin = performance.now();
const optimized = await optimize(result);
const optimizeEnd = performance.now();

console.log(
  `[Anime Girl Optimization] Time: ${(optimizeEnd - optimizeBegin).toFixed(2)}ms | Length: ${optimized.length}`,
);

await writeFile('./example/result-optimized.svg', optimized);

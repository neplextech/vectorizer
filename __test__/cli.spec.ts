import test from 'ava';
import { execFile } from 'node:child_process';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { promisify } from 'node:util';

const execFileAsync = promisify(execFile);
const cliPath = resolve('cli/index.mjs');
const redPixel = Buffer.from([255, 0, 0, 255]);

test('cli vectorizes with on-the-fly optimization', async (t) => {
  const dir = await mkdtemp(join(tmpdir(), 'vectorizer-cli-'));
  t.teardown(() => rm(dir, { recursive: true, force: true }));
  const inputPath = join(dir, 'pixel.rgba');
  const outputPath = join(dir, 'pixel.svg');

  await writeFile(inputPath, redPixel);

  const { stderr } = await execFileAsync(process.execPath, [
    cliPath,
    inputPath,
    outputPath,
    '--raw',
    '1x1',
    '--preset',
    'poster',
    '--optimize',
    '--optimize-plugin',
    'removeComments',
  ]);

  const svg = await readFile(outputPath, 'utf8');
  t.true(svg.includes('<svg'));
  t.regex(stderr, /Progress:/);
  t.regex(stderr, /Optimized/);
});

test('cli optimize command optimizes an existing svg file', async (t) => {
  const dir = await mkdtemp(join(tmpdir(), 'vectorizer-cli-'));
  t.teardown(() => rm(dir, { recursive: true, force: true }));
  const inputPath = join(dir, 'input.svg');
  const outputPath = join(dir, 'output.svg');

  await writeFile(inputPath, '<svg xmlns="http://www.w3.org/2000/svg"><!-- remove me --><g><path d="M0 0"/></g></svg>');

  const { stderr } = await execFileAsync(process.execPath, [
    cliPath,
    'optimize',
    inputPath,
    outputPath,
    '--plugin',
    'removeComments',
  ]);

  const svg = await readFile(outputPath, 'utf8');
  t.true(svg.includes('<svg'));
  t.false(svg.includes('remove me'));
  t.regex(stderr, /Optimized/);
});

test('cli print streams svg chunks without a progress bar', async (t) => {
  const dir = await mkdtemp(join(tmpdir(), 'vectorizer-cli-'));
  t.teardown(() => rm(dir, { recursive: true, force: true }));
  const inputPath = join(dir, 'pixel.rgba');

  await writeFile(inputPath, redPixel);

  const { stdout, stderr } = await execFileAsync(process.execPath, [
    cliPath,
    inputPath,
    '--raw',
    '1x1',
    '--preset',
    'poster',
    '--print',
  ]);

  t.true(stdout.includes('<svg'));
  t.false(stderr.includes('Progress:'));
});

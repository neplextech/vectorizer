import {
  createOnMessage as __wasmCreateOnMessageForFsProxy,
  getDefaultContext as __emnapiGetDefaultContext,
  instantiateNapiModuleSync as __emnapiInstantiateNapiModuleSync,
  WASI as __WASI,
} from '@napi-rs/wasm-runtime';

const __wasi = new __WASI({
  version: 'preview1',
});

const __wasmUrl = new URL('./vectorizer.wasm32-wasi.wasm', import.meta.url).href;
const __emnapiContext = __emnapiGetDefaultContext();

const __sharedMemory = new WebAssembly.Memory({
  initial: 4000,
  maximum: 65536,
  shared: true,
});

const __wasmFile = await fetch(__wasmUrl).then((res) => res.arrayBuffer());

const {
  instance: __napiInstance,
  module: __wasiModule,
  napiModule: __napiModule,
} = __emnapiInstantiateNapiModuleSync(__wasmFile, {
  context: __emnapiContext,
  asyncWorkPoolSize: 4,
  wasi: __wasi,
  onCreateWorker() {
    const worker = new Worker(new URL('./wasi-worker-browser.mjs', import.meta.url), {
      type: 'module',
    });

    return worker;
  },
  overwriteImports(importObject) {
    importObject.env = {
      ...importObject.env,
      ...importObject.napi,
      ...importObject.emnapi,
      memory: __sharedMemory,
    };
    return importObject;
  },
  beforeInit({ instance }) {
    for (const name of Object.keys(instance.exports)) {
      if (name.startsWith('__napi_register__')) {
        instance.exports[name]();
      }
    }
  },
});
export default __napiModule.exports;
export const SvgFile = __napiModule.exports.SvgFile;
export const JsSvgFile = __napiModule.exports.JsSvgFile;
export const colorExistsInImage = __napiModule.exports.colorExistsInImage;
export const colorExistsInImageSync = __napiModule.exports.colorExistsInImageSync;
export const ColorMode = __napiModule.exports.ColorMode;
export const findUnusedColorInImage = __napiModule.exports.findUnusedColorInImage;
export const findUnusedColorInImageSync = __napiModule.exports.findUnusedColorInImageSync;
export const Hierarchical = __napiModule.exports.Hierarchical;
export const optimize = __napiModule.exports.optimize;
export const OptimizePreset = __napiModule.exports.OptimizePreset;
export const JsOptimizePreset = __napiModule.exports.JsOptimizePreset;
export const optimizeSync = __napiModule.exports.optimizeSync;
export const PathSimplifyMode = __napiModule.exports.PathSimplifyMode;
export const JsPathSimplifyMode = __napiModule.exports.JsPathSimplifyMode;
export const Preset = __napiModule.exports.Preset;
export const readImage = __napiModule.exports.readImage;
export const readImageSync = __napiModule.exports.readImageSync;
export const vectorize = __napiModule.exports.vectorize;
export const vectorizeRaw = __napiModule.exports.vectorizeRaw;
export const vectorizeRawSync = __napiModule.exports.vectorizeRawSync;
export const vectorizeRawToCallback = __napiModule.exports.vectorizeRawToCallback;
export const vectorizeSync = __napiModule.exports.vectorizeSync;
export const vectorizeToCallback = __napiModule.exports.vectorizeToCallback;

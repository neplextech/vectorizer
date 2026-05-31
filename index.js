const nativeBinding = require('./js-bindings')
module.exports = nativeBinding

module.exports.isEOF = function isEOF(chunk, progress) {
  if (typeof chunk !== 'string' || typeof progress !== 'number') {
    return false;
  }

  return progress >= 100 && chunk === '</svg>\n';
}

// Auto-generated exports by postbuild.js. Do not edit directly.
module.exports.SvgFile = nativeBinding.SvgFile
module.exports.JsSvgFile = nativeBinding.JsSvgFile
module.exports.colorExistsInImage = nativeBinding.colorExistsInImage
module.exports.colorExistsInImageSync = nativeBinding.colorExistsInImageSync
module.exports.ColorMode = nativeBinding.ColorMode
module.exports.findUnusedColorInImage = nativeBinding.findUnusedColorInImage
module.exports.findUnusedColorInImageSync = nativeBinding.findUnusedColorInImageSync
module.exports.Hierarchical = nativeBinding.Hierarchical
module.exports.optimize = nativeBinding.optimize
module.exports.OptimizePreset = nativeBinding.OptimizePreset
module.exports.JsOptimizePreset = nativeBinding.JsOptimizePreset
module.exports.optimizeSync = nativeBinding.optimizeSync
module.exports.PathSimplifyMode = nativeBinding.PathSimplifyMode
module.exports.JsPathSimplifyMode = nativeBinding.JsPathSimplifyMode
module.exports.Preset = nativeBinding.Preset
module.exports.readImage = nativeBinding.readImage
module.exports.readImageSync = nativeBinding.readImageSync
module.exports.vectorize = nativeBinding.vectorize
module.exports.vectorizeRaw = nativeBinding.vectorizeRaw
module.exports.vectorizeRawSync = nativeBinding.vectorizeRawSync
module.exports.vectorizeRawToCallback = nativeBinding.vectorizeRawToCallback
module.exports.vectorizeSync = nativeBinding.vectorizeSync
module.exports.vectorizeToCallback = nativeBinding.vectorizeToCallback

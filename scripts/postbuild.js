const { readFileSync, writeFileSync } = require('node:fs');

const EXPORT_MARKER = '// Auto-generated exports by postbuild.js. Do not edit directly.';

function collectNativeExportNames(jsBindings) {
  const names = [];
  const seen = new Set();
  const matches = jsBindings.matchAll(/module\.exports\.(\w+)/g);

  for (const match of matches) {
    const name = match[1];
    if (!seen.has(name)) {
      seen.add(name);
      names.push(name);
    }
  }

  return names;
}

function buildIndexJs(nativeExportNames) {
  const originalSource = readFileSync('./index.js', 'utf-8');
  const exportStatements = nativeExportNames.map((name) => `module.exports.${name} = nativeBinding.${name}`).join('\n');

  return `${originalSource.split(EXPORT_MARKER)[0].trimEnd()}\n
${EXPORT_MARKER}
${exportStatements}
`;
}

function postbuild() {
  const jsBindings = readFileSync('./js-bindings.js', 'utf-8');
  const nativeExportNames = collectNativeExportNames(jsBindings);

  writeFileSync('./index.js', buildIndexJs(nativeExportNames));
}

if (require.main === module) {
  postbuild();
}

module.exports = postbuild;

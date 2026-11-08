import { existsSync, mkdirSync, readFileSync, watch, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { deflateSync } from 'node:zlib';

const __dirname = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(__dirname, '../..');

/**
 * 1. State & Method Extractor (SRP): Extracts Angular `signal(...)`, fields, and signal updater methods from `app.ts`.
 */
function extractComponentMetadata(tsSource) {
  const state = {};
  const numericSignals = {};
  const methods = {};

  const signalRegex =
    /(?:protected|public|private|readonly|\s)+([a-zA-Z_]\w*)\s*=\s*signal\(\s*(?:'([^']*)'|"([^"]*)"|(-?\d+))\s*\)/g;
  let match;
  while ((match = signalRegex.exec(tsSource)) !== null) {
    const [, key, singleQuoted, doubleQuoted, numeric] = match;
    state[key] = singleQuoted ?? doubleQuoted ?? numeric;
    if (numeric !== undefined) {
      numericSignals[key] = Number(numeric);
    }
  }

  const propRegex = /^\s*([a-zA-Z_]\w*)\s*=\s*(?:'([^']*)'|"([^"]*)"|(-?\d+))\s*;/gm;
  while ((match = propRegex.exec(tsSource)) !== null) {
    const [, key, singleQuoted, doubleQuoted, numeric] = match;
    if (!(key in state)) {
      state[key] = singleQuoted ?? doubleQuoted ?? numeric;
    }
  }

  const updateMethodRegex =
    /([a-zA-Z_]\w*)\s*\([^)]*\)\s*(?::\s*\w+)?\s*\{\s*this\.([a-zA-Z_]\w*)\.update\(\s*\(?[a-zA-Z_]\w*\)?\s*=>\s*[a-zA-Z_]\w*\s*([+-])\s*(\d+)\s*\)\s*;?\s*\}/g;
  while ((match = updateMethodRegex.exec(tsSource)) !== null) {
    const [, methodName, signalName, op, amountStr] = match;
    const delta = op === '-' ? -Number(amountStr) : Number(amountStr);
    methods[methodName] = { kind: 'add', signalName, value: delta };
  }

  const setMethodRegex =
    /([a-zA-Z_]\w*)\s*\([^)]*\)\s*(?::\s*\w+)?\s*\{\s*this\.([a-zA-Z_]\w*)\.set\(\s*(-?\d+)\s*\)\s*;?\s*\}/g;
  while ((match = setMethodRegex.exec(tsSource)) !== null) {
    const [, methodName, signalName, valStr] = match;
    methods[methodName] = { kind: 'set', signalName, value: Number(valStr) };
  }

  return { state, numericSignals, methods };
}

/**
 * 2. Style Parser (SRP): Parses `app.css` into intermediate Style IR.
 */
function parseStylesCss(cssSource) {
  const ruleRegex = /\.([a-zA-Z0-9_-]+)\s*\{([^}]*)\}/g;
  const rules = [];
  let match;

  while ((match = ruleRegex.exec(cssSource)) !== null) {
    const [, className, body] = match;
    const props = {
      className,
      background: 'none',
      color: 'none',
      padding: '0',
      spacing: '0',
    };

    for (const rawDecl of body.split(';')) {
      const [rawKey, rawVal] = rawDecl.split(':').map((s) => s?.trim());
      if (!rawKey || !rawVal) continue;
      if (rawKey === 'background') props.background = rawVal;
      if (rawKey === 'color') props.color = rawVal;
      if (rawKey === 'padding') props.padding = rawVal;
      if (rawKey === 'spacing') props.spacing = rawVal;
    }

    rules.push(props);
  }

  return rules;
}

/**
 * Resolves Angular `{{ signal() }}` and `{{ prop }}` expressions against component state.
 */
function interpolate(text, state) {
  return text.replace(/\{\{\s*([a-zA-Z_]\w*)(?:\(\))?\s*\}\}/g, (_, key) =>
    key in state ? String(state[key]) : '',
  );
}

/**
 * 3. Template Parser (SRP): Parses `app.html` into a hierarchical AST including reactive bindings & events.
 */
function parseTemplateHtml(html, state, numericSignals) {
  const tokenRegex = /<\/?([a-zA-Z0-9-]+)([^>]*)>|([^<]+)/g;
  const rootNodes = [];
  const stack = [];
  let autoIdCounter = 1;
  let match;

  while ((match = tokenRegex.exec(html)) !== null) {
    const [fullToken, tagName, attrString, textNode] = match;

    if (textNode !== undefined) {
      const rawTrimmed = textNode.trim();
      const cleaned = interpolate(rawTrimmed, state);
      if (cleaned && stack.length > 0) {
        const current = stack[stack.length - 1];
        current.text = cleaned;

        const exprMatch = rawTrimmed.match(/\{\{\s*([a-zA-Z_]\w*)(?:\(\))?\s*\}\}/);
        if (exprMatch && exprMatch[1] in numericSignals) {
          current.reactiveBinding = {
            signalName: exprMatch[1],
            template: rawTrimmed.replace(/\{\{\s*([a-zA-Z_]\w*)(?:\(\))?\s*\}\}/, '{}'),
          };
        }
      }
      continue;
    }

    const isClosing = fullToken.startsWith('</');
    const isSelfClosing = fullToken.endsWith('/>');

    if (isClosing) {
      stack.pop();
      continue;
    }

    const attrs = {};
    const events = {};
    const attrRegex = /(\(?[a-zA-Z0-9_-]+\)?)\s*=\s*"([^"]*)"/g;
    let attrMatch;
    while ((attrMatch = attrRegex.exec(attrString)) !== null) {
      const [, rawName, rawVal] = attrMatch;
      if (rawName.startsWith('(') && rawName.endsWith(')')) {
        const evtName = rawName.slice(1, -1);
        const handlerCall = rawVal.replace(/\(\)\s*$/, '').trim();
        events[evtName] = handlerCall;
      } else {
        attrs[rawName] = interpolate(rawVal, state);
      }
    }

    const node = {
      id: attrs.id || `node_${autoIdCounter++}`,
      tag: tagName,
      title: attrs.title || null,
      classes: attrs.class ? attrs.class.split(/\s+/).filter(Boolean) : [],
      text: null,
      reactiveBinding: null,
      events,
      children: [],
    };

    if (stack.length === 0) {
      rootNodes.push(node);
    } else {
      stack[stack.length - 1].children.push(node);
    }

    if (!isSelfClosing) {
      stack.push(node);
    }
  }

  return rootNodes;
}

/**
 * 4. Multi-Resolution Icon Compiler (SRP):
 * Decodes 32-bit DIB frames from `favicon.ico` and emits multi-DPI PNGs (`16x16`..`256x256`)
 * for the Freedesktop `hicolor` theme and desktop taskbar integration.
 */
function crc32(buf) {
  let crc = 0xffffffff;
  for (let i = 0; i < buf.length; i++) {
    crc ^= buf[i];
    for (let j = 0; j < 8; j++) {
      crc = (crc >>> 1) ^ (crc & 1 ? 0xedb88320 : 0);
    }
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function makePngChunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const typeBuf = Buffer.from(type, 'ascii');
  const crcBuf = Buffer.alloc(4);
  crcBuf.writeUInt32BE(crc32(Buffer.concat([typeBuf, data])), 0);
  return Buffer.concat([len, typeBuf, data, crcBuf]);
}

function encodeRgbaToPng(width, height, rgba) {
  const signature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8;
  ihdr[9] = 6;

  const stride = width * 4;
  const rawScanlines = Buffer.alloc((stride + 1) * height);
  for (let y = 0; y < height; y++) {
    const rowStart = y * (stride + 1);
    rawScanlines[rowStart] = 0;
    rgba.copy(rawScanlines, rowStart + 1, y * stride, (y + 1) * stride);
  }

  const idat = deflateSync(rawScanlines);
  return Buffer.concat([
    signature,
    makePngChunk('IHDR', ihdr),
    makePngChunk('IDAT', idat),
    makePngChunk('IEND', Buffer.alloc(0)),
  ]);
}

function extractIcoFrames(icoBuf) {
  const count = icoBuf.readUInt16LE(4);
  const frames = [];
  for (let i = 0; i < count; i++) {
    const entryOff = 6 + i * 16;
    const width = icoBuf[entryOff] || 256;
    const height = icoBuf[entryOff + 1] || 256;
    const bpp = icoBuf.readUInt16LE(entryOff + 6);
    const imgOff = icoBuf.readUInt32LE(entryOff + 12);

    if (bpp !== 32) continue;
    const headerSize = icoBuf.readUInt32LE(imgOff);
    const pixelStart = imgOff + headerSize;
    const rgba = Buffer.alloc(width * height * 4);

    for (let y = 0; y < height; y++) {
      const srcRow = height - 1 - y;
      for (let x = 0; x < width; x++) {
        const srcIdx = pixelStart + (srcRow * width + x) * 4;
        const dstIdx = (y * width + x) * 4;
        rgba[dstIdx] = icoBuf[srcIdx + 2];
        rgba[dstIdx + 1] = icoBuf[srcIdx + 1];
        rgba[dstIdx + 2] = icoBuf[srcIdx];
        rgba[dstIdx + 3] = icoBuf[srcIdx + 3];
      }
    }
    frames.push({ width, height, rgba });
  }
  return frames;
}

function scaleNearest(frame, targetSize) {
  if (frame.width === targetSize && frame.height === targetSize) {
    return frame.rgba;
  }
  const out = Buffer.alloc(targetSize * targetSize * 4);
  for (let y = 0; y < targetSize; y++) {
    const srcY = Math.min(frame.height - 1, Math.floor((y * frame.height) / targetSize));
    for (let x = 0; x < targetSize; x++) {
      const srcX = Math.min(frame.width - 1, Math.floor((x * frame.width) / targetSize));
      const srcIdx = (srcY * frame.width + srcX) * 4;
      const dstIdx = (y * targetSize + x) * 4;
      frame.rgba.copy(out, dstIdx, srcIdx, srcIdx + 4);
    }
  }
  return out;
}

function buildMultiResolutionIcons(icoAbsPath, appId, outDir, windowTitle) {
  const icoBuf = readFileSync(icoAbsPath);
  const frames = extractIcoFrames(icoBuf);
  if (frames.length === 0) return;

  frames.sort((a, b) => b.width - a.width);
  const targetSizes = [16, 24, 32, 48, 64, 128, 256];
  const localShare = resolve(homedir(), '.local/share');

  for (const size of targetSizes) {
    const exact = frames.find((f) => f.width === size) || frames[0];
    const scaledRgba = scaleNearest(exact, size);
    const pngBytes = encodeRgbaToPng(size, size, scaledRgba);

    const distIconDir = resolve(outDir, 'icons/hicolor', `${size}x${size}`, 'apps');
    mkdirSync(distIconDir, { recursive: true });
    writeFileSync(resolve(distIconDir, `${appId}.png`), pngBytes);

    try {
      const userIconDir = resolve(localShare, 'icons/hicolor', `${size}x${size}`, 'apps');
      mkdirSync(userIconDir, { recursive: true });
      writeFileSync(resolve(userIconDir, `${appId}.png`), pngBytes);
    } catch {
      // Ignore if running in a restricted sandbox without ~/.local/share write access
    }
  }

  try {
    const appsDir = resolve(localShare, 'applications');
    mkdirSync(appsDir, { recursive: true });
    const desktopEntry = [
      '[Desktop Entry]',
      'Type=Application',
      `Name=${windowTitle || appId}`,
      `Icon=${appId}`,
      `StartupWMClass=${appId}`,
      'NoDisplay=true',
      'Terminal=false',
      '',
    ].join('\n');
    writeFileSync(resolve(appsDir, `${appId}.desktop`), desktopEntry, 'utf8');
  } catch {
    // Ignore if ~/.local/share/applications is not writable
  }
}

/**
 * 5. Bytecode Emitter (SRP): Translates Styles IR + Signals + Icons + Template AST into `.ngb` instructions.
 */
function emitNgbBundle(styleRules, numericSignals, methods, astRoots, iconMeta, sourceLabel) {
  const lines = [`// Auto-compiled by Angular Native Template Compiler from ${sourceLabel}`];

  for (const [sigName, initialValue] of Object.entries(numericSignals)) {
    lines.push(`signal ${sigName} ${initialValue}`);
  }

  for (const rule of styleRules) {
    lines.push(
      `style ${rule.className} ${rule.background} ${rule.color} ${rule.padding} ${rule.spacing}`,
    );
  }

  const appendInstructions = [];
  const bindingInstructions = [];

  function visitNode(node) {
    lines.push(`create ${node.id} ${node.tag}`);

    if (node.title) {
      lines.push(`text ${node.id} ${node.title}`);
    } else if (node.text) {
      lines.push(`text ${node.id} ${node.text}`);
    }

    if ((node.tag === 'ng-window' || node.tag === 'adw-window') && iconMeta) {
      lines.push(`icon ${node.id} ${iconMeta.appId} ${iconMeta.relPath}`);
    }

    for (const cls of node.classes) {
      lines.push(`add_class ${node.id} ${cls}`);
    }

    if (node.reactiveBinding) {
      bindingInstructions.push(
        `bind_text ${node.id} ${node.reactiveBinding.signalName} ${node.reactiveBinding.template}`,
      );
    }

    for (const [evtName, methodName] of Object.entries(node.events)) {
      const nativeEvt = evtName === 'click' ? 'clicked' : evtName;
      const methodMeta = methods[methodName];
      if (methodMeta) {
        const opcode = methodMeta.kind === 'set' ? 'on_set' : 'on_add';
        bindingInstructions.push(
          `${opcode} ${node.id} ${nativeEvt} ${methodMeta.signalName} ${methodMeta.value}`,
        );
      }
    }

    for (const child of node.children) {
      visitNode(child);
      appendInstructions.push(`append ${node.id} ${child.id}`);
    }
  }

  for (const root of astRoots) {
    visitNode(root);
  }

  lines.push(...appendInstructions);
  lines.push(...bindingInstructions);
  return lines.join('\n') + '\n';
}

export function compileBundle({ rebuildIcons = true } = {}) {
  const angularConfig = JSON.parse(readFileSync(resolve(rootDir, 'angular.json'), 'utf8'));
  const projectName = angularConfig.defaultProject || Object.keys(angularConfig.projects)[0];
  const projectCfg = angularConfig.projects[projectName];
  const nativeOptions = projectCfg.architect.native?.options || projectCfg.architect.build.options;

  const appDir = resolve(rootDir, projectCfg.sourceRoot, 'app');
  const tsSource = readFileSync(resolve(appDir, 'app.ts'), 'utf8');
  const htmlSource = readFileSync(resolve(appDir, 'app.html'), 'utf8');
  const cssSource = readFileSync(resolve(appDir, 'app.css'), 'utf8');
  const indexHtml = readFileSync(resolve(rootDir, projectCfg.sourceRoot, 'index.html'), 'utf8');

  const iconHrefMatch = indexHtml.match(/<link[^>]*rel="icon"[^>]*href="([^"]+)"/);
  const iconFileName = iconHrefMatch ? iconHrefMatch[1] : 'favicon.ico';
  const iconRelPath = `${projectCfg.root}/public/${iconFileName}`;
  const iconAbsPath = resolve(rootDir, iconRelPath);

  const { state, numericSignals, methods } = extractComponentMetadata(tsSource);
  const styleRules = parseStylesCss(cssSource);
  const templateAst = parseTemplateHtml(htmlSource, state, numericSignals);

  const outDir = resolve(rootDir, nativeOptions.outputPath);
  mkdirSync(outDir, { recursive: true });

  let iconMeta = null;
  if (existsSync(iconAbsPath)) {
    iconMeta = { appId: projectName, relPath: iconRelPath };
    if (rebuildIcons) {
      buildMultiResolutionIcons(iconAbsPath, projectName, outDir, state.windowTitle);
    }
  }

  const bundle = emitNgbBundle(
    styleRules,
    numericSignals,
    methods,
    templateAst,
    iconMeta,
    `${projectCfg.sourceRoot}/app/app.{ts,html,css}`,
  );

  const bundleFile = resolve(outDir, 'main.ngb');
  writeFileSync(bundleFile, bundle, 'utf8');
  console.log(`[Angular Native Compiler] Compiled ${projectName} -> ${bundleFile}`);
  return { projectName, sourceRoot: resolve(rootDir, projectCfg.sourceRoot), bundleFile };
}

export function watchBundle() {
  const { sourceRoot } = compileBundle({ rebuildIcons: true });
  console.log(`[Angular Native Watcher] Watching '${sourceRoot}' for changes...`);

  let debounceTimer = null;
  return watch(sourceRoot, { recursive: true }, (_eventType, filename) => {
    if (!filename || !/\.(ts|html|css)$/.test(filename)) return;
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      try {
        compileBundle({ rebuildIcons: filename.endsWith('index.html') });
      } catch (err) {
        console.error('[Angular Native Watcher] Compilation error:', err);
      }
    }, 60);
  });
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  if (process.argv.includes('--watch')) {
    watchBundle();
  } else {
    compileBundle({ rebuildIcons: true });
  }
}

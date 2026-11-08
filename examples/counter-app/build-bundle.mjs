import { existsSync, mkdirSync, readFileSync, watch, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { extractComponentMetadata } from './ast-metadata-extractor.mjs';
import { buildMultiResolutionIcons } from './icon-compiler.mjs';
import { pushHmrReload } from './hmr-client.mjs';
import { emitStyleInstruction, parseStylesCss } from './style-compiler.mjs';
import { parseTemplateHtml } from './template-compiler.mjs';

export { extractComponentMetadata, parseStylesCss, parseTemplateHtml, pushHmrReload };

const __dirname = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(__dirname, '../..');

function mapNativeEventName(evtName) {
  if (evtName === 'click') return 'clicked';
  if (evtName === 'change') return 'toggled';
  if (evtName === 'valueChange') return 'value-changed';
  return evtName;
}

/**
 * 4. Bytecode Emitter (SRP): Translates Styles IR + Signals + Icons + Template AST into `.ngb` instructions.
 */
export function emitNgbBundle(
  styleRules,
  numericSignals,
  methods,
  astRoots,
  iconMeta,
  sourceLabel,
) {
  const lines = [`// Auto-compiled by Angular Native Template Compiler from ${sourceLabel}`];

  for (const [sigName, initialValue] of Object.entries(numericSignals)) {
    lines.push(`signal ${sigName} ${initialValue}`);
  }

  for (const rule of styleRules) {
    lines.push(emitStyleInstruction(rule));
  }

  const appendInstructions = [];
  const bindingInstructions = [];

  function visitNode(node) {
    if (node.isComment) {
      lines.push(`comment ${node.id} ${node.text}`);
      return;
    }

    lines.push(`create ${node.id} ${node.tag}`);

    if (node.title) {
      lines.push(`text ${node.id} ${node.title}`);
    } else if (node.text) {
      lines.push(`text ${node.id} ${node.text}`);
    }

    if ((node.tag === 'ng-window' || node.tag === 'adw-window') && iconMeta) {
      lines.push(`icon ${node.id} ${iconMeta.appId} ${iconMeta.relPath}`);
    }

    for (const [attrKey, attrVal] of Object.entries(node.attributes || {})) {
      lines.push(`attr ${node.id} ${attrKey} ${attrVal}`);
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
      const nativeEvt = mapNativeEventName(evtName);
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
  const { state, numericSignals, methods, component } = extractComponentMetadata(tsSource);
  const htmlSource = readFileSync(resolve(appDir, component.templateUrl), 'utf8');
  const cssSource = readFileSync(resolve(appDir, component.styleUrl), 'utf8');
  const indexHtml = readFileSync(resolve(rootDir, projectCfg.sourceRoot, 'index.html'), 'utf8');

  const iconHrefMatch = indexHtml.match(/<link[^>]*rel="icon"[^>]*href="([^"]+)"/);
  const iconFileName = iconHrefMatch ? iconHrefMatch[1] : 'favicon.ico';
  const iconRelPath = `${projectCfg.root}/public/${iconFileName}`;
  const iconAbsPath = resolve(rootDir, iconRelPath);

  const componentRegistry = {};
  const allStyleRules = [...parseStylesCss(cssSource)];

  const candidateFiles = ['pokemon-card.ts', 'type-badge.ts'];
  for (const file of candidateFiles) {
    const filePath = resolve(appDir, file);
    if (existsSync(filePath)) {
      const ts = readFileSync(filePath, 'utf8');
      const meta = extractComponentMetadata(ts, file);
      const tmplPath = resolve(appDir, meta.component.templateUrl);
      const stylePath = resolve(appDir, meta.component.styleUrl);
      const templateHtml = existsSync(tmplPath) ? readFileSync(tmplPath, 'utf8') : '';
      if (existsSync(stylePath)) {
        const rules = parseStylesCss(readFileSync(stylePath, 'utf8'));
        allStyleRules.push(...rules);
      }
      componentRegistry[meta.component.selector] = {
        templateHtml,
        templateUrl: meta.component.templateUrl,
      };
    }
  }

  const templateAst = parseTemplateHtml(
    htmlSource,
    state,
    numericSignals,
    'app.html',
    componentRegistry,
  );

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
    allStyleRules,
    numericSignals,
    methods,
    templateAst,
    iconMeta,
    `${projectCfg.sourceRoot}/app/app.{ts,html,css}`,
  );

  const bundleFile = resolve(outDir, 'main.ngb');
  writeFileSync(bundleFile, bundle, 'utf8');
  console.log(`[Angular Native Compiler] Compiled ${projectName} -> ${bundleFile}`);

  const socketPath = resolve(outDir, 'hmr.sock');
  if (existsSync(socketPath)) {
    pushHmrReload(socketPath, bundle).then((res) => {
      if (res.connected && res.response?.status === 'ok') {
        console.log(`[Angular Native HMR] Pushed live bundle update over IPC -> ${socketPath}`);
      }
    });
  }

  return {
    projectName,
    sourceRoot: resolve(rootDir, projectCfg.sourceRoot),
    bundleFile,
    socketPath,
  };
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

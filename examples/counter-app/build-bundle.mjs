import { existsSync, mkdirSync, readFileSync, watch, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { extractComponentMetadata } from './ast-metadata-extractor.mjs';
import { buildMultiResolutionIcons } from './icon-compiler.mjs';
import { parseTemplateHtml } from './template-compiler.mjs';

export { extractComponentMetadata, parseTemplateHtml };

const __dirname = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(__dirname, '../..');

/**
 * 2. Style Parser (SRP): Parses `app.css` into intermediate Style IR.
 */
export function parseStylesCss(cssSource) {
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
    lines.push(
      `style ${rule.className} ${rule.background} ${rule.color} ${rule.padding} ${rule.spacing}`,
    );
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

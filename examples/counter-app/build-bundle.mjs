import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

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
 * 4. Bytecode Emitter (SRP): Translates Styles IR + Signals + Template AST into `.ngb` instructions.
 */
function emitNgbBundle(styleRules, numericSignals, methods, astRoots, sourceLabel) {
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

// Run compiler pipeline using `angular.json` configuration
const angularConfig = JSON.parse(readFileSync(resolve(rootDir, 'angular.json'), 'utf8'));
const projectName = angularConfig.defaultProject || Object.keys(angularConfig.projects)[0];
const projectCfg = angularConfig.projects[projectName];
const nativeOptions = projectCfg.architect.native?.options || projectCfg.architect.build.options;

const appDir = resolve(rootDir, projectCfg.sourceRoot, 'app');
const tsSource = readFileSync(resolve(appDir, 'app.ts'), 'utf8');
const htmlSource = readFileSync(resolve(appDir, 'app.html'), 'utf8');
const cssSource = readFileSync(resolve(appDir, 'app.css'), 'utf8');

const { state, numericSignals, methods } = extractComponentMetadata(tsSource);
const styleRules = parseStylesCss(cssSource);
const templateAst = parseTemplateHtml(htmlSource, state, numericSignals);
const bundle = emitNgbBundle(
  styleRules,
  numericSignals,
  methods,
  templateAst,
  `${projectCfg.sourceRoot}/app/app.{ts,html,css}`,
);

const outDir = resolve(rootDir, nativeOptions.outputPath);
mkdirSync(outDir, { recursive: true });
const bundleFile = resolve(outDir, 'main.ngb');
writeFileSync(bundleFile, bundle, 'utf8');
console.log(`[Angular Native Compiler] Compiled ${projectName} -> ${bundleFile}`);

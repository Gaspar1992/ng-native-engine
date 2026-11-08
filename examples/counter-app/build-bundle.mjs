import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(__dirname, '../..');

/**
 * 1. State Extractor (SRP): Extracts simple class fields from the component source.
 */
function extractComponentState(source) {
  const state = {};
  const propRegex = /^\s*([a-zA-Z_]\w*)\s*=\s*(?:'([^']*)'|"([^"]*)"|(\d+))\s*;/gm;
  let match;
  while ((match = propRegex.exec(source)) !== null) {
    const [, key, singleQuoted, doubleQuoted, numeric] = match;
    state[key] = singleQuoted ?? doubleQuoted ?? numeric;
  }
  return state;
}

/**
 * 2. Style Parser (SRP): Parses `static styles` CSS blocks into intermediate Style IR.
 */
function parseStylesBlock(source) {
  const stylesMatch = source.match(/static\s+styles\s*=\s*`([\s\S]*?)`/);
  if (!stylesMatch) return [];

  const cssBody = stylesMatch[1];
  const ruleRegex = /\.([a-zA-Z0-9_-]+)\s*\{([^}]*)\}/g;
  const rules = [];
  let match;

  while ((match = ruleRegex.exec(cssBody)) !== null) {
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
 * Resolves Angular `{{ interpolation }}` expressions against component state.
 */
function interpolate(text, state) {
  return text.replace(/\{\{\s*([a-zA-Z_]\w*)\s*\}\}/g, (_, key) =>
    key in state ? String(state[key]) : ''
  );
}

/**
 * 3. Template Parser (SRP): Parses `static template` XML/HTML into a hierarchical AST.
 */
function parseTemplateBlock(source, state) {
  const tplMatch = source.match(/static\s+template\s*=\s*`([\s\S]*?)`/);
  if (!tplMatch) {
    throw new Error('No `static template` found in component.');
  }

  const html = tplMatch[1];
  const tokenRegex = /<\/?([a-zA-Z0-9-]+)([^>]*)>|([^<]+)/g;
  const rootNodes = [];
  const stack = [];
  let autoIdCounter = 1;
  let match;

  while ((match = tokenRegex.exec(html)) !== null) {
    const [fullToken, tagName, attrString, textNode] = match;

    if (textNode !== undefined) {
      const cleaned = interpolate(textNode.trim(), state);
      if (cleaned && stack.length > 0) {
        stack[stack.length - 1].text = cleaned;
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
    const attrRegex = /([a-zA-Z0-9_-]+)\s*=\s*"([^"]*)"/g;
    let attrMatch;
    while ((attrMatch = attrRegex.exec(attrString)) !== null) {
      attrs[attrMatch[1]] = interpolate(attrMatch[2], state);
    }

    const node = {
      id: attrs.id || `node_${autoIdCounter++}`,
      tag: tagName,
      title: attrs.title || null,
      classes: attrs.class ? attrs.class.split(/\s+/).filter(Boolean) : [],
      text: null,
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
 * 4. Bytecode Emitter (SRP): Translates Styles IR + Template AST into `.ngb` instructions.
 */
function emitNgbBundle(styleRules, astRoots, sourcePath) {
  const lines = [`// Auto-compiled by Angular Native Template Compiler from ${sourcePath}`];

  for (const rule of styleRules) {
    lines.push(
      `style ${rule.className} ${rule.background} ${rule.color} ${rule.padding} ${rule.spacing}`
    );
  }

  const appendInstructions = [];

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

    for (const child of node.children) {
      visitNode(child);
      appendInstructions.push(`append ${node.id} ${child.id}`);
    }
  }

  for (const root of astRoots) {
    visitNode(root);
  }

  lines.push(...appendInstructions);
  return lines.join('\n') + '\n';
}

// Run compiler pipeline using `angular.json` configuration
const angularConfig = JSON.parse(readFileSync(resolve(rootDir, 'angular.json'), 'utf8'));
const defaultProject = angularConfig.defaultProject;
const buildOptions = angularConfig.projects[defaultProject].architect.build.options;

const componentRelPath = 'examples/counter-app/src/app.component.ts';
const componentSource = readFileSync(resolve(rootDir, componentRelPath), 'utf8');

const state = extractComponentState(componentSource);
const styleRules = parseStylesBlock(componentSource);
const templateAst = parseTemplateBlock(componentSource, state);
const bundle = emitNgbBundle(styleRules, templateAst, componentRelPath);

const outDir = resolve(rootDir, buildOptions.outputPath);
mkdirSync(outDir, { recursive: true });
const bundleFile = resolve(outDir, 'main.ngb');
writeFileSync(bundleFile, bundle, 'utf8');
console.log(`[Angular Native Compiler] Compiled ${componentRelPath} -> ${bundleFile}`);

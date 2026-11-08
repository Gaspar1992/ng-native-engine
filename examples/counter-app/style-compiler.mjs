import postcss from 'postcss';

const SIMPLE_CLASS_SELECTOR = /^\.([a-zA-Z0-9_-]+)$/;

function toPixelInt(raw) {
  const parsed = Number.parseInt(String(raw).trim().replace(/px$/, ''), 10);
  return Number.isFinite(parsed) && parsed >= 0 ? parsed : null;
}

function createEmptyRule(className) {
  return {
    className,
    background: 'none',
    color: 'none',
    padding: '0',
    spacing: '0',
    borderRadius: null,
    fontSize: null,
    fontWeight: null,
  };
}

function applyDeclaration(rule, prop, value) {
  switch (prop) {
    case 'background':
    case 'background-color':
      rule.background = value;
      break;
    case 'color':
      rule.color = value;
      break;
    case 'padding': {
      const px = toPixelInt(value);
      if (px !== null) rule.padding = String(px);
      break;
    }
    case 'gap':
    case 'spacing': {
      const px = toPixelInt(value);
      if (px !== null) rule.spacing = String(px);
      break;
    }
    case 'border-radius':
      rule.borderRadius = toPixelInt(value);
      break;
    case 'font-size':
      rule.fontSize = toPixelInt(value);
      break;
    case 'font-weight': {
      const keyword = { normal: 400, bold: 700 }[value];
      rule.fontWeight = keyword ?? toPixelInt(value);
      break;
    }
    default:
      break;
  }
}

/**
 * Parses component CSS with the `postcss` AST into the native Style IR.
 * Only simple class selectors (`.name`, including selector lists) are mapped;
 * pseudo-classes, combinators, and at-rules are skipped because the native
 * backends have no equivalent for them yet.
 */
export function parseStylesCss(cssSource, from = 'app.css') {
  const root = postcss.parse(cssSource, { from });
  const rulesByClass = new Map();

  root.walkRules((cssRule) => {
    if (cssRule.parent?.type === 'atrule') return;

    for (const selector of cssRule.selectors) {
      const match = selector.trim().match(SIMPLE_CLASS_SELECTOR);
      if (!match) continue;

      const className = match[1];
      const rule = rulesByClass.get(className) ?? createEmptyRule(className);
      cssRule.walkDecls((decl) => applyDeclaration(rule, decl.prop, decl.value.trim()));
      rulesByClass.set(className, rule);
    }
  });

  return [...rulesByClass.values()];
}

/**
 * Serializes a Style IR rule into a `.ngb` `style` instruction, appending optional
 * `key=value` extensions only when declared (keeps legacy bundles byte-identical).
 */
export function emitStyleInstruction(rule) {
  const base = `style ${rule.className} ${rule.background} ${rule.color} ${rule.padding} ${rule.spacing}`;
  const extensions = [];
  if (rule.borderRadius !== null) extensions.push(`radius=${rule.borderRadius}`);
  if (rule.fontSize !== null) extensions.push(`font-size=${rule.fontSize}`);
  if (rule.fontWeight !== null) extensions.push(`font-weight=${rule.fontWeight}`);
  return extensions.length > 0 ? `${base} ${extensions.join(' ')}` : base;
}

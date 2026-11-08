import { parseTemplate } from '@angular/compiler';

function extractExpressionIdentifier(exprAst) {
  if (!exprAst) return null;
  if (typeof exprAst.name === 'string') return exprAst.name;
  if (exprAst.receiver && typeof exprAst.receiver.name === 'string') {
    return exprAst.receiver.name;
  }
  return null;
}

function evaluateBoundValue(boundValue, state) {
  const ast = boundValue?.ast;
  if (!ast) {
    return { text: '', signalName: null, templatePattern: '{}' };
  }

  if (ast.value !== undefined) {
    return { text: String(ast.value), signalName: null, templatePattern: String(ast.value) };
  }

  if (Array.isArray(ast.strings) && Array.isArray(ast.expressions)) {
    let text = '';
    for (let i = 0; i < ast.strings.length; i++) {
      text += ast.strings[i];
      if (i < ast.expressions.length) {
        const ident = extractExpressionIdentifier(ast.expressions[i]);
        if (ident && ident in state) {
          text += String(state[ident]);
        }
      }
    }

    const firstIdent =
      ast.expressions.length === 1 ? extractExpressionIdentifier(ast.expressions[0]) : null;
    const templatePattern =
      ast.expressions.length === 1 ? `${ast.strings[0]}{}${ast.strings[1] ?? ''}` : text;

    return { text: text.trim(), signalName: firstIdent, templatePattern: templatePattern.trim() };
  }

  const ident = extractExpressionIdentifier(ast);
  const resolved = ident && ident in state ? String(state[ident]) : '';
  return { text: resolved, signalName: ident, templatePattern: '{}' };
}

function isBranchActive(branch, state) {
  if (!branch.expression) {
    return true;
  }
  const ident = extractExpressionIdentifier(branch.expression.ast);
  if (!ident || !(ident in state)) {
    return true;
  }
  const val = state[ident];
  return val !== 'false' && val !== '0' && val !== '' && Boolean(val);
}

/**
 * Parses an Angular 22 HTML template using `@angular/compiler`'s official `parseTemplate` AST,
 * resolving static attributes, `[property]` / `{{ interpolation }}` bindings, `(event)` outputs,
 * and `@if` control-flow blocks into the Native Scene Graph IR.
 */
export function parseTemplateHtml(
  html,
  state,
  numericSignals,
  templateUrl = 'app.html',
  componentRegistry = {},
  projectedChildren = null,
) {
  const parsed = parseTemplate(html, templateUrl, {
    preserveWhitespaces: false,
    collectCommentNodes: true,
  });

  let autoIdCounter = 1;

  function transformNodes(astNodes) {
    const out = [];

    for (const tmplNode of astNodes || []) {
      if (tmplNode.name === 'ng-content') {
        if (projectedChildren && projectedChildren.length > 0) {
          out.push(...projectedChildren);
        }
        continue;
      }

      if (Array.isArray(tmplNode.branches)) {
        const firstExpr = tmplNode.branches[0]?.expression?.source?.trim() || 'cond';
        out.push({
          id: `anchor_${autoIdCounter++}`,
          tag: '#comment',
          isComment: true,
          text: `@if(${firstExpr})`,
          title: null,
          classes: [],
          attributes: {},
          reactiveBinding: null,
          events: {},
          children: [],
        });

        for (const branch of tmplNode.branches) {
          if (isBranchActive(branch, state)) {
            out.push(...transformNodes(branch.children));
            break;
          }
        }
        continue;
      }

      if (typeof tmplNode.name !== 'string') {
        continue;
      }

      if (tmplNode.name in componentRegistry) {
        const compDef = componentRegistry[tmplNode.name];
        const compState = { ...state };
        for (const attr of tmplNode.attributes || []) {
          compState[attr.name] = attr.value;
        }
        for (const input of tmplNode.inputs || []) {
          const evaluated = evaluateBoundValue(input.value, state);
          compState[input.name] = evaluated.text;
        }

        const projected = transformNodes(tmplNode.children);
        const compNodes = parseTemplateHtml(
          compDef.templateHtml,
          compState,
          numericSignals,
          compDef.templateUrl || `${tmplNode.name}.html`,
          componentRegistry,
          projected,
        );

        const hostId = tmplNode.attributes?.find((a) => a.name === 'id')?.value;
        const hostClasses =
          tmplNode.attributes
            ?.find((a) => a.name === 'class')
            ?.value?.split(/\s+/)
            .filter(Boolean) || [];

        if (compNodes.length > 0) {
          if (hostId) compNodes[0].id = hostId;
          if (hostClasses.length > 0) {
            compNodes[0].classes = Array.from(new Set([...compNodes[0].classes, ...hostClasses]));
          }
        }

        out.push(...compNodes);
        continue;
      }

      const attrs = {};
      for (const attr of tmplNode.attributes || []) {
        attrs[attr.name] = attr.value;
      }

      for (const input of tmplNode.inputs || []) {
        const evaluated = evaluateBoundValue(input.value, state);
        attrs[input.name] = evaluated.text;
      }

      const events = {};
      for (const output of tmplNode.outputs || []) {
        const handlerName =
          extractExpressionIdentifier(output.handler?.ast) ||
          output.handler?.source?.replace(/\(\)\s*$/, '').trim();
        if (handlerName) {
          events[output.name] = handlerName;
        }
      }

      const customAttrs = {};
      for (const [k, v] of Object.entries(attrs)) {
        if (k !== 'id' && k !== 'title' && k !== 'class') {
          customAttrs[k] = v;
        }
      }

      let textContent = null;
      let reactiveBinding = null;
      const childElements = [];

      for (const child of tmplNode.children || []) {
        if (typeof child.name === 'string' || Array.isArray(child.branches)) {
          childElements.push(...transformNodes([child]));
        } else if (typeof child.value === 'string') {
          const trimmed = child.value.trim();
          if (trimmed) {
            textContent = trimmed;
          }
        } else if (child.value && typeof child.value === 'object') {
          const evaluated = evaluateBoundValue(child.value, state);
          if (evaluated.text) {
            textContent = evaluated.text;
          }
          if (evaluated.signalName && evaluated.signalName in numericSignals) {
            reactiveBinding = {
              signalName: evaluated.signalName,
              template: evaluated.templatePattern,
            };
          }
        }
      }

      out.push({
        id: attrs.id || `node_${autoIdCounter++}`,
        tag: tmplNode.name,
        title: attrs.title || null,
        classes: attrs.class ? attrs.class.split(/\s+/).filter(Boolean) : [],
        attributes: customAttrs,
        text: textContent,
        reactiveBinding,
        events,
        children: childElements,
      });
    }

    return out;
  }

  return transformNodes(parsed.nodes);
}

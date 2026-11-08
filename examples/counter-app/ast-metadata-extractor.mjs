import { parseSync } from 'oxc-parser';

function extractScalarLiteral(node) {
  if (!node) return null;

  if (node.type === 'Literal') {
    if (typeof node.value === 'number') {
      return { kind: 'number', value: node.value, raw: String(node.value) };
    }
    if (typeof node.value === 'string') {
      return { kind: 'string', value: node.value, raw: node.value };
    }
    if (typeof node.value === 'boolean') {
      return { kind: 'boolean', value: node.value, raw: String(node.value) };
    }
  }

  if (
    node.type === 'UnaryExpression' &&
    (node.operator === '-' || node.operator === '+') &&
    node.argument?.type === 'Literal' &&
    typeof node.argument.value === 'number'
  ) {
    const value = node.operator === '-' ? -node.argument.value : node.argument.value;
    return { kind: 'number', value, raw: String(value) };
  }

  return null;
}

function extractDecoratorMetadata(decorators = []) {
  const component = {
    selector: 'app-root',
    templateUrl: './app.html',
    styleUrl: './app.css',
  };

  for (const dec of decorators) {
    const expr = dec.expression;
    if (
      expr?.type !== 'CallExpression' ||
      expr.callee?.type !== 'Identifier' ||
      expr.callee.name !== 'Component'
    ) {
      continue;
    }

    const arg = expr.arguments?.[0];
    if (arg?.type !== 'ObjectExpression') continue;

    for (const prop of arg.properties || []) {
      if (prop.type !== 'Property' || prop.key?.type !== 'Identifier') continue;
      const lit = extractScalarLiteral(prop.value);
      if (!lit || lit.kind !== 'string') continue;

      if (prop.key.name === 'selector') component.selector = lit.value;
      if (prop.key.name === 'templateUrl') component.templateUrl = lit.value;
      if (prop.key.name === 'styleUrl') component.styleUrl = lit.value;
    }
  }

  return component;
}

function resolveReturnedExpression(fnNode) {
  if (!fnNode?.body) return null;
  if (fnNode.body.type !== 'BlockStatement') {
    return fnNode.body;
  }

  for (const stmt of fnNode.body.body || []) {
    if (stmt.type === 'ReturnStatement') {
      return stmt.argument;
    }
  }
  return null;
}

function extractMethodMutation(methodDef, methods) {
  const methodName = methodDef.key?.type === 'Identifier' ? methodDef.key.name : null;
  if (!methodName) return;

  const statements = methodDef.value?.body?.body || [];
  for (const stmt of statements) {
    if (stmt.type !== 'ExpressionStatement' || stmt.expression?.type !== 'CallExpression') {
      continue;
    }

    const callExpr = stmt.expression;
    const callee = callExpr.callee;
    if (
      callee?.type !== 'MemberExpression' ||
      callee.property?.type !== 'Identifier' ||
      callee.object?.type !== 'MemberExpression' ||
      callee.object.object?.type !== 'ThisExpression' ||
      callee.object.property?.type !== 'Identifier'
    ) {
      continue;
    }

    const signalName = callee.object.property.name;
    const action = callee.property.name;

    if (action === 'set') {
      const lit = extractScalarLiteral(callExpr.arguments?.[0]);
      if (lit?.kind === 'number') {
        methods[methodName] = { kind: 'set', signalName, value: lit.value };
      }
      continue;
    }

    if (action === 'update') {
      const fnArg = callExpr.arguments?.[0];
      const retExpr = resolveReturnedExpression(fnArg);
      if (
        retExpr?.type === 'BinaryExpression' &&
        (retExpr.operator === '+' || retExpr.operator === '-')
      ) {
        const rightLit = extractScalarLiteral(retExpr.right);
        if (rightLit?.kind === 'number') {
          const delta = retExpr.operator === '-' ? -rightLit.value : rightLit.value;
          methods[methodName] = { kind: 'add', signalName, value: delta };
        }
      }
    }
  }
}

/**
 * Parses an Angular 22 TypeScript component source file using the Rust-based `oxc-parser` AST
 * and extracts `@Component` decorator paths, `signal(...)` declarations, class properties, and mutation methods.
 */
export function extractComponentMetadata(tsSource, filename = 'app.ts') {
  const parsed = parseSync(filename, tsSource);
  const state = {};
  const numericSignals = {};
  const methods = {};
  let component = {
    selector: 'app-root',
    templateUrl: './app.html',
    styleUrl: './app.css',
  };

  for (const stmt of parsed.program.body || []) {
    const decl =
      stmt.type === 'ExportNamedDeclaration' || stmt.type === 'ExportDefaultDeclaration'
        ? stmt.declaration
        : stmt;

    if (decl?.type !== 'ClassDeclaration') continue;

    component = extractDecoratorMetadata(decl.decorators);

    for (const member of decl.body?.body || []) {
      if (member.type === 'PropertyDefinition' && member.key?.type === 'Identifier') {
        const propName = member.key.name;
        const valNode = member.value;

        if (
          valNode?.type === 'CallExpression' &&
          valNode.callee?.type === 'Identifier' &&
          valNode.callee.name === 'signal'
        ) {
          const lit = extractScalarLiteral(valNode.arguments?.[0]);
          if (lit) {
            state[propName] = lit.raw;
            if (lit.kind === 'number') {
              numericSignals[propName] = lit.value;
            }
          }
          continue;
        }

        const lit = extractScalarLiteral(valNode);
        if (lit && !(propName in state)) {
          state[propName] = lit.raw;
        }
      } else if (member.type === 'MethodDefinition' && member.kind === 'method') {
        extractMethodMutation(member, methods);
      }
    }
  }

  return { state, numericSignals, methods, component };
}

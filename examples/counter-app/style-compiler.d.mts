export interface StyleRuleIr {
  className: string;
  background: string;
  color: string;
  padding: string;
  spacing: string;
  borderRadius: number | null;
  fontSize: number | null;
  fontWeight: number | null;
}

export function parseStylesCss(cssSource: string, from?: string): StyleRuleIr[];

export function emitStyleInstruction(rule: StyleRuleIr): string;

export interface ReactiveBindingIr {
  signalName: string;
  template: string;
}

export interface TemplateAstNode {
  id: string;
  tag: string;
  isComment?: boolean;
  text: string | null;
  title: string | null;
  classes: string[];
  attributes: Record<string, string>;
  reactiveBinding: ReactiveBindingIr | null;
  events: Record<string, string>;
  children: TemplateAstNode[];
}

export function parseTemplateHtml(
  html: string,
  state: Record<string, string>,
  numericSignals: Record<string, number>,
  templateUrl?: string,
): TemplateAstNode[];

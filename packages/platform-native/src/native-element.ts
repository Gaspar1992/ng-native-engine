/**
 * Lightweight scene-graph proxy node used by `NativeRenderer2`.
 * Allows synchronous `parentNode()` and `nextSibling()` traversal required by Angular's
 * `@if`, `@for`, `@switch`, and `<ng-content>` algorithms without round-tripping across FFI.
 */
export class NativeElementNode {
  readonly id: string;
  readonly tag: string;
  readonly isComment: boolean;
  parent: NativeElementNode | null = null;
  readonly children: NativeElementNode[] = [];
  readonly attributes = new Map<string, string>();
  readonly classes = new Set<string>();
  readonly styles = new Map<string, string>();
  textContent = '';

  constructor(id: string, tag: string, isComment = false) {
    this.id = id;
    this.tag = tag;
    this.isComment = isComment;
  }

  get nextSibling(): NativeElementNode | null {
    if (!this.parent) {
      return null;
    }
    const idx = this.parent.children.indexOf(this);
    if (idx === -1 || idx + 1 >= this.parent.children.length) {
      return null;
    }
    return this.parent.children[idx + 1] ?? null;
  }
}

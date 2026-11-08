import {
  Injectable,
  Renderer2,
  RendererFactory2,
  RendererStyleFlags2,
  RendererType2,
} from '@angular/core';
import { ensureNgNativeBridge, NgNativeBridge } from './bridge';
import { NativeElementNode } from './native-element';

/**
 * Custom Angular `Renderer2` implementation that translates Ivy template instructions
 * (`createElement`, `createComment`, `insertBefore`, `removeChild`, `setAttribute`, `listen`)
 * into `globalThis.__NG_NATIVE__` scene graph commands.
 */
export class NativeRenderer2 extends Renderer2 {
  readonly data: { [key: string]: unknown } = Object.create(null);
  override destroyNode: ((node: unknown) => void) | null;

  private readonly bridge: NgNativeBridge;
  private readonly nodesById: Map<string, NativeElementNode>;
  private readonly nextId: () => string;

  constructor(
    bridge: NgNativeBridge,
    nodesById: Map<string, NativeElementNode>,
    nextId: () => string,
  ) {
    super();
    this.bridge = bridge;
    this.nodesById = nodesById;
    this.nextId = nextId;
    this.destroyNode = (node: unknown) => {
      if (node instanceof NativeElementNode) {
        this.nodesById.delete(node.id);
        this.bridge.destroyNode(node.id);
      }
    };
  }

  override destroy(): void {
    // Renderer lifecycle cleanup
  }

  override createElement(name: string, _namespace?: string | null): NativeElementNode {
    const id = this.nextId();
    const node = new NativeElementNode(id, name, false);
    this.nodesById.set(id, node);
    this.bridge.createElement(id, name);
    return node;
  }

  override createComment(value: string): NativeElementNode {
    const id = this.nextId();
    const cleanComment = value.trim() || 'anchor';
    const node = new NativeElementNode(id, '#comment', true);
    node.textContent = cleanComment;
    this.nodesById.set(id, node);
    this.bridge.createComment(id, cleanComment);
    return node;
  }

  override createText(value: string): NativeElementNode {
    const id = this.nextId();
    const node = new NativeElementNode(id, 'ng-label', false);
    node.textContent = value;
    this.nodesById.set(id, node);
    this.bridge.createElement(id, 'ng-label');
    if (value.trim().length > 0) {
      this.bridge.setText(id, value);
    }
    return node;
  }

  override appendChild(parent: NativeElementNode, newChild: NativeElementNode): void {
    if (newChild.parent) {
      this.removeChild(newChild.parent, newChild);
    }
    newChild.parent = parent;
    parent.children.push(newChild);
    this.bridge.appendChild(parent.id, newChild.id);
  }

  override insertBefore(
    parent: NativeElementNode,
    newChild: NativeElementNode,
    refChild: NativeElementNode | null,
    _isMove?: boolean,
  ): void {
    if (!parent) {
      return;
    }
    const existingIdx = parent.children.indexOf(newChild);
    if (existingIdx !== -1) {
      parent.children.splice(existingIdx, 1);
    }
    newChild.parent = parent;

    if (refChild) {
      const refIdx = parent.children.indexOf(refChild);
      if (refIdx !== -1) {
        parent.children.splice(refIdx, 0, newChild);
        this.bridge.insertBefore(parent.id, newChild.id, refChild.id);
        return;
      }
    }

    parent.children.push(newChild);
    this.bridge.insertBefore(parent.id, newChild.id);
  }

  override removeChild(
    parent: NativeElementNode | null,
    oldChild: NativeElementNode,
    _isHostElement?: boolean,
  ): void {
    const targetParent = parent ?? oldChild.parent;
    if (!targetParent) {
      return;
    }
    const idx = targetParent.children.indexOf(oldChild);
    if (idx !== -1) {
      targetParent.children.splice(idx, 1);
    }
    oldChild.parent = null;
    this.bridge.removeChild(targetParent.id, oldChild.id);
  }

  override selectRootElement(
    selectorOrNode: string | NativeElementNode,
    _preserveContent?: boolean,
  ): NativeElementNode {
    if (selectorOrNode instanceof NativeElementNode) {
      return selectorOrNode;
    }
    const normalizedId = selectorOrNode.replace(/^[#.]/, '');
    const existing = this.nodesById.get(normalizedId);
    if (existing) {
      return existing;
    }
    const root = new NativeElementNode(normalizedId, 'ng-window', false);
    this.nodesById.set(normalizedId, root);
    this.bridge.createElement(normalizedId, 'ng-window');
    return root;
  }

  override parentNode(node: NativeElementNode): NativeElementNode | null {
    return node.parent;
  }

  override nextSibling(node: NativeElementNode): NativeElementNode | null {
    return node.nextSibling;
  }

  override setAttribute(
    el: NativeElementNode,
    name: string,
    value: string,
    _namespace?: string | null,
  ): void {
    el.attributes.set(name, value);
    if (name === 'title' && (el.tag === 'ng-window' || el.tag === 'ng-header-bar')) {
      this.bridge.setText(el.id, value);
    } else {
      this.bridge.setAttribute(el.id, name, value);
    }
  }

  override removeAttribute(el: NativeElementNode, name: string, _namespace?: string | null): void {
    el.attributes.delete(name);
  }

  override addClass(el: NativeElementNode, name: string): void {
    el.classes.add(name);
    this.bridge.addClass(el.id, name);
  }

  override removeClass(el: NativeElementNode, name: string): void {
    el.classes.delete(name);
  }

  override setStyle(
    el: NativeElementNode,
    style: string,
    value: unknown,
    _flags?: RendererStyleFlags2,
  ): void {
    const strVal = String(value);
    el.styles.set(style, strVal);
    this.bridge.setAttribute(el.id, style, strVal);
  }

  override removeStyle(el: NativeElementNode, style: string, _flags?: RendererStyleFlags2): void {
    el.styles.delete(style);
  }

  override setProperty(el: NativeElementNode, name: string, value: unknown): void {
    const strVal = String(value ?? '');
    if (name === 'textContent' || name === 'innerText' || name === 'label') {
      el.textContent = strVal;
      this.bridge.setText(el.id, strVal);
    } else {
      el.attributes.set(name, strVal);
      this.bridge.setAttribute(el.id, name, strVal);
    }
  }

  override setValue(node: NativeElementNode, value: string): void {
    node.textContent = value;
    this.bridge.setText(node.id, value);
  }

  override listen(
    target: 'window' | 'document' | 'body' | NativeElementNode,
    eventName: string,
    callback: (event: unknown) => boolean | void,
  ): () => void {
    const nodeId = typeof target === 'string' ? target : target.id;
    const nativeEvent = eventName === 'click' ? 'clicked' : eventName;
    return this.bridge.listen(nodeId, nativeEvent, (evt) => {
      callback(evt);
    });
  }
}

@Injectable({ providedIn: 'root' })
export class NativeRendererFactory2 implements RendererFactory2 {
  private readonly bridge: NgNativeBridge;
  private readonly nodesById = new Map<string, NativeElementNode>();
  private readonly defaultRenderer: NativeRenderer2;
  private nodeCounter = 1;

  constructor() {
    this.bridge = ensureNgNativeBridge();
    this.defaultRenderer = new NativeRenderer2(this.bridge, this.nodesById, () => {
      const id = `ng_node_${this.nodeCounter++}`;
      return id;
    });
  }

  createRenderer(_hostElement: unknown, _type: RendererType2 | null): Renderer2 {
    return this.defaultRenderer;
  }

  begin(): void {
    // Frame batching begins
  }

  end(): void {
    // Frame batching completes
  }

  getNode(id: string): NativeElementNode | undefined {
    return this.nodesById.get(id);
  }
}

export interface NativeHttpBridgeResponse {
  status: number;
  statusText: string;
  body: string;
}

/**
 * Host FFI bridge contract injected at `globalThis.__NG_NATIVE__` by the Rust `JsEngineBackend`
 * (`JscEngine` or `QuickJsEngine`).
 */
export interface NgNativeBridge {
  registerStyle(cls: string, bg?: string, fg?: string, pad?: number, space?: number): void;
  declareSignal(name: string, initial?: number): void;
  createElement(id: string, tag: string): string;
  createComment(id: string, text?: string): string;
  setText(id: string, text: string): void;
  setAttribute(id: string, key: string, value: string): void;
  setWindowIcon(id: string, appId: string, icoPath: string): void;
  appendChild(parent: string, child: string): void;
  insertBefore(parent: string, child: string, refChild?: string): void;
  removeChild(parent: string, child: string): void;
  addClass(id: string, className: string): void;
  bindText(id: string, signalName: string, template: string): void;
  onAdd(id: string, eventName: string, signalName: string, delta?: number): void;
  onSet(id: string, eventName: string, signalName: string, value?: number): void;
  listen(id: string, eventName: string, callback: (event?: unknown) => void): () => void;
  destroyNode(id: string): void;
  getSystemTheme(): 'dark' | 'light';
  setSystemTheme(theme: 'dark' | 'light'): void;
  getClipboard(): string;
  setClipboard(text: string): void;
  sendNotification(title: string, body: string, icon?: string): void;
  openFileDialog(title?: string, defaultPath?: string): string;
  saveFileDialog(title?: string, suggestedName?: string): string;
  registerHttpEndpoint(method: string, url: string, status: number, body: string): void;
  httpRequest(method: string, url: string, body?: string): NativeHttpBridgeResponse;
  __dispatchEvent(id: string, eventName: string, eventPayload?: unknown): void;
  __flushTimers(): void;
  __drainQueue(): string;
}

declare global {
  var __NG_NATIVE__: NgNativeBridge | undefined;
}

/**
 * Ensures `globalThis.__NG_NATIVE__` is initialized.
 * In the Rust native runtime (`JscEngine` / `QuickJsEngine`), the host injects this at startup;
 * in Node/Vitest unit tests, this provides an identical in-memory command queue.
 */
export function ensureNgNativeBridge(): NgNativeBridge {
  if (globalThis.__NG_NATIVE__) {
    return globalThis.__NG_NATIVE__;
  }

  let queue: string[] = [];
  let systemTheme: 'dark' | 'light' = 'dark';
  let clipboardText = '';
  const httpEndpoints = new Map<string, NativeHttpBridgeResponse>();
  const listeners = new Map<
    number,
    { id: string; eventName: string; callback: (e?: unknown) => void }
  >();
  let nextListenerId = 1;

  const fallbackBridge: NgNativeBridge = {
    registerStyle(cls, bg = 'none', fg = 'none', pad = 0, space = 0) {
      queue.push(`style ${cls} ${bg} ${fg} ${pad} ${space}`);
    },
    declareSignal(name, initial = 0) {
      queue.push(`signal ${name} ${initial}`);
    },
    createElement(id, tag) {
      queue.push(`create ${id} ${tag}`);
      return id;
    },
    createComment(id, text = 'anchor') {
      queue.push(`comment ${id} ${text}`);
      return id;
    },
    setText(id, text) {
      queue.push(`text ${id} ${text}`);
    },
    setAttribute(id, key, value) {
      queue.push(`attr ${id} ${key} ${value}`);
    },
    setWindowIcon(id, appId, icoPath) {
      queue.push(`icon ${id} ${appId} ${icoPath}`);
    },
    appendChild(parent, child) {
      queue.push(`append ${parent} ${child}`);
    },
    insertBefore(parent, child, refChild) {
      if (refChild) {
        queue.push(`insert_before ${parent} ${child} ${refChild}`);
      } else {
        queue.push(`insert_before ${parent} ${child}`);
      }
    },
    removeChild(parent, child) {
      queue.push(`remove_child ${parent} ${child}`);
    },
    addClass(id, className) {
      queue.push(`add_class ${id} ${className}`);
    },
    bindText(id, signalName, template) {
      queue.push(`bind_text ${id} ${signalName} ${template}`);
    },
    onAdd(id, eventName, signalName, delta = 1) {
      queue.push(`on_add ${id} ${eventName} ${signalName} ${delta}`);
    },
    onSet(id, eventName, signalName, value = 0) {
      queue.push(`on_set ${id} ${eventName} ${signalName} ${value}`);
    },
    listen(id, eventName, callback) {
      const lid = nextListenerId++;
      listeners.set(lid, { id, eventName, callback });
      return () => {
        listeners.delete(lid);
      };
    },
    destroyNode(id) {
      queue.push(`destroy ${id}`);
    },
    getSystemTheme() {
      return systemTheme;
    },
    setSystemTheme(theme) {
      systemTheme = theme === 'light' ? 'light' : 'dark';
    },
    getClipboard() {
      return clipboardText;
    },
    setClipboard(text) {
      clipboardText = text;
    },
    sendNotification(title, body, icon = 'ng-linux-counter') {
      queue.push(`comment notify_${nextListenerId++} [${icon}] ${title}: ${body}`);
    },
    openFileDialog(_title = 'Open File', defaultPath = '/home/user/document.txt') {
      return defaultPath;
    },
    saveFileDialog(_title = 'Save File', suggestedName = '/home/user/untitled.txt') {
      return suggestedName;
    },
    registerHttpEndpoint(method, url, status, body) {
      const key = `${method.trim().toUpperCase()} ${url.trim()}`;
      httpEndpoints.set(key, {
        status,
        statusText: status < 400 ? 'OK' : 'Error',
        body,
      });
    },
    httpRequest(method, url, _body) {
      const key = `${method.trim().toUpperCase()} ${url.trim()}`;
      return (
        httpEndpoints.get(key) ?? {
          status: 404,
          statusText: 'Not Found',
          body: '{"error":"Endpoint not registered"}',
        }
      );
    },
    __dispatchEvent(id, eventName, eventPayload = {}) {
      for (const entry of listeners.values()) {
        if (entry.id === id && entry.eventName === eventName) {
          entry.callback(eventPayload);
        }
      }
    },
    __flushTimers() {
      // No-op in synchronous fallback
    },
    __drainQueue() {
      const out = queue.join('\n');
      queue = [];
      return out;
    },
  };

  globalThis.__NG_NATIVE__ = fallbackBridge;
  return fallbackBridge;
}

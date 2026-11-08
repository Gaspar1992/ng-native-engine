/// Standard `globalThis.__NG_NATIVE__` + Web/Host globals (`console`, `queueMicrotask`, `setTimeout`)
/// injected into every JS engine context.
/// `@ng-native/platform` (`NativeRenderer2`), `@ng-native/forms`, `@ng-native/http`, and
/// `@ng-native/desktop` call this object to queue frame-batched scene/desktop commands
/// and register live JS event listener closures.
pub const NG_NATIVE_BOOTSTRAP_JS: &str = r"
globalThis.__NG_NATIVE__ = (() => {
  let queue = [];
  let logs = [];
  let timers = new Map();
  let nextTimerId = 1;
  let listeners = new Map();
  let nextListenerId = 1;
  let systemTheme = 'dark';
  let clipboardText = '';
  let httpEndpoints = new Map();

  globalThis.console = {
    log: (...args) => logs.push(['LOG', args.join(' ')]),
    info: (...args) => logs.push(['INFO', args.join(' ')]),
    warn: (...args) => logs.push(['WARN', args.join(' ')]),
    error: (...args) => logs.push(['ERROR', args.join(' ')]),
  };

  if (typeof globalThis.queueMicrotask !== 'function') {
    globalThis.queueMicrotask = (cb) => Promise.resolve().then(cb);
  }

  globalThis.setTimeout = (cb, _ms = 0) => {
    const id = nextTimerId++;
    timers.set(id, cb);
    return id;
  };

  globalThis.clearTimeout = (id) => {
    timers.delete(id);
  };

  return {
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
      return () => listeners.delete(lid);
    },
     __dispatchEvent(id, eventName, eventPayload = {}) {
      for (const entry of listeners.values()) {
        if (entry.id === id && entry.eventName === eventName) {
          entry.callback(eventPayload);
        }
      }
    },
    __flushTimers() {
      const pending = Array.from(timers.values());
      timers.clear();
      for (const cb of pending) {
        cb();
      }
    },
    destroyNode(id) {
      queue.push(`destroy ${id}`);
    },
    getSystemTheme() {
      return systemTheme;
    },
    setSystemTheme(theme) {
      systemTheme = theme === 'light' ? 'light' : 'dark';
      queue.push(`theme ${systemTheme}`);
    },
    getClipboard() {
      return clipboardText;
    },
    setClipboard(text) {
      clipboardText = String(text ?? '');
      queue.push(`clipboard ${clipboardText}`);
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
      const key = `${String(method).trim().toUpperCase()} ${String(url).trim()}`;
      httpEndpoints.set(key, {
        status: Number(status),
        statusText: Number(status) < 400 ? 'OK' : 'Error',
        body: String(body ?? ''),
      });
    },
    httpRequest(method, url, _body) {
      const key = `${String(method).trim().toUpperCase()} ${String(url).trim()}`;
      return httpEndpoints.get(key) || {
        status: 404,
        statusText: 'Not Found',
        body: JSON.stringify({ error: 'Endpoint not registered' }),
      };
    },
    __drainQueue() {
      const out = queue.join('\n');
      queue = [];
      return out;
    }
  };
})();
";

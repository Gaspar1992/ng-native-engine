import {
  bootstrapNativeApplication,
  ensureNgNativeBridge,
  NativeRendererFactory2,
} from '@ng-native/platform';
import { App } from './app';

describe('@ng-native/platform — NativeRenderer2 & Bridge', () => {
  beforeEach(() => {
    const bridge = ensureNgNativeBridge();
    bridge.__drainQueue();
  });

  it('translates Renderer2 element, comment, insertBefore, and attribute calls into .ngb opcodes', () => {
    const bridge = ensureNgNativeBridge();
    const factory = new NativeRendererFactory2();
    const renderer = factory.createRenderer(null, null);

    const win = renderer.selectRootElement('#win');
    renderer.setAttribute(win, 'title', 'NgNative Renderer2');

    const box = renderer.createElement('ng-box');
    renderer.setAttribute(box, 'orientation', 'vertical');
    renderer.addClass(box, 'main-card');
    renderer.appendChild(win, box);

    const anchor = renderer.createComment('@if(showGreeting)');
    renderer.appendChild(box, anchor);

    const dynLabel = renderer.createElement('ng-label');
    renderer.setProperty(dynLabel, 'textContent', 'Insertado via NativeRenderer2');
    renderer.insertBefore(box, dynLabel, anchor);

    expect(renderer.parentNode(dynLabel)).toBe(box);
    expect(renderer.nextSibling(dynLabel)).toBe(anchor);

    const instructions = bridge.__drainQueue();
    expect(instructions).toContain('create win ng-window');
    expect(instructions).toContain('text win NgNative Renderer2');
    expect(instructions).toContain('attr ng_node_1 orientation vertical');
    expect(instructions).toContain('add_class ng_node_1 main-card');
    expect(instructions).toContain('comment ng_node_2 @if(showGreeting)');
    expect(instructions).toContain('insert_before ng_node_1 ng_node_3 ng_node_2');

    renderer.removeChild(box, dynLabel);
    expect(bridge.__drainQueue()).toContain('remove_child ng_node_1 ng_node_3');
  });

  it('dispatches native click events back into Angular Renderer2.listen callbacks', () => {
    const bridge = ensureNgNativeBridge();
    const factory = new NativeRendererFactory2();
    const renderer = factory.createRenderer(null, null);

    const btn = renderer.createElement('ng-button');
    let clickedCount = 0;

    const unlisten = renderer.listen(btn, 'click', () => {
      clickedCount += 1;
    });

    bridge.__dispatchEvent(btn.id, 'clicked');
    bridge.__dispatchEvent(btn.id, 'clicked');
    expect(clickedCount).toBe(2);

    unlisten();
    bridge.__dispatchEvent(btn.id, 'clicked');
    expect(clickedCount).toBe(2);
  });

  it('bootstraps an Angular root component with provideNativePlatform()', () => {
    const ctx = bootstrapNativeApplication(App);
    expect(ctx.rootComponent).toBe(App);
    expect(ctx.config.providers.length).toBeGreaterThan(0);
  });
});

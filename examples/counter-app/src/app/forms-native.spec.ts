import { ElementRef, Injector, Renderer2, runInInjectionContext } from '@angular/core';
import { FormControl } from '@angular/forms';
import {
  NativeBooleanValueAccessor,
  NativeNumberValueAccessor,
  NativeTextValueAccessor,
} from '@ng-native/forms';
import {
  ensureNgNativeBridge,
  NativeElementNode,
  NativeRendererFactory2,
} from '@ng-native/platform';

describe('@ng-native/forms — ControlValueAccessor Directives', () => {
  let renderer: Renderer2;

  beforeEach(() => {
    const bridge = ensureNgNativeBridge();
    bridge.__drainQueue();
    const factory = new NativeRendererFactory2();
    renderer = factory.createRenderer(null, null);
  });

  function createDirective<T>(directiveType: new () => T, element: NativeElementNode): T {
    const injector = Injector.create({
      providers: [
        { provide: Renderer2, useValue: renderer },
        { provide: ElementRef, useValue: new ElementRef(element) },
      ],
    });
    return runInInjectionContext(injector, () => new directiveType());
  }

  it('binds NativeTextValueAccessor (<ng-entry>) two-way with FormControl and native input events', () => {
    const bridge = ensureNgNativeBridge();
    const entryNode = renderer.createElement('ng-entry') as NativeElementNode;
    const accessor = createDirective(NativeTextValueAccessor, entryNode);
    const control = new FormControl('Initial user');

    accessor.writeValue(control.value);
    accessor.registerOnChange((val) => control.setValue(val, { emitEvent: false }));

    const writeOps = bridge.__drainQueue();
    expect(writeOps).toContain(`text ${entryNode.id} Initial user`);
    expect(writeOps).toContain(`attr ${entryNode.id} value Initial user`);

    bridge.__dispatchEvent(entryNode.id, 'input', {
      type: 'input',
      targetId: entryNode.id,
      value: 'Ada Lovelace',
    });

    expect(control.value).toBe('Ada Lovelace');
    expect(entryNode.textContent).toBe('Ada Lovelace');

    accessor.setDisabledState(true);
    expect(bridge.__drainQueue()).toContain(`attr ${entryNode.id} sensitive false`);
    accessor.ngOnDestroy();
  });

  it('binds NativeBooleanValueAccessor (<ng-switch> & <ng-checkbox>) two-way with toggled events', () => {
    const bridge = ensureNgNativeBridge();
    const switchNode = renderer.createElement('ng-switch') as NativeElementNode;
    const accessor = createDirective(NativeBooleanValueAccessor, switchNode);
    const darkThemeControl = new FormControl(false);

    accessor.writeValue(darkThemeControl.value);
    accessor.registerOnChange((val) => darkThemeControl.setValue(val, { emitEvent: false }));

    expect(bridge.__drainQueue()).toContain(`attr ${switchNode.id} active false`);

    bridge.__dispatchEvent(switchNode.id, 'toggled', {
      type: 'toggled',
      targetId: switchNode.id,
      checked: true,
      value: true,
    });

    expect(darkThemeControl.value).toBe(true);
    expect(switchNode.attributes.get('active')).toBe('true');
    accessor.ngOnDestroy();
  });

  it('binds NativeNumberValueAccessor (<ng-slider> & <ng-select>) two-way with value-changed events', () => {
    const bridge = ensureNgNativeBridge();
    const sliderNode = renderer.createElement('ng-slider') as NativeElementNode;
    const sliderAccessor = createDirective(NativeNumberValueAccessor, sliderNode);
    const volumeControl = new FormControl(40);

    sliderAccessor.writeValue(volumeControl.value);
    sliderAccessor.registerOnChange((val) => volumeControl.setValue(val, { emitEvent: false }));

    expect(bridge.__drainQueue()).toContain(`attr ${sliderNode.id} value 40`);

    bridge.__dispatchEvent(sliderNode.id, 'value-changed', {
      type: 'value-changed',
      targetId: sliderNode.id,
      value: 85,
    });
    expect(volumeControl.value).toBe(85);

    const selectNode = renderer.createElement('ng-select') as NativeElementNode;
    const selectAccessor = createDirective(NativeNumberValueAccessor, selectNode);
    selectAccessor.writeValue(2);
    expect(bridge.__drainQueue()).toContain(`attr ${selectNode.id} selected 2`);

    sliderAccessor.ngOnDestroy();
    selectAccessor.ngOnDestroy();
  });
});

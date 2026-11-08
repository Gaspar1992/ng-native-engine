import { Directive, ElementRef, forwardRef, inject, OnDestroy, Renderer2 } from '@angular/core';
import { ControlValueAccessor, NG_VALUE_ACCESSOR } from '@angular/forms';
import { NativeElementNode } from '@ng-native/platform';

interface NativeWidgetEvent {
  type?: string;
  targetId?: string;
  value?: string | number | boolean;
  checked?: boolean;
  detail?: unknown;
}

/**
 * Angular `ControlValueAccessor` for `<ng-entry>` / `<gtk-entry>` text input widgets.
 * Bridges `GtkEditable::changed` (`input`) native signals and `gtk_editable_set_text`.
 */
@Directive({
  selector:
    'ng-entry[formControlName],ng-entry[formControl],ng-entry[ngModel],gtk-entry[formControlName],gtk-entry[formControl],gtk-entry[ngModel]',
  standalone: true,
  providers: [
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => NativeTextValueAccessor),
      multi: true,
    },
  ],
})
export class NativeTextValueAccessor implements ControlValueAccessor, OnDestroy {
  private readonly renderer = inject(Renderer2);
  private readonly elementRef = inject<ElementRef<NativeElementNode>>(ElementRef);
  private onChange: (value: string) => void = () => {};
  private onTouched: () => void = () => {};
  private readonly unlistenFns: Array<() => void> = [];

  constructor() {
    const el = this.elementRef.nativeElement;
    this.unlistenFns.push(
      this.renderer.listen(el, 'input', (event: unknown) => {
        const payload = event as NativeWidgetEvent | undefined;
        const nextText =
          typeof payload?.value === 'string' ? payload.value : String(payload?.value ?? '');
        el.textContent = nextText;
        el.attributes.set('value', nextText);
        this.onChange(nextText);
      }),
      this.renderer.listen(el, 'blur', () => {
        this.onTouched();
      }),
    );
  }

  writeValue(value: unknown): void {
    const normalized = value === null || value === undefined ? '' : String(value);
    const el = this.elementRef.nativeElement;
    this.renderer.setValue(el, normalized);
    this.renderer.setAttribute(el, 'value', normalized);
  }

  registerOnChange(fn: (value: string) => void): void {
    this.onChange = fn;
  }

  registerOnTouched(fn: () => void): void {
    this.onTouched = fn;
  }

  setDisabledState(isDisabled: boolean): void {
    this.renderer.setAttribute(
      this.elementRef.nativeElement,
      'sensitive',
      isDisabled ? 'false' : 'true',
    );
  }

  ngOnDestroy(): void {
    for (const unlisten of this.unlistenFns) {
      unlisten();
    }
  }
}

/**
 * Angular `ControlValueAccessor` for boolean native widgets:
 * `<ng-switch>`, `<ng-checkbox>`, `<ng-check>`, `<gtk-switch>`, `<gtk-check-button>`.
 */
@Directive({
  selector:
    'ng-switch[formControlName],ng-switch[formControl],ng-switch[ngModel],ng-checkbox[formControlName],ng-checkbox[formControl],ng-checkbox[ngModel],ng-check[formControlName],ng-check[formControl],ng-check[ngModel]',
  standalone: true,
  providers: [
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => NativeBooleanValueAccessor),
      multi: true,
    },
  ],
})
export class NativeBooleanValueAccessor implements ControlValueAccessor, OnDestroy {
  private readonly renderer = inject(Renderer2);
  private readonly elementRef = inject<ElementRef<NativeElementNode>>(ElementRef);
  private onChange: (value: boolean) => void = () => {};
  private onTouched: () => void = () => {};
  private readonly unlistenFns: Array<() => void> = [];

  constructor() {
    const el = this.elementRef.nativeElement;
    this.unlistenFns.push(
      this.renderer.listen(el, 'toggled', (event: unknown) => {
        const payload = event as NativeWidgetEvent | undefined;
        const active =
          typeof payload?.checked === 'boolean' ? payload.checked : Boolean(payload?.value);
        el.attributes.set('active', active ? 'true' : 'false');
        this.onChange(active);
        this.onTouched();
      }),
    );
  }

  writeValue(value: unknown): void {
    const active = Boolean(value);
    this.renderer.setAttribute(this.elementRef.nativeElement, 'active', active ? 'true' : 'false');
  }

  registerOnChange(fn: (value: boolean) => void): void {
    this.onChange = fn;
  }

  registerOnTouched(fn: () => void): void {
    this.onTouched = fn;
  }

  setDisabledState(isDisabled: boolean): void {
    this.renderer.setAttribute(
      this.elementRef.nativeElement,
      'sensitive',
      isDisabled ? 'false' : 'true',
    );
  }

  ngOnDestroy(): void {
    for (const unlisten of this.unlistenFns) {
      unlisten();
    }
  }
}

/**
 * Angular `ControlValueAccessor` for numeric/index native widgets:
 * `<ng-slider>`, `<ng-scale>`, `<ng-select>`, `<ng-dropdown>`.
 */
@Directive({
  selector:
    'ng-slider[formControlName],ng-slider[formControl],ng-slider[ngModel],ng-scale[formControlName],ng-scale[formControl],ng-scale[ngModel],ng-select[formControlName],ng-select[formControl],ng-select[ngModel],ng-dropdown[formControlName],ng-dropdown[formControl],ng-dropdown[ngModel]',
  standalone: true,
  providers: [
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => NativeNumberValueAccessor),
      multi: true,
    },
  ],
})
export class NativeNumberValueAccessor implements ControlValueAccessor, OnDestroy {
  private readonly renderer = inject(Renderer2);
  private readonly elementRef = inject<ElementRef<NativeElementNode>>(ElementRef);
  private onChange: (value: number) => void = () => {};
  private onTouched: () => void = () => {};
  private readonly unlistenFns: Array<() => void> = [];

  constructor() {
    const el = this.elementRef.nativeElement;
    this.unlistenFns.push(
      this.renderer.listen(el, 'value-changed', (event: unknown) => {
        const payload = event as NativeWidgetEvent | undefined;
        const numValue = Number(payload?.value ?? 0);
        const attrKey = el.tag === 'ng-select' || el.tag === 'ng-dropdown' ? 'selected' : 'value';
        el.attributes.set(attrKey, String(numValue));
        this.onChange(numValue);
        this.onTouched();
      }),
    );
  }

  writeValue(value: unknown): void {
    const numValue = Number(value ?? 0);
    const el = this.elementRef.nativeElement;
    const attrKey = el.tag === 'ng-select' || el.tag === 'ng-dropdown' ? 'selected' : 'value';
    this.renderer.setAttribute(el, attrKey, String(numValue));
  }

  registerOnChange(fn: (value: number) => void): void {
    this.onChange = fn;
  }

  registerOnTouched(fn: () => void): void {
    this.onTouched = fn;
  }

  setDisabledState(isDisabled: boolean): void {
    this.renderer.setAttribute(
      this.elementRef.nativeElement,
      'sensitive',
      isDisabled ? 'false' : 'true',
    );
  }

  ngOnDestroy(): void {
    for (const unlisten of this.unlistenFns) {
      unlisten();
    }
  }
}

export const NATIVE_FORM_DIRECTIVES = [
  NativeTextValueAccessor,
  NativeBooleanValueAccessor,
  NativeNumberValueAccessor,
] as const;

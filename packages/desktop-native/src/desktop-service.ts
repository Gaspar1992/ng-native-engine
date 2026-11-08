import {
  EnvironmentProviders,
  Injectable,
  makeEnvironmentProviders,
  signal,
  WritableSignal,
} from '@angular/core';
import { ensureNgNativeBridge } from '@ng-native/platform';

export type NativeThemeMode = 'dark' | 'light';

export interface OpenFileDialogOptions {
  title?: string;
  defaultPath?: string;
}

export interface SaveFileDialogOptions {
  title?: string;
  suggestedName?: string;
}

/**
 * Injectable Angular service providing reactive access to native Desktop OS capabilities:
 * - System Dark/Light theme (`theme` signal synchronized with `GtkSettings` / `AdwStyleManager`)
 * - Native Clipboard (`GdkClipboard`)
 * - Desktop Notifications (`GNotification`)
 * - Native File Dialogs (`GtkFileDialog` open/save)
 */
@Injectable({ providedIn: 'root' })
export class NativeDesktopService {
  private readonly bridge = ensureNgNativeBridge();
  readonly theme: WritableSignal<NativeThemeMode>;

  constructor() {
    this.theme = signal<NativeThemeMode>(this.bridge.getSystemTheme());
  }

  setTheme(mode: NativeThemeMode): void {
    this.bridge.setSystemTheme(mode);
    this.theme.set(mode);
  }

  toggleTheme(): NativeThemeMode {
    const next: NativeThemeMode = this.theme() === 'dark' ? 'light' : 'dark';
    this.setTheme(next);
    return next;
  }

  getClipboardText(): string {
    return this.bridge.getClipboard();
  }

  setClipboardText(text: string): void {
    this.bridge.setClipboard(text);
  }

  notify(title: string, body: string, icon?: string): void {
    this.bridge.sendNotification(title, body, icon);
  }

  openFileDialog(options?: OpenFileDialogOptions): string {
    return this.bridge.openFileDialog(options?.title, options?.defaultPath);
  }

  saveFileDialog(options?: SaveFileDialogOptions): string {
    return this.bridge.saveFileDialog(options?.title, options?.suggestedName);
  }
}

export function provideNativeDesktop(): EnvironmentProviders {
  return makeEnvironmentProviders([NativeDesktopService]);
}

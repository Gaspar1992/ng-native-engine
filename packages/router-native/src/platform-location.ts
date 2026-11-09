import {
  type LocationChangeEvent,
  type LocationChangeListener,
  PlatformLocation,
} from '@angular/common';
import { type EnvironmentProviders, Injectable, makeEnvironmentProviders } from '@angular/core';
import { provideRouter, type RouterFeatures, type Routes } from '@angular/router';

interface HistoryEntry {
  url: string;
  title: string;
  state: unknown;
}

/**
 * DOM-free Angular `PlatformLocation` implementation for native desktop applications.
 * Maintains an in-memory URL and state history stack so `@angular/router` (`<router-outlet>`,
 * `routerLink`, guards, resolvers, lazy loading) runs natively without `window.location`.
 */
@Injectable({ providedIn: 'root' })
export class NativePlatformLocation extends PlatformLocation {
  private readonly historyStack: HistoryEntry[] = [{ url: '/', title: '', state: null }];
  private currentIndex = 0;
  private readonly popStateListeners = new Set<LocationChangeListener>();
  private readonly hashChangeListeners = new Set<LocationChangeListener>();

  readonly hostname = 'ng-native.local';
  readonly protocol = 'ng-native:';
  readonly port = '';

  override getBaseHrefFromDOM(): string {
    return '/';
  }

  override getState(): unknown {
    return this.currentEntry().state;
  }

  override onPopState(fn: LocationChangeListener): VoidFunction {
    this.popStateListeners.add(fn);
    return () => {
      this.popStateListeners.delete(fn);
    };
  }

  override onHashChange(fn: LocationChangeListener): VoidFunction {
    this.hashChangeListeners.add(fn);
    return () => {
      this.hashChangeListeners.delete(fn);
    };
  }

  override get href(): string {
    return `${this.protocol}//${this.hostname}${this.currentEntry().url}`;
  }

  override get pathname(): string {
    const raw = this.currentEntry().url;
    const qIdx = raw.indexOf('?');
    const hIdx = raw.indexOf('#');
    const end =
      qIdx !== -1 && hIdx !== -1
        ? Math.min(qIdx, hIdx)
        : qIdx !== -1
          ? qIdx
          : hIdx !== -1
            ? hIdx
            : raw.length;
    return raw.slice(0, end) || '/';
  }

  override get search(): string {
    const raw = this.currentEntry().url;
    const qIdx = raw.indexOf('?');
    if (qIdx === -1) {
      return '';
    }
    const hIdx = raw.indexOf('#', qIdx);
    return hIdx === -1 ? raw.slice(qIdx) : raw.slice(qIdx, hIdx);
  }

  override get hash(): string {
    const raw = this.currentEntry().url;
    const hIdx = raw.indexOf('#');
    return hIdx === -1 ? '' : raw.slice(hIdx);
  }

  override replaceState(state: unknown, title: string, url: string): void {
    this.historyStack[this.currentIndex] = {
      url: this.normalizeUrl(url),
      title,
      state,
    };
  }

  override pushState(state: unknown, title: string, url: string): void {
    this.historyStack.splice(this.currentIndex + 1);
    this.historyStack.push({
      url: this.normalizeUrl(url),
      title,
      state,
    });
    this.currentIndex = this.historyStack.length - 1;
  }

  override forward(): void {
    this.historyGo(1);
  }

  override back(): void {
    this.historyGo(-1);
  }

  override historyGo(relativePosition = 0): void {
    if (relativePosition === 0) {
      return;
    }
    const nextIdx = this.currentIndex + relativePosition;
    if (nextIdx < 0 || nextIdx >= this.historyStack.length) {
      return;
    }
    this.currentIndex = nextIdx;
    const evt: LocationChangeEvent = {
      type: 'popstate',
      state: this.currentEntry().state,
    };
    for (const listener of this.popStateListeners) {
      listener(evt);
    }
  }

  private currentEntry(): HistoryEntry {
    return (
      this.historyStack[this.currentIndex] ?? {
        url: '/',
        title: '',
        state: null,
      }
    );
  }

  private normalizeUrl(url: string): string {
    const trimmed = url.trim();
    if (!trimmed) {
      return '/';
    }
    return trimmed.startsWith('/') ? trimmed : `/${trimmed}`;
  }
}

/**
 * Provides `@angular/router` backed by `NativePlatformLocation` for DOM-free native windows.
 */
export function provideNativeRouter(
  routes: Routes,
  ...features: RouterFeatures[]
): EnvironmentProviders {
  return makeEnvironmentProviders([
    { provide: PlatformLocation, useExisting: NativePlatformLocation },
    provideRouter(routes, ...features),
  ]);
}

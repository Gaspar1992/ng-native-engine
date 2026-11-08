import { HttpClient, HttpInterceptorFn, withInterceptors } from '@angular/common/http';
import { TestBed } from '@angular/core/testing';
import { NativeDesktopService } from '@ng-native/desktop';
import { NativeHttpBackend, provideNativeHttpClient } from '@ng-native/http';
import { ensureNgNativeBridge } from '@ng-native/platform';
import { NativePlatformLocation } from '@ng-native/router';
import { firstValueFrom } from 'rxjs';

describe('Phase 5 — @ng-native/router, @ng-native/http & @ng-native/desktop', () => {
  beforeEach(() => {
    const bridge = ensureNgNativeBridge();
    bridge.__drainQueue();
    bridge.setSystemTheme('dark');
    bridge.setClipboard('');
  });

  it('navigates in-memory URL history stack via NativePlatformLocation without window.location', () => {
    const loc = new NativePlatformLocation();
    expect(loc.pathname).toBe('/');

    let popEvents = 0;
    const unlisten = loc.onPopState(() => {
      popEvents += 1;
    });

    loc.pushState({ tab: 'settings' }, 'Settings', '/settings?mode=advanced#audio');
    expect(loc.pathname).toBe('/settings');
    expect(loc.search).toBe('?mode=advanced');
    expect(loc.hash).toBe('#audio');
    expect(loc.getState()).toEqual({ tab: 'settings' });

    loc.pushState({ tab: 'about' }, 'About', '/about');
    expect(loc.pathname).toBe('/about');

    loc.back();
    expect(loc.pathname).toBe('/settings');
    expect(popEvents).toBe(1);

    loc.forward();
    expect(loc.pathname).toBe('/about');
    expect(popEvents).toBe(2);

    unlisten();
  });

  it('executes Angular HttpClient requests with functional interceptors over NativeHttpBackend', async () => {
    const bridge = ensureNgNativeBridge();
    bridge.registerHttpEndpoint(
      'GET',
      'https://api.ng-native.dev/projects?platform=linux',
      200,
      JSON.stringify({ project: 'ng-linux-counter', fps: 60 }),
    );

    let interceptedUrl = '';
    const authInterceptor: HttpInterceptorFn = (req, next) => {
      interceptedUrl = req.urlWithParams;
      return next(req);
    };

    TestBed.configureTestingModule({
      providers: [NativeHttpBackend, provideNativeHttpClient(withInterceptors([authInterceptor]))],
    });
    const http = TestBed.inject(HttpClient);

    const data = await firstValueFrom(
      http.get<{ project: string; fps: number }>(
        'https://api.ng-native.dev/projects?platform=linux',
      ),
    );
    expect(interceptedUrl).toBe('https://api.ng-native.dev/projects?platform=linux');
    expect(data).toEqual({ project: 'ng-linux-counter', fps: 60 });
  });

  it('synchronizes NativeDesktopService theme Signal, clipboard, notifications, and file dialogs', () => {
    const bridge = ensureNgNativeBridge();
    const desktop = new NativeDesktopService();

    expect(desktop.theme()).toBe('dark');
    expect(desktop.toggleTheme()).toBe('light');
    expect(desktop.theme()).toBe('light');
    expect(bridge.getSystemTheme()).toBe('light');

    desktop.setClipboardText('Copied from Angular 21');
    expect(desktop.getClipboardText()).toBe('Copied from Angular 21');

    desktop.notify('Export Ready', 'Saved package successfully');
    expect(bridge.__drainQueue()).toContain('Export Ready: Saved package successfully');

    expect(desktop.openFileDialog({ defaultPath: '/home/dev/config.json' })).toBe(
      '/home/dev/config.json',
    );
    expect(desktop.saveFileDialog({ suggestedName: '/home/dev/bundle.ngb' })).toBe(
      '/home/dev/bundle.ngb',
    );
  });
});

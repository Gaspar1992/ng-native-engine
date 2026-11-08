import { createNativeTargetConfig, executeNativeBuilder } from '@ng-native/builders';
import { parseTemplateHtml } from '../../template-compiler.mjs';

describe('Phase 6 — @ng-native/builders & Distribution Orchestration', () => {
  it('generates angular.json architect.native target configuration with defaults', () => {
    const cfg = createNativeTargetConfig({
      targetPlatform: 'linux-gtk4',
      outputPath: 'dist/ng-linux-counter',
    });

    expect(cfg.builder).toBe('@ng-native/builders:build');
    expect(cfg.options.targetPlatform).toBe('linux-gtk4');
    expect(cfg.options.jsEngine).toBe('jsc');
    expect(cfg.options.packageFormats).toEqual(['deb', 'appimage', 'flatpak']);
  });

  it('executes builder pipeline and produces .deb, AppImage, and Flatpak artifact paths', () => {
    const result = executeNativeBuilder('ng-linux-counter', {
      targetPlatform: 'linux-gtk4',
      jsEngine: 'jsc',
      outputPath: 'dist/ng-linux-counter/',
    });

    expect(result.success).toBe(true);
    expect(result.bundlePath).toBe('dist/ng-linux-counter/main.ngb');
    expect(result.packageArtifacts).toContain('dist/ng-linux-counter/packages/deb/DEBIAN/control');
    expect(result.packageArtifacts).toContain('dist/ng-linux-counter/packages/appimage/AppRun');
    expect(result.packageArtifacts).toContain(
      'dist/ng-linux-counter/packages/flatpak/ng-linux-counter.yml',
    );
  });

  it('compiles Angular 22 templates with @angular/compiler AST including @if blocks and reactive bindings', () => {
    const template = `
      <ng-window id="win" title="{{ windowTitle() }}">
        @if (showBanner) {
          <ng-label id="lbl">Count: {{ count() }}</ng-label>
          <ng-button id="btn" (click)="increment()">+1</ng-button>
        }
      </ng-window>
    `;

    const roots = parseTemplateHtml(
      template,
      { windowTitle: 'AST Demo', showBanner: 'true', count: '5' },
      { count: 5 },
    );

    expect(roots.length).toBe(1);
    expect(roots[0].id).toBe('win');
    expect(roots[0].title).toBe('AST Demo');
    expect(roots[0].children[0].isComment).toBe(true);
    expect(roots[0].children[0].text).toBe('@if(showBanner)');
    expect(roots[0].children[1].id).toBe('lbl');
    expect(roots[0].children[1].reactiveBinding).toEqual({
      signalName: 'count',
      template: 'Count: {}',
    });
    expect(roots[0].children[2].events).toEqual({ click: 'increment' });
  });
});

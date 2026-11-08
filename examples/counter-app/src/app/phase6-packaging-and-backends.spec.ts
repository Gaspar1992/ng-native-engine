import { createNativeTargetConfig, executeNativeBuilder } from '@ng-native/builders';

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
});

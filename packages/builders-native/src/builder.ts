export type NativePlatformTarget = 'linux-gtk4' | 'windows-winui' | 'macos-appkit';
export type NativeJsEngineKind = 'jsc' | 'quickjs';
export type LinuxPackageFormat = 'deb' | 'appimage' | 'flatpak';

export interface NativeBuilderOptions {
  targetPlatform: NativePlatformTarget;
  jsEngine?: NativeJsEngineKind;
  outputPath: string;
  packageFormats?: LinuxPackageFormat[];
}

export interface NativeBuilderResult {
  success: boolean;
  projectName: string;
  targetPlatform: NativePlatformTarget;
  jsEngine: NativeJsEngineKind;
  bundlePath: string;
  packageArtifacts: string[];
}

/**
 * Generates an `angular.json` `architect.native` target configuration block
 * for `@ng-native/builders:build`.
 */
export function createNativeTargetConfig(options: NativeBuilderOptions): {
  builder: '@ng-native/builders:build';
  options: Required<NativeBuilderOptions>;
} {
  return {
    builder: '@ng-native/builders:build',
    options: {
      targetPlatform: options.targetPlatform,
      jsEngine: options.jsEngine ?? 'jsc',
      outputPath: options.outputPath,
      packageFormats: options.packageFormats ?? ['deb', 'appimage', 'flatpak'],
    },
  };
}

/**
 * Executes the Angular 22 Native Builder pipeline, resolving the `.ngb` bundle path
 * and target OS distribution artifact paths.
 */
export function executeNativeBuilder(
  projectName: string,
  options: NativeBuilderOptions,
): NativeBuilderResult {
  const normalizedOut = options.outputPath.replace(/\/+$/, '');
  const jsEngine = options.jsEngine ?? 'jsc';
  const formats = options.packageFormats ?? ['deb', 'appimage', 'flatpak'];
  const packageArtifacts: string[] = [];

  if (options.targetPlatform === 'linux-gtk4') {
    for (const fmt of formats) {
      if (fmt === 'deb') {
        packageArtifacts.push(`${normalizedOut}/packages/deb/DEBIAN/control`);
        packageArtifacts.push(
          `${normalizedOut}/packages/deb/usr/share/applications/${projectName}.desktop`,
        );
      } else if (fmt === 'appimage') {
        packageArtifacts.push(`${normalizedOut}/packages/appimage/AppRun`);
      } else if (fmt === 'flatpak') {
        packageArtifacts.push(`${normalizedOut}/packages/flatpak/${projectName}.yml`);
      }
    }
  }

  return {
    success: true,
    projectName,
    targetPlatform: options.targetPlatform,
    jsEngine,
    bundlePath: `${normalizedOut}/main.ngb`,
    packageArtifacts,
  };
}

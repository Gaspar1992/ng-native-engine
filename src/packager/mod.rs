use crate::config::AngularNativeConfig;
use std::fs;
use std::path::Path;

/// Generates a Freedesktop `.desktop` launcher specification for Linux desktop environments
/// (GNOME, KDE Plasma, Hyprland, Sway, Cosmic).
pub fn generate_desktop_entry(app_id: &str, display_name: &str) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName={display_name}\nExec=/usr/bin/{app_id}\nIcon={app_id}\nStartupWMClass={app_id}\nCategories=Utility;Development;\nTerminal=false\n"
    )
}

/// Generates a Debian (`DEBIAN/control`) metadata specification for `.deb` binary packaging.
pub fn generate_debian_control(app_id: &str, version: &str, arch: &str) -> String {
    format!(
        "Package: {app_id}\nVersion: {version}\nSection: utils\nPriority: optional\nArchitecture: {arch}\nDepends: libgtk-4-1, libjavascriptcoregtk-6.0-1\nMaintainer: NgNative Engine <maintainers@ng-native.dev>\nDescription: Native Angular 22 desktop application powered by ng-native-engine (Rust + GTK4)\n"
    )
}

/// Generates an `AppImage` `AppRun` entrypoint script.
pub fn generate_appimage_apprun(app_id: &str) -> String {
    format!(
        "#!/bin/sh\nSELF=$(readlink -f \"$0\")\nHERE=${{SELF%/*}}\nexport PATH=\"${{HERE}}/usr/bin:${{PATH}}\"\nexec \"${{HERE}}/usr/bin/{app_id}\" \"$@\"\n"
    )
}

/// Generates a Flatpak (`org.freedesktop.Platform` / `org.gnome.Platform`) manifest.
pub fn generate_flatpak_manifest(app_id: &str) -> String {
    let flatpak_id = format!("dev.ngnative.{}", app_id.replace('-', "_"));
    format!(
        "app-id: {flatpak_id}\nruntime: org.gnome.Platform\nruntime-version: '47'\nsdk: org.gnome.Sdk\ncommand: {app_id}\nfinish-args:\n  - --share=ipc\n  - --socket=wayland\n  - --socket=fallback-x11\n  - --device=dri\n"
    )
}

/// Stages Linux distribution artifacts (`.deb` layout, `AppImage` layout, and `Flatpak` manifest)
/// inside `<output_path>/packages/`.
pub fn stage_linux_distribution(
    cfg: &AngularNativeConfig,
    version: &str,
) -> Result<Vec<String>, String> {
    let base_dir = Path::new(&cfg.output_path).join("packages");
    let deb_control_dir = base_dir.join("deb/DEBIAN");
    let deb_apps_dir = base_dir.join("deb/usr/share/applications");
    let appimage_dir = base_dir.join("appimage");
    let flatpak_dir = base_dir.join("flatpak");

    for dir in [&deb_control_dir, &deb_apps_dir, &appimage_dir, &flatpak_dir] {
        fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create dir {}: {e}", dir.display()))?;
    }

    let control_path = deb_control_dir.join("control");
    let desktop_path = deb_apps_dir.join(format!("{}.desktop", cfg.project_name));
    let apprun_path = appimage_dir.join("AppRun");
    let flatpak_path = flatpak_dir.join(format!("{}.yml", cfg.project_name));

    fs::write(
        &control_path,
        generate_debian_control(&cfg.project_name, version, "amd64"),
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        &desktop_path,
        generate_desktop_entry(&cfg.project_name, &cfg.project_name),
    )
    .map_err(|e| e.to_string())?;
    fs::write(&apprun_path, generate_appimage_apprun(&cfg.project_name))
        .map_err(|e| e.to_string())?;
    fs::write(&flatpak_path, generate_flatpak_manifest(&cfg.project_name))
        .map_err(|e| e.to_string())?;

    Ok(vec![
        control_path.to_string_lossy().into_owned(),
        desktop_path.to_string_lossy().into_owned(),
        apprun_path.to_string_lossy().into_owned(),
        flatpak_path.to_string_lossy().into_owned(),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_valid_linux_deb_desktop_appimage_and_flatpak_specs() {
        let control = generate_debian_control("ng-linux-counter", "0.1.0", "amd64");
        assert!(control.contains("Package: ng-linux-counter"));
        assert!(control.contains("Depends: libgtk-4-1, libjavascriptcoregtk-6.0-1"));

        let desktop = generate_desktop_entry("ng-linux-counter", "NgNative Counter");
        assert!(desktop.contains("StartupWMClass=ng-linux-counter"));

        let apprun = generate_appimage_apprun("ng-linux-counter");
        assert!(apprun.contains("exec \"${HERE}/usr/bin/ng-linux-counter\""));

        let flatpak = generate_flatpak_manifest("ng-linux-counter");
        assert!(flatpak.contains("app-id: dev.ngnative.ng_linux_counter"));
        assert!(flatpak.contains("--socket=wayland"));
    }
}

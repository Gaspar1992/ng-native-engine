use std::fs;
use std::process::Command;

/// Loads `dist/<output_path>/main.ngb` if it exists; otherwise automatically compiles
/// the bundle on cold caches using Node.js and `build-bundle.mjs`.
#[allow(dead_code, unreachable_pub)]
pub fn load_or_build_test_bundle(output_path: &str) -> String {
    let bundle_path = format!("{output_path}/main.ngb");
    if let Ok(content) = fs::read_to_string(&bundle_path)
        && !content.trim().is_empty()
    {
        return content;
    }

    let _ = Command::new("node")
        .arg("examples/counter-app/build-bundle.mjs")
        .status();

    fs::read_to_string(&bundle_path)
        .unwrap_or_else(|_| panic!("Failed to load or auto-compile test bundle at '{bundle_path}'"))
}

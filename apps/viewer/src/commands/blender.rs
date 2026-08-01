use crate::commands::notice::report;
use crate::resources::Incoming;

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn convert_pack(inbox: &Incoming, name: &str, files: Vec<(String, Vec<u8>)>) {
    use crate::data::FBX_PACK_SCRIPT;
    use crate::queries::files::find_blender;
    use crate::resources::PendingAsset;

    let Some(blender) = find_blender() else {
        report(
            &inbox.notice,
            "Blender is not on this machine, so a pack of FBX cannot be converted. Install Blender, or name it in NIGHTSHADE_BLENDER.",
        );
        return;
    };

    let label = pack_label(name);
    if let Ok(mut guard) = inbox.loading.lock() {
        *guard = Some(format!("Converting {label} with Blender"));
    }

    let mut crew = inbox.crew.clone();
    let inbox = inbox.clone();
    crate::commands::crew::hire(&mut crew, move || {
        match run(&blender, FBX_PACK_SCRIPT, files) {
            Ok(glb) => {
                if let Ok(mut slot) = inbox.asset.lock() {
                    *slot = Some(PendingAsset::Model(glb));
                }
                if let Ok(mut guard) = inbox.loading.lock() {
                    *guard = Some(label);
                }
            }
            Err(message) => {
                if let Ok(mut guard) = inbox.loading.lock() {
                    *guard = None;
                }
                report(&inbox.notice, message);
            }
        }
    });
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
pub fn convert_pack(inbox: &Incoming, _name: &str, _files: Vec<(String, Vec<u8>)>) {
    report(
        &inbox.notice,
        "A pack of FBX is converted with Blender, which runs only on the desktop viewer.",
    );
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
fn pack_label(name: &str) -> String {
    let stem = name.rsplit('/').next().unwrap_or(name);
    stem.rsplit_once('.')
        .map(|(above, _)| above.to_string())
        .unwrap_or_else(|| stem.to_string())
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
fn run(
    blender: &std::path::Path,
    script: &str,
    files: Vec<(String, Vec<u8>)>,
) -> Result<Vec<u8>, String> {
    let root = std::env::temp_dir().join(format!("nightshade-fbx-{}", std::process::id()));
    let source = root.join("source");
    std::fs::create_dir_all(&source).map_err(|error| error.to_string())?;
    let clean = Scrub(root.clone());

    let mut written = 0;
    for (path, bytes) in &files {
        let stem = path.rsplit('/').next().unwrap_or(path);
        if !stem.to_lowercase().ends_with(".fbx") {
            continue;
        }
        std::fs::write(source.join(stem), bytes).map_err(|error| error.to_string())?;
        written += 1;
    }
    if written == 0 {
        return Err("That pack holds no FBX.".to_string());
    }

    let script_path = root.join("convert.py");
    std::fs::write(&script_path, script).map_err(|error| error.to_string())?;
    let output_path = root.join("pack.glb");

    let output = std::process::Command::new(blender)
        .arg("-b")
        .arg("--factory-startup")
        .arg("--python-exit-code")
        .arg("1")
        .arg("--python")
        .arg(&script_path)
        .arg("--")
        .arg("--source")
        .arg(&source)
        .arg("--output")
        .arg(&output_path)
        .output()
        .map_err(|error| format!("Blender did not start.\n\n{error}"))?;

    if !output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout);
        let tail: Vec<&str> = text.lines().rev().take(12).collect();
        let tail: Vec<&str> = tail.into_iter().rev().collect();
        return Err(format!(
            "Blender did not convert that pack.\n\n{}",
            tail.join("\n")
        ));
    }

    let glb = std::fs::read(&output_path)
        .map_err(|_| "Blender wrote no model for that pack.".to_string())?;
    drop(clean);
    Ok(glb)
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
struct Scrub(std::path::PathBuf);

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
impl Drop for Scrub {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

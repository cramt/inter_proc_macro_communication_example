use std::{env, fs, io, path::PathBuf};

pub fn prepare_runner_and_emit_env() -> io::Result<()> {
    let exe = env::current_exe()?;
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let runner = out_dir.join("my_framework_runner");
    fs::copy(&exe, &runner)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&runner)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&runner, perms)?;
    }
    println!("cargo:rustc-env=MY_FRAMEWORK_RUNNER={}", runner.display());
    Ok(())
}

pub fn run_as_runner() -> std::io::Result<()> {
    use plugin_api::{PluginEntry, inventory};
    use std::io::{Read, Write};

    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    // argv[1] == "--runner"; argv[2] may be the plugin name to select
    let target = std::env::args().nth(2);
    let mut last_err: Option<String> = None;

    for entry in inventory::iter::<PluginEntry> {
        if let Some(name) = target.as_deref() {
            if entry.name != name {
                continue;
            }
        }
        let plugin = (entry.constructor)(); // construct instance
        match plugin.transform(&input) {
            Ok(s) => {
                std::io::stdout().write_all(s.as_bytes())?;
                return Ok(());
            }
            Err(e) => {
                last_err = Some(e);
            }
        }
    }

    let msg = if let Some(name) = target {
        format!("no matching plugin found for '{name}'")
    } else {
        "no plugin succeeded".to_string()
    };
    let err = last_err
        .map(|e| format!("{msg}; last error: {e}"))
        .unwrap_or(msg);
    Err(std::io::Error::new(std::io::ErrorKind::Other, err))
}

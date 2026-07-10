fn main() {
    // M17: Lance's `protoc` feature (Cargo.toml) vendors protobuf-compiler so a
    // system `protoc` is not strictly required for clean builds. This build
    // script no longer panics when protoc is missing — it only emits a warning
    // if neither `PROTOC` nor a system `protoc` is found, since the vendored
    // path is expected to cover the build.
    if std::env::var_os("PROTOC").is_some() || command_exists("protoc") {
        return;
    }

    println!(
        "cargo:warning=nowdocs: no system `protoc` found; relying on the Lance `protoc` feature (vendored protobuf-compiler). \
         If the build fails, install protobuf-compiler (Debian/Ubuntu: apt-get install protobuf-compiler) \
         or set PROTOC to a protoc binary."
    );
}

fn command_exists(cmd: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    let has_exe = cfg!(target_os = "windows") || cmd.ends_with(".exe");
    std::env::split_paths(&paths).any(|dir| {
        if has_exe {
            let candidate_exe = dir.join(format!("{}.exe", cmd));
            if candidate_exe.is_file() {
                return true;
            }
        }
        let candidate = dir.join(cmd);
        candidate.is_file()
    })
}

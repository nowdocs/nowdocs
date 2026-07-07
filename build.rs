fn main() {
    if std::env::var_os("PROTOC").is_some() || command_exists("protoc") {
        return;
    }

    panic!(
        "nowdocs requires protoc to build LanceDB dependencies. \
         Install protobuf-compiler (Debian/Ubuntu: apt-get install protobuf-compiler) \
         or set PROTOC to a protoc binary before running cargo."
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

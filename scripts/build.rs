use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // Read version from Cargo.toml
    let version = get_version();
    // Get short git commit hash
    let git_hash = get_git_hash();

    let targets = vec![
        ("x86_64-unknown-linux-gnu", "snix-linux"),
        ("x86_64-pc-windows-gnu", "snix-windows.exe"),
    ];

    let out_dir = "release-builds";
    fs::create_dir_all(out_dir).expect("Failed to create output directory");

    // Detect if 'cross' is available
    let use_cross = Command::new("cross")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let build_tool = if use_cross { "cross" } else { "cargo" };
    println!("Using build tool: {}", build_tool);

    for (target, base_name) in &targets {
        println!("Building for target: {}", target);
        let status = Command::new(build_tool)
            .args(["build", "--release", "--target", target])
            .status()
            .expect(&format!("Failed to run {} build", build_tool));
        if !status.success() {
            panic!("Build failed for target: {}", target);
        }
        let bin_name = if target.contains("windows") {
            "snix.exe"
        } else {
            "snix"
        };
        let dest_path = format!(
            "{}/{}-{}-git{}",
            out_dir,
            base_name.trim_end_matches(if target.contains("windows") {
                ".exe"
            } else {
                ""
            }),
            version,
            git_hash
        );
        let dest_path = if target.contains("windows") {
            format!("{}.exe", dest_path)
        } else {
            dest_path
        };
        let src_path = format!("target/{}/release/{}", target, bin_name);
        fs::copy(&src_path, &dest_path)
            .expect(&format!("Failed to copy {} to {}", src_path, dest_path));
        println!("Copied {} to {}", src_path, dest_path);
    }

    // Optionally, zip the outputs
    #[cfg(not(windows))]
    {
        let zip_files: Vec<String> = targets
            .iter()
            .map(|(_, base_name)| {
                let mut name = format!(
                    "{}/{}-{}-git{}",
                    out_dir,
                    base_name.trim_end_matches(if base_name.ends_with(".exe") {
                        ".exe"
                    } else {
                        ""
                    }),
                    version,
                    git_hash
                );
                if base_name.ends_with(".exe") {
                    name.push_str(".exe");
                }
                name
            })
            .collect();
        let zip_name = format!("{}/snix-binaries-{}-git{}.zip", out_dir, version, git_hash);
        let mut args = vec!["-j", &zip_name];
        args.extend(zip_files.iter().map(|s| s.as_str()));
        let zip_status = Command::new("zip")
            .args(&args)
            .status()
            .expect("Failed to zip binaries");
        if !zip_status.success() {
            eprintln!("Warning: Failed to zip binaries");
        }
    }

    println!(
        "All builds complete. Binaries are in the '{}' directory.",
        out_dir
    );
}

fn get_version() -> String {
    let cargo = std::fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");
    for line in cargo.lines() {
        if line.trim_start().starts_with("version = ") {
            let v = line.split('=').nth(1).unwrap().trim().trim_matches('"');
            return v.to_string();
        }
    }
    panic!("Could not find version in Cargo.toml");
}

fn get_git_hash() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .expect("Failed to get git hash");
    if output.status.success() {
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    } else {
        "unknown".to_string()
    }
}

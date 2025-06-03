use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let targets = vec![
        ("x86_64-unknown-linux-gnu", "snix-linux"),
        ("x86_64-pc-windows-gnu", "snix-windows.exe"),
    ];

    let out_dir = "release-builds";
    fs::create_dir_all(out_dir).expect("Failed to create output directory");

    // Detect if 'cross' is available
    let use_cross = Command::new("cross").arg("--version").output().map(|o| o.status.success()).unwrap_or(false);
    let build_tool = if use_cross { "cross" } else { "cargo" };
    println!("Using build tool: {}", build_tool);

    for (target, out_name) in &targets {
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
        let src_path = format!("target/{}/release/{}", target, bin_name);
        let dest_path = format!("{}/{}", out_dir, out_name);
        fs::copy(&src_path, &dest_path)
            .expect(&format!("Failed to copy {} to {}", src_path, dest_path));
        println!("Copied {} to {}", src_path, dest_path);
    }

    // Optionally, zip the outputs
    #[cfg(not(windows))]
    {
        let zip_status = Command::new("zip")
            .args([
                "-j",
                &format!("{}/snix-binaries.zip", out_dir),
                &format!("{}/snix-linux", out_dir),
                &format!("{}/snix-windows.exe", out_dir),
            ])
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

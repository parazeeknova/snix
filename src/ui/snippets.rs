/// Get syntax-highlighted content using bat command
fn get_bat_highlighted_content(snippet: &CodeSnippet, app: &App) -> Option<String> {
    if let Some(ref storage) = app.storage_manager {
        let file_path = storage.get_snippet_file_path(snippet);

        // Save current content to file for bat to read
        if storage.save_snippet_content(snippet).is_err() {
            return None;
        }

        // Run bat command with appropriate options
        let output = std::process::Command::new("bat")
            .arg("--color=always")
            .arg("--style=numbers,grid")
            .arg("--theme=TwoDark") // Nice dark theme that works well with our Rose Pine
            .arg("--line-range=:50") // Limit to first 50 lines for preview
            .arg("--terminal-width=80") // Consistent width
            .arg(&file_path)
            .output();

        match output {
            Ok(output) if output.status.success() => String::from_utf8(output.stdout).ok(),
            _ => None,
        }
    } else {
        None
    }
}

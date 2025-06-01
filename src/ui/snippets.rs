// This is a earlier implementation of syntax highlighting it will soon be removed in future refactor if any..
fn get_bat_highlighted_content(snippet: &CodeSnippet, app: &App) -> Option<String> {
    if let Some(ref storage) = app.storage_manager {
        let file_path = storage.get_snippet_file_path(snippet);

        if storage.save_snippet_content(snippet).is_err() {
            return None;
        }

        let output = std::process::Command::new("bat")
            .arg("--color=always")
            .arg("--style=numbers,grid")
            .arg("--theme=OneHalfLight")
            .arg("--line-range=:1000")
            .arg("--terminal-width=80")
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

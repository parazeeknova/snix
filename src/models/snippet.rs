use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSnippet {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub content: String,
    pub language: SnippetLanguage,
    pub notebook_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub accessed_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub is_favorite: bool,
    pub use_count: u32,
    pub file_extension: String,
    pub metadata: HashMap<String, String>,
    pub version: u32,
    pub syntax_theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SnippetLanguage {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Go,
    Java,
    C,
    Cpp,
    CSharp,
    PHP,
    Ruby,
    Swift,
    Kotlin,
    Dart,
    HTML,
    CSS,
    SCSS,
    SQL,
    Bash,
    PowerShell,
    Yaml,
    Json,
    Xml,
    Markdown,
    Dockerfile,
    Toml,
    Ini,
    Config,
    Text,
    Other(String),
}

impl SnippetLanguage {
    /// Get file extension for the language
    pub fn file_extension(&self) -> &'static str {
        match self {
            SnippetLanguage::Rust => "rs",
            SnippetLanguage::JavaScript => "js",
            SnippetLanguage::TypeScript => "ts",
            SnippetLanguage::Python => "py",
            SnippetLanguage::Go => "go",
            SnippetLanguage::Java => "java",
            SnippetLanguage::C => "c",
            SnippetLanguage::Cpp => "cpp",
            SnippetLanguage::CSharp => "cs",
            SnippetLanguage::PHP => "php",
            SnippetLanguage::Ruby => "rb",
            SnippetLanguage::Swift => "swift",
            SnippetLanguage::Kotlin => "kt",
            SnippetLanguage::Dart => "dart",
            SnippetLanguage::HTML => "html",
            SnippetLanguage::CSS => "css",
            SnippetLanguage::SCSS => "scss",
            SnippetLanguage::SQL => "sql",
            SnippetLanguage::Bash => "sh",
            SnippetLanguage::PowerShell => "ps1",
            SnippetLanguage::Yaml => "yaml",
            SnippetLanguage::Json => "json",
            SnippetLanguage::Xml => "xml",
            SnippetLanguage::Markdown => "md",
            SnippetLanguage::Dockerfile => "dockerfile",
            SnippetLanguage::Toml => "toml",
            SnippetLanguage::Ini => "ini",
            SnippetLanguage::Config => "conf",
            SnippetLanguage::Text => "txt",
            SnippetLanguage::Other(ext) => {
                // Return the extension as is for custom types
                // This is just a reference to a static string, so it will leak
                // but it's not a big deal for this application
                Box::leak(ext.clone().into_boxed_str())
            }
        }
    }

    /// Get language from file extension
    pub fn _from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => SnippetLanguage::Rust,
            "js" => SnippetLanguage::JavaScript,
            "ts" => SnippetLanguage::TypeScript,
            "py" => SnippetLanguage::Python,
            "go" => SnippetLanguage::Go,
            "java" => SnippetLanguage::Java,
            "c" => SnippetLanguage::C,
            "cpp" | "cc" | "cxx" => SnippetLanguage::Cpp,
            "cs" => SnippetLanguage::CSharp,
            "php" => SnippetLanguage::PHP,
            "rb" => SnippetLanguage::Ruby,
            "swift" => SnippetLanguage::Swift,
            "kt" => SnippetLanguage::Kotlin,
            "dart" => SnippetLanguage::Dart,
            "html" | "htm" => SnippetLanguage::HTML,
            "css" => SnippetLanguage::CSS,
            "scss" => SnippetLanguage::SCSS,
            "sql" => SnippetLanguage::SQL,
            "sh" => SnippetLanguage::Bash,
            "ps1" => SnippetLanguage::PowerShell,
            "yml" | "yaml" => SnippetLanguage::Yaml,
            "json" => SnippetLanguage::Json,
            "xml" => SnippetLanguage::Xml,
            "md" => SnippetLanguage::Markdown,
            "dockerfile" => SnippetLanguage::Dockerfile,
            "toml" => SnippetLanguage::Toml,
            "ini" => SnippetLanguage::Ini,
            "conf" | "config" => SnippetLanguage::Config,
            "txt" => SnippetLanguage::Text,
            _ => SnippetLanguage::Other(ext.to_string()),
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            SnippetLanguage::Rust => "Rust",
            SnippetLanguage::JavaScript => "JavaScript",
            SnippetLanguage::TypeScript => "TypeScript",
            SnippetLanguage::Python => "Python",
            SnippetLanguage::Go => "Go",
            SnippetLanguage::Java => "Java",
            SnippetLanguage::C => "C",
            SnippetLanguage::Cpp => "C++",
            SnippetLanguage::CSharp => "C#",
            SnippetLanguage::PHP => "PHP",
            SnippetLanguage::Ruby => "Ruby",
            SnippetLanguage::Swift => "Swift",
            SnippetLanguage::Kotlin => "Kotlin",
            SnippetLanguage::Dart => "Dart",
            SnippetLanguage::HTML => "HTML",
            SnippetLanguage::CSS => "CSS",
            SnippetLanguage::SCSS => "SCSS",
            SnippetLanguage::SQL => "SQL",
            SnippetLanguage::Bash => "Bash",
            SnippetLanguage::PowerShell => "PowerShell",
            SnippetLanguage::Yaml => "YAML",
            SnippetLanguage::Json => "JSON",
            SnippetLanguage::Xml => "XML",
            SnippetLanguage::Markdown => "Markdown",
            SnippetLanguage::Dockerfile => "Dockerfile",
            SnippetLanguage::Toml => "TOML",
            SnippetLanguage::Ini => "INI",
            SnippetLanguage::Config => "Config",
            SnippetLanguage::Text => "Text",
            SnippetLanguage::Other(name) => name,
        }
    }

    /// Get icon for the language
    pub fn icon(&self) -> &'static str {
        match self {
            SnippetLanguage::Rust => "",
            SnippetLanguage::JavaScript => "",
            SnippetLanguage::TypeScript => "",
            SnippetLanguage::Python => "",
            SnippetLanguage::Go => "󰟓",
            SnippetLanguage::Java => "",
            SnippetLanguage::C => "",
            SnippetLanguage::Cpp => "",
            SnippetLanguage::CSharp => "",
            SnippetLanguage::PHP => "",
            SnippetLanguage::Ruby => "",
            SnippetLanguage::Swift => "",
            SnippetLanguage::Kotlin => "",
            SnippetLanguage::Dart => "",
            SnippetLanguage::HTML => "",
            SnippetLanguage::CSS => "",
            SnippetLanguage::SCSS => "󰟬",
            SnippetLanguage::SQL => "",
            SnippetLanguage::Bash => "",
            SnippetLanguage::PowerShell => "",
            SnippetLanguage::Yaml => "",
            SnippetLanguage::Json => "",
            SnippetLanguage::Xml => "󰗀",
            SnippetLanguage::Markdown => "",
            SnippetLanguage::Dockerfile => "",
            SnippetLanguage::Toml => "",
            SnippetLanguage::Ini => "",
            SnippetLanguage::Config => "",
            SnippetLanguage::Text => "",
            SnippetLanguage::Other(_) => "",
        }
    }

    /// Get short name for the language
    pub fn short_name(&self) -> &'static str {
        match self {
            SnippetLanguage::Rust => "Rust",
            SnippetLanguage::JavaScript => "JS",
            SnippetLanguage::TypeScript => "TS",
            SnippetLanguage::Python => "Py",
            SnippetLanguage::Go => "Go",
            SnippetLanguage::Java => "Java",
            SnippetLanguage::C => "C",
            SnippetLanguage::Cpp => "C++",
            SnippetLanguage::CSharp => "C#",
            SnippetLanguage::PHP => "PHP",
            SnippetLanguage::Ruby => "Ruby",
            SnippetLanguage::Swift => "Swift",
            SnippetLanguage::Kotlin => "Kotlin",
            SnippetLanguage::Dart => "Dart",
            SnippetLanguage::HTML => "HTML",
            SnippetLanguage::CSS => "CSS",
            SnippetLanguage::SCSS => "SCSS",
            SnippetLanguage::SQL => "SQL",
            SnippetLanguage::Bash => "Bash",
            SnippetLanguage::PowerShell => "PS",
            SnippetLanguage::Yaml => "YAML",
            SnippetLanguage::Json => "JSON",
            SnippetLanguage::Xml => "XML",
            SnippetLanguage::Markdown => "MD",
            SnippetLanguage::Dockerfile => "Docker",
            SnippetLanguage::Toml => "TOML",
            SnippetLanguage::Ini => "INI",
            SnippetLanguage::Config => "Conf",
            SnippetLanguage::Text => "Text",
            SnippetLanguage::Other(name) => Box::leak(name.clone().into_boxed_str()),
        }
    }
}

impl CodeSnippet {
    pub fn new(title: String, language: SnippetLanguage, notebook_id: Uuid) -> Self {
        let now = Utc::now();
        let file_extension = language.file_extension().to_string();

        Self {
            id: Uuid::new_v4(),
            title,
            description: None,
            content: String::new(),
            language,
            notebook_id,
            created_at: now,
            updated_at: now,
            accessed_at: now,
            tags: Vec::new(),
            is_favorite: false,
            use_count: 0,
            file_extension,
            metadata: HashMap::new(),
            version: 1,
            syntax_theme: String::from("base16-ocean.dark"),
        }
    }

    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
        self.version += 1;
    }

    pub fn mark_accessed(&mut self) {
        self.accessed_at = Utc::now();
        self.use_count += 1;
    }

    pub fn get_preview(&self, _max_lines: usize) -> String {
        self.content.clone()
    }

    pub fn get_line_count(&self) -> usize {
        self.content.lines().count()
    }
}

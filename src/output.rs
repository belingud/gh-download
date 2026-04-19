use crate::cli::ResolvedOptions;
use crate::download::format_remote_path;
use crate::error::UserFacingError;
use crate::i18n::Language;

#[derive(Debug, Clone)]
pub struct Output {
    color: bool,
    language: Language,
}

impl Output {
    pub fn new(color: bool, language: Language) -> Self {
        Self { color, language }
    }

    pub fn startup(&self, options: &ResolvedOptions) {
        println!("{}", self.paint("34", "● gh-download"));
        println!(
            "{} {}",
            self.paint("34", self.label_repository()),
            options.repo
        );
        println!(
            "{} {}",
            self.paint("34", self.label_ref()),
            options
                .git_ref
                .as_deref()
                .unwrap_or(self.default_ref_label())
        );
        println!(
            "{} {}",
            self.paint("34", self.label_remote()),
            format_remote_path(&options.remote_path)
        );
        println!(
            "{} {}",
            self.paint("34", self.label_local()),
            options.local_target.display()
        );
        println!();
    }

    pub fn scan_directory(&self) {
        println!(
            "{} {}",
            self.paint("33", "↻"),
            self.message_scanning_directory()
        );
    }

    pub fn found_files(&self, count: usize) {
        println!(
            "{} {}",
            self.paint("34", "ℹ"),
            self.message_found_files(count)
        );
    }

    pub fn downloading(&self, path: &str) {
        println!("{} {}", self.paint("34", "↓"), path);
    }

    pub fn warning(&self, message: &str) {
        println!("{} {}", self.paint("33", "⚠"), message);
    }

    pub fn success(&self, message: &str) {
        println!("{} {}", self.paint("32", "✔"), message);
    }

    pub fn print_user_error(&self, error: &UserFacingError) {
        eprintln!("{}", self.paint("31", &error.title));
        eprintln!("{} {}", self.reason_label(), error.reason);
        if !error.suggestions.is_empty() {
            eprintln!("{}", self.suggestions_label());
            for suggestion in &error.suggestions {
                eprintln!("- {}", suggestion);
            }
        }
    }

    fn label_repository(&self) -> &'static str {
        if self.language.is_chinese() {
            "仓库"
        } else {
            "Repository"
        }
    }

    fn label_ref(&self) -> &'static str {
        if self.language.is_chinese() {
            "引用"
        } else {
            "Ref"
        }
    }

    fn default_ref_label(&self) -> &'static str {
        if self.language.is_chinese() {
            "默认分支"
        } else {
            "default branch"
        }
    }

    fn label_remote(&self) -> &'static str {
        if self.language.is_chinese() {
            "远端"
        } else {
            "Remote"
        }
    }

    fn label_local(&self) -> &'static str {
        if self.language.is_chinese() {
            "本地"
        } else {
            "Local"
        }
    }

    fn reason_label(&self) -> &'static str {
        if self.language.is_chinese() {
            "原因："
        } else {
            "Reason:"
        }
    }

    fn suggestions_label(&self) -> &'static str {
        if self.language.is_chinese() {
            "建议："
        } else {
            "Suggestions:"
        }
    }

    fn message_scanning_directory(&self) -> &'static str {
        if self.language.is_chinese() {
            "正在读取目录结构..."
        } else {
            "Reading directory structure..."
        }
    }

    fn message_found_files(&self, count: usize) -> String {
        if self.language.is_chinese() {
            format!("发现 {} 个文件", count)
        } else {
            format!("Found {} files", count)
        }
    }

    fn paint(&self, code: &str, text: &str) -> String {
        if self.color {
            format!("\u{1b}[{}m{}\u{1b}[0m", code, text)
        } else {
            text.to_string()
        }
    }
}

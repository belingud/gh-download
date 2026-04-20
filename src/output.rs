use crate::cli::ResolvedOptions;
use crate::download::format_remote_path;
use crate::error::UserFacingError;
use crate::i18n::Language;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Output {
    color: bool,
    language: Language,
    io_lock: Arc<Mutex<()>>,
}

impl Output {
    pub fn new(color: bool, language: Language) -> Self {
        Self {
            color,
            language,
            io_lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn startup(&self, options: &ResolvedOptions) {
        self.with_io_lock(|| {
            self.print_separator();
            println!(
                "{}{}",
                self.paint("34", self.label_repository()),
                options.repo
            );
            println!(
                "{}{}",
                self.paint("32", self.label_ref()),
                options
                    .git_ref
                    .as_deref()
                    .unwrap_or(self.default_ref_label())
            );
            println!(
                "{}{}",
                self.paint("33", self.label_remote()),
                format_remote_path(&options.remote_path)
            );
            println!(
                "{}{}",
                self.paint("35", self.label_local()),
                options.local_target.display()
            );
            self.print_separator();
        });
    }

    pub fn found_directory(&self, count: usize, remote_path: &str) {
        self.print_stdout_line(&self.message_found_files(count, remote_path));
    }

    pub fn created_directory(&self, path: &Path) {
        self.with_io_lock(|| {
            println!(
                "{}{}",
                self.paint("34", self.label_created_directory()),
                path.display()
            );
            self.print_separator();
        });
    }

    pub fn downloading(&self, path: &str) {
        self.print_stdout_line(&format!(
            "{}{}",
            self.paint("34", self.label_downloading()),
            path
        ));
    }

    pub fn warning(&self, message: &str) {
        self.print_stdout_line(&format!("{} {}", self.paint("33", "⚠"), message));
    }

    pub fn success(&self, message: &str) {
        self.print_stdout_line(&format!("{} {}", self.paint("32", "✔"), message));
    }

    pub fn skipping_existing(&self, path: &str) {
        self.print_stdout_line(&self.message_skipping_existing(path));
    }

    pub fn completion(
        &self,
        repo: &str,
        remote_path: &str,
        saved_path: &Path,
        files_downloaded: usize,
        skipped_existing_files: usize,
        skipped_entries: usize,
    ) {
        self.with_io_lock(|| {
            self.print_separator();
            println!("{}", self.message_completion(repo, remote_path, saved_path));
            if files_downloaded > 1 || skipped_existing_files > 0 || skipped_entries > 0 {
                println!(
                    "{}",
                    self.message_download_stats(
                        files_downloaded,
                        skipped_existing_files,
                        skipped_entries,
                    )
                );
            }
        });
    }

    pub fn print_user_error(&self, error: &UserFacingError) {
        self.with_io_lock(|| {
            eprintln!("{}", self.paint("31", &error.title));
            eprintln!("{} {}", self.reason_label(), error.reason);
            if !error.suggestions.is_empty() {
                eprintln!("{}", self.suggestions_label());
                for suggestion in &error.suggestions {
                    eprintln!("- {}", suggestion);
                }
            }
        });
    }

    pub fn debug_line(&self, message: &str) {
        self.with_io_lock(|| eprintln!("{}", message));
    }

    fn label_repository(&self) -> &'static str {
        if self.language.is_chinese() {
            "📦 仓库："
        } else {
            "📦 Repository:"
        }
    }

    fn label_ref(&self) -> &'static str {
        if self.language.is_chinese() {
            "🌿 分支："
        } else {
            "🌿 Ref:"
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
            "📂 远端路径："
        } else {
            "📂 Remote Path:"
        }
    }

    fn label_local(&self) -> &'static str {
        if self.language.is_chinese() {
            "💾 本地路径："
        } else {
            "💾 Local Path:"
        }
    }

    fn label_created_directory(&self) -> &'static str {
        if self.language.is_chinese() {
            "📁 创建本地目录："
        } else {
            "📁 Created Local Directory:"
        }
    }

    fn label_downloading(&self) -> &'static str {
        if self.language.is_chinese() {
            "⬇️ 下载："
        } else {
            "⬇️ Download:"
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

    fn message_found_files(&self, count: usize, remote_path: &str) -> String {
        if self.language.is_chinese() {
            format!(
                "{} 发现 {} 个文件，目录：{}",
                self.paint("33", "🔎"),
                count,
                format_remote_path(remote_path)
            )
        } else {
            format!(
                "{} Found {} files in directory: {}",
                self.paint("33", "🔎"),
                count,
                format_remote_path(remote_path)
            )
        }
    }

    fn message_completion(&self, repo: &str, remote_path: &str, saved_path: &Path) -> String {
        if self.language.is_chinese() {
            format!(
                "{} 完成：{} 的 {} 已保存到 {}",
                self.paint("32", "✅"),
                repo,
                format_remote_path(remote_path),
                saved_path.display()
            )
        } else {
            format!(
                "{} Done: {} {} saved to {}",
                self.paint("32", "✅"),
                repo,
                format_remote_path(remote_path),
                saved_path.display()
            )
        }
    }

    fn message_download_stats(
        &self,
        files_downloaded: usize,
        skipped_existing_files: usize,
        skipped_entries: usize,
    ) -> String {
        if self.language.is_chinese() {
            format!(
                "共下载 {} 个文件，跳过 {} 个已存在文件，跳过 {} 个不支持条目",
                files_downloaded, skipped_existing_files, skipped_entries
            )
        } else {
            format!(
                "Downloaded {} files, skipped {} existing files, skipped {} unsupported entries",
                files_downloaded, skipped_existing_files, skipped_entries
            )
        }
    }

    fn message_skipping_existing(&self, path: &str) -> String {
        if self.language.is_chinese() {
            format!("{} 跳过已存在文件：{}", self.paint("33", "⏭"), path)
        } else {
            format!("{} Skipping existing file: {}", self.paint("33", "⏭"), path)
        }
    }

    fn print_separator(&self) {
        println!(
            "{}",
            self.paint("90", "-------------------------------------")
        );
    }

    fn print_stdout_line(&self, message: &str) {
        self.with_io_lock(|| println!("{}", message));
    }

    fn with_io_lock<T>(&self, action: impl FnOnce() -> T) -> T {
        let _guard = self.io_lock.lock().expect("output lock");
        action()
    }

    fn paint(&self, code: &str, text: &str) -> String {
        if self.color {
            format!("\u{1b}[{}m{}\u{1b}[0m", code, text)
        } else {
            text.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn chinese_found_directory_mentions_count_and_remote_path() {
        let output = Output::new(false, Language::Zh);

        assert_eq!(
            output.message_found_files(11, "skills/baoyu-translate"),
            "🔎 发现 11 个文件，目录：skills/baoyu-translate"
        );
    }

    #[test]
    fn chinese_completion_mentions_repo_remote_path_and_saved_path() {
        let output = Output::new(false, Language::Zh);

        assert_eq!(
            output.message_completion(
                "jimliu/baoyu-skills",
                "skills/baoyu-translate",
                Path::new("/tmp/baoyu-translate")
            ),
            "✅ 完成：jimliu/baoyu-skills 的 skills/baoyu-translate 已保存到 /tmp/baoyu-translate"
        );
    }

    #[test]
    fn english_download_stats_distinguish_existing_and_unsupported_skips() {
        let output = Output::new(false, Language::En);

        assert_eq!(
            output.message_download_stats(3, 2, 1),
            "Downloaded 3 files, skipped 2 existing files, skipped 1 unsupported entries"
        );
    }

    #[test]
    fn chinese_skip_existing_message_mentions_file_path() {
        let output = Output::new(false, Language::Zh);

        assert_eq!(
            output.message_skipping_existing("README.md"),
            "⏭ 跳过已存在文件：README.md"
        );
    }
}

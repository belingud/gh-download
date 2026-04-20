use crate::cli::ResolvedOptions;
use crate::download::DownloadStats;
use crate::download::format_remote_path;
use crate::error::UserFacingError;
use crate::i18n::Language;
use serde::Serialize;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Output {
    color: bool,
    language: Language,
    json_mode: bool,
    io_lock: Arc<Mutex<()>>,
}

impl Output {
    pub fn new(color: bool, language: Language) -> Self {
        Self {
            color,
            language,
            json_mode: false,
            io_lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn with_json_mode(mut self) -> Self {
        self.json_mode = true;
        self
    }

    pub fn startup(&self, options: &ResolvedOptions) {
        if self.json_mode {
            return;
        }

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

    pub fn found_directory(&self, count: usize, remote_path: &str, worker_threads: usize) {
        if self.json_mode {
            return;
        }

        self.print_stdout_line(&self.message_found_files(count, remote_path, worker_threads));
    }

    pub fn created_directory(&self, path: &Path) {
        if self.json_mode {
            return;
        }

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
        if self.json_mode {
            return;
        }

        self.print_stdout_line(&format!(
            "{}{}",
            self.paint("34", self.label_downloading()),
            path
        ));
    }

    pub fn warning(&self, message: &str) {
        if self.json_mode {
            return;
        }

        self.print_stdout_line(&format!("{} {}", self.paint("33", "⚠"), message));
    }

    pub fn success(&self, message: &str) {
        if self.json_mode {
            return;
        }

        self.print_stdout_line(&format!("{} {}", self.paint("32", "✔"), message));
    }

    pub fn skipping_existing(&self, path: &str) {
        if self.json_mode {
            return;
        }

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
        if self.json_mode {
            return;
        }

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
        if self.json_mode {
            return;
        }

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

    pub fn print_json_success(&self, saved_path: &Path, stats: &DownloadStats) {
        self.print_stdout_line(&self.render_json_success(saved_path, stats));
    }

    pub fn print_json_error(&self, error: &UserFacingError) {
        self.print_stdout_line(&self.render_json_error(error));
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

    fn message_found_files(
        &self,
        count: usize,
        remote_path: &str,
        worker_threads: usize,
    ) -> String {
        if self.language.is_chinese() {
            format!(
                "{} 发现 {} 个文件，目录：{}，使用 {} 个线程",
                self.paint("33", "🔎"),
                count,
                format_remote_path(remote_path),
                worker_threads
            )
        } else {
            format!(
                "{} Found {} files in directory: {} using {} threads",
                self.paint("33", "🔎"),
                count,
                format_remote_path(remote_path),
                worker_threads
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

    fn render_json_success(&self, saved_path: &Path, stats: &DownloadStats) -> String {
        serde_json::to_string_pretty(&JsonSuccessPayload {
            success: true,
            saved_path: saved_path.display().to_string(),
            stats: JsonStats {
                files_downloaded: stats.files_downloaded,
                skipped_existing_files: stats.skipped_existing_files,
                skipped_unsupported_entries: stats.skipped_entries,
            },
        })
        .expect("json success payload")
    }

    fn render_json_error(&self, error: &UserFacingError) -> String {
        serde_json::to_string_pretty(&JsonFailurePayload {
            success: false,
            error: JsonErrorPayload {
                title: error.title.clone(),
                reason: error.reason.clone(),
                suggestions: error.suggestions.clone(),
            },
        })
        .expect("json failure payload")
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

#[derive(Debug, Serialize, PartialEq, Eq)]
struct JsonSuccessPayload {
    success: bool,
    saved_path: String,
    stats: JsonStats,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct JsonStats {
    files_downloaded: usize,
    skipped_existing_files: usize,
    skipped_unsupported_entries: usize,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct JsonFailurePayload {
    success: bool,
    error: JsonErrorPayload,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct JsonErrorPayload {
    title: String,
    reason: String,
    suggestions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::Path;
    use std::process::Command;

    use serde_json::Value;

    use super::*;

    #[test]
    fn chinese_found_directory_mentions_count_and_remote_path() {
        let output = Output::new(false, Language::Zh);

        assert_eq!(
            output.message_found_files(11, "skills/baoyu-translate", 4),
            "🔎 发现 11 个文件，目录：skills/baoyu-translate，使用 4 个线程"
        );
    }

    #[test]
    fn english_found_directory_mentions_count_remote_path_and_threads() {
        let output = Output::new(false, Language::En);

        assert_eq!(
            output.message_found_files(3, "src", 3),
            "🔎 Found 3 files in directory: src using 3 threads"
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

    #[test]
    fn json_success_payload_includes_saved_path_and_stats() {
        let output = Output::new(false, Language::En).with_json_mode();
        let stats = DownloadStats {
            files_downloaded: 2,
            skipped_existing_files: 1,
            skipped_entries: 3,
        };

        let value: Value = serde_json::from_str(
            &output.render_json_success(Path::new("/tmp/downloads/src"), &stats),
        )
        .expect("valid json");

        assert_eq!(value["success"], true);
        assert_eq!(value["saved_path"], "/tmp/downloads/src");
        assert_eq!(value["stats"]["files_downloaded"], 2);
        assert_eq!(value["stats"]["skipped_existing_files"], 1);
        assert_eq!(value["stats"]["skipped_unsupported_entries"], 3);
    }

    #[test]
    fn json_failure_payload_includes_classified_error_fields() {
        let output = Output::new(false, Language::En).with_json_mode();
        let error = UserFacingError {
            title: "✖ Download failed".to_string(),
            reason: "boom".to_string(),
            suggestions: vec!["retry".to_string()],
        };

        let value: Value =
            serde_json::from_str(&output.render_json_error(&error)).expect("valid json");

        assert_eq!(value["success"], false);
        assert_eq!(value["error"]["title"], "✖ Download failed");
        assert_eq!(value["error"]["reason"], "boom");
        assert_eq!(value["error"]["suggestions"][0], "retry");
    }

    #[test]
    fn json_mode_keeps_debug_on_stderr_and_suppresses_human_stdout() {
        let output = Command::new(std::env::current_exe().expect("current test binary"))
            .arg("output::tests::json_mode_keeps_debug_on_stderr_and_suppresses_human_stdout_subprocess")
            .arg("--exact")
            .arg("--nocapture")
            .env("GH_DOWNLOAD_JSON_DEBUG_SUBPROCESS", "1")
            .output()
            .expect("run json/debug subprocess test");

        assert!(
            output.status.success(),
            "json/debug subprocess should pass: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let json = extract_json_document(&stdout).expect("stdout should contain json");
        let value: Value = serde_json::from_str(json).expect("stdout json should parse");

        assert_eq!(value["success"], true);
        assert_eq!(value["saved_path"], "/tmp/output.json");
        assert!(!stdout.contains("Repository:"));
        assert!(!stdout.contains("Download:"));
        assert!(!stdout.contains("[debug]"));
        assert!(stderr.contains("[debug] token-source: GITHUB_TOKEN"));
    }

    #[test]
    fn json_mode_keeps_debug_on_stderr_and_suppresses_human_stdout_subprocess() {
        if env::var("GH_DOWNLOAD_JSON_DEBUG_SUBPROCESS").as_deref() != Ok("1") {
            return;
        }

        let output = Output::new(false, Language::En).with_json_mode();
        let options = ResolvedOptions {
            repo: "owner/repo".to_string(),
            remote_path: "README.md".to_string(),
            local_target: Path::new("/tmp/output.json").to_path_buf(),
            git_ref: None,
            token: None,
            api_base: crate::DEFAULT_GITHUB_API_BASE.to_string(),
            proxy_base: String::new(),
            prefix_mode: crate::PrefixProxyMode::Direct,
            concurrency: 4,
            language: Language::En,
            overwrite: false,
            json: true,
            debug: true,
            no_color: true,
        };
        let stats = DownloadStats {
            files_downloaded: 1,
            skipped_existing_files: 0,
            skipped_entries: 0,
        };

        output.startup(&options);
        output.downloading("README.md");
        output.warning("this should be hidden");
        output.debug_line("[debug] token-source: GITHUB_TOKEN");
        output.completion(
            &options.repo,
            &options.remote_path,
            &options.local_target,
            stats.files_downloaded,
            stats.skipped_existing_files,
            stats.skipped_entries,
        );
        output.print_json_success(Path::new("/tmp/output.json"), &stats);
    }

    fn extract_json_document(text: &str) -> Option<&str> {
        let start = text.find('{')?;
        let end = text.rfind('}')?;
        text.get(start..=end)
    }
}

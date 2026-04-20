use clap::CommandFactory;

use crate::i18n::Language;

use super::types::Cli;

pub fn command() -> clap::Command {
    command_for_language(Language::En)
}

pub fn command_for_language(language: Language) -> clap::Command {
    let mut command = Cli::command()
        .help_template(help_template(language))
        .about(command_about(language))
        .after_help(command_after_help(language));

    command = command
        .mut_arg("repo", |arg| arg.help(repo_help(language)))
        .mut_arg("remote_path", |arg| arg.help(remote_path_help(language)))
        .mut_arg("local_target", |arg| arg.help(local_target_help(language)))
        .mut_arg("git_ref", |arg| arg.help(ref_help(language)))
        .mut_arg("token", |arg| arg.help(token_help(language)))
        .mut_arg("proxy_base", |arg| arg.help(proxy_help(language)))
        .mut_arg("prefix_mode", |arg| arg.help(prefix_mode_help(language)))
        .mut_arg("concurrency", |arg| arg.help(concurrency_help(language)))
        .mut_arg("language", |arg| arg.help(language_help(language)))
        .mut_arg("debug", |arg| arg.help(debug_help(language)))
        .mut_arg("no_color", |arg| arg.help(no_color_help(language)));

    command
}

fn command_about(language: Language) -> &'static str {
    match language {
        Language::En => "Download a file or directory from a GitHub repository path",
        Language::Zh => "下载 GitHub 仓库里的单个文件或整个目录",
    }
}

fn command_after_help(language: Language) -> &'static str {
    match language {
        Language::En => {
            "Examples:\n  gh-download openai/openai-python README.md ./README.md\n  gh-download owner/repo src ./downloads --ref main\n  gh-download owner/repo src ./downloads --concurrency 8\n  gh-download owner/private-repo docs ./docs --token <token>\n  gh-download owner/repo docs ./docs --lang zh"
        }
        Language::Zh => {
            "示例:\n  gh-download openai/openai-python README.md ./README.md\n  gh-download owner/repo src ./downloads --ref main\n  gh-download owner/repo src ./downloads --concurrency 8\n  gh-download owner/private-repo docs ./docs --token <token>\n  gh-download owner/repo docs ./docs --lang zh"
        }
    }
}

fn help_template(language: Language) -> &'static str {
    match language {
        Language::En => {
            "{about-with-newline}\nUsage: {usage}\n\nArguments:\n{positionals}\nOptions:\n{options}{after-help}\n"
        }
        Language::Zh => {
            "{about-with-newline}\n用法: {usage}\n\n参数:\n{positionals}\n选项:\n{options}{after-help}\n"
        }
    }
}

fn repo_help(language: Language) -> &'static str {
    match language {
        Language::En => "GitHub repository in OWNER/REPO format, for example openai/openai-python",
        Language::Zh => "GitHub 仓库，格式为 OWNER/REPO，例如 openai/openai-python",
    }
}

fn remote_path_help(language: Language) -> &'static str {
    match language {
        Language::En => "Path inside the repository, for example README.md or src/openai",
        Language::Zh => "仓库内路径，例如 README.md 或 src/openai",
    }
}

fn local_target_help(language: Language) -> &'static str {
    match language {
        Language::En => {
            "Local destination path. Files may be written to a file path or an existing directory; directory downloads treat it as the parent directory by default"
        }
        Language::Zh => "本地目标路径。文件可写入文件路径或现有目录；目录下载时默认作为父目录",
    }
}

fn ref_help(language: Language) -> &'static str {
    match language {
        Language::En => "Branch, tag, or commit SHA",
        Language::Zh => "分支、tag 或 commit SHA",
    }
}

fn token_help(language: Language) -> &'static str {
    match language {
        Language::En => "GitHub token. Defaults to GITHUB_TOKEN or GH_TOKEN",
        Language::Zh => "GitHub token。默认读取 GITHUB_TOKEN 或 GH_TOKEN",
    }
}

fn proxy_help(language: Language) -> &'static str {
    match language {
        Language::En => {
            "URL-prefix proxy base for anonymous raw downloads. Defaults to GH_PROXY_BASE; in fallback/prefer mode it falls back to the built-in gh-proxy when unset"
        }
        Language::Zh => {
            "匿名 raw 下载使用的 URL 前缀代理。默认读取 GH_PROXY_BASE；在 fallback/prefer 模式下未设置时会回退到内置 gh-proxy"
        }
    }
}

fn prefix_mode_help(language: Language) -> &'static str {
    match language {
        Language::En => {
            "Raw download prefix-proxy mode: direct, fallback, or prefer. Defaults to GH_DOWNLOAD_PREFIX_MODE or direct"
        }
        Language::Zh => {
            "raw 下载的前缀代理模式：direct、fallback 或 prefer。默认读取 GH_DOWNLOAD_PREFIX_MODE，未设置时为 direct"
        }
    }
}

fn language_help(language: Language) -> &'static str {
    match language {
        Language::En => {
            "Force the user-facing language. Defaults to English unless the locale indicates Chinese"
        }
        Language::Zh => "显式指定用户可见语言。默认英文；当 locale 指向中文时自动切换为中文",
    }
}

fn concurrency_help(language: Language) -> &'static str {
    match language {
        Language::En => {
            "Maximum number of concurrent file downloads for directory transfers. Must be at least 1. Defaults to 4"
        }
        Language::Zh => "目录下载时的最大并发文件数，最小为 1，默认值为 4",
    }
}

fn debug_help(language: Language) -> &'static str {
    match language {
        Language::En => {
            "Print debug diagnostics for request URLs and download strategy. Defaults to GH_DOWNLOAD_DEBUG"
        }
        Language::Zh => "打印请求 URL 和下载策略的调试信息。默认读取 GH_DOWNLOAD_DEBUG",
    }
}

fn no_color_help(language: Language) -> &'static str {
    match language {
        Language::En => "Disable ANSI colors",
        Language::Zh => "关闭 ANSI 彩色输出",
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use clap::error::ErrorKind;

    use super::*;

    #[test]
    fn help_is_localized_for_chinese() {
        let mut command = command_for_language(Language::Zh);
        let rendered = command.render_help().to_string();
        assert!(rendered.contains("用法:"));
        assert!(rendered.contains("-c"));
        assert!(rendered.contains("显式指定用户可见语言"));
        assert!(rendered.contains("--concurrency"));
    }

    #[test]
    fn empty_invocation_uses_localized_help_flow() {
        let args = vec![OsString::from("gh-download"), OsString::from("--help")];
        let mut command = command_for_language(Language::Zh);
        let error = command
            .try_get_matches_from_mut(args)
            .expect_err("empty invocation should display help");

        assert_eq!(error.kind(), ErrorKind::DisplayHelp);
        let rendered = error.to_string();
        assert!(rendered.contains("用法:"));
        assert!(rendered.contains("下载 GitHub 仓库里的单个文件或整个目录"));
    }

    #[test]
    fn partial_invocation_still_requires_missing_arguments() {
        let args = vec![OsString::from("gh-download"), OsString::from("owner/repo")];
        let mut command = command_for_language(Language::En);
        let error = command
            .try_get_matches_from_mut(args)
            .expect_err("partial invocation should still fail");

        assert_eq!(error.kind(), ErrorKind::MissingRequiredArgument);
        let rendered = error.to_string();
        assert!(rendered.contains("<REMOTE_PATH>"));
        assert!(rendered.contains("<LOCAL_TARGET>"));
    }
}

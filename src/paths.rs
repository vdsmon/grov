use std::path::{Path, PathBuf};

/// Convert a branch name into a safe directory name.
/// `/` → `-`, strip leading `.`, collapse consecutive `-`.
pub fn sanitize_branch_name(branch: &str) -> String {
    let mut result = branch.replace('/', "-");

    // Strip leading dots
    result = result.trim_start_matches('.').to_string();

    // Collapse consecutive dashes
    while result.contains("--") {
        result = result.replace("--", "-");
    }

    // Trim leading/trailing dashes
    result = result.trim_matches('-').to_string();

    if result.is_empty() {
        result = "branch".to_string();
    }

    result
}

/// Build the worktree directory path as a sibling of the bare repo.
///
/// - If prefix is empty: `<bare_repo>/../<sanitized_branch>`
/// - If prefix is set:   `<bare_repo>/../<prefix>_<sanitized_branch>`
pub fn worktree_dir(bare_repo: &Path, branch: &str, prefix: &str) -> PathBuf {
    let parent = bare_repo
        .parent()
        .expect("bare repo must have a parent dir");
    let sanitized = sanitize_branch_name(branch);
    let dir_name = if prefix.is_empty() {
        sanitized
    } else {
        format!("{prefix}_{sanitized}")
    };
    parent.join(dir_name)
}

/// Extract the repository name from a URL, stripping `.git` suffix.
pub fn repo_name_from_url(url: &str) -> String {
    let name = url
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("repo");

    name.strip_suffix(".git").unwrap_or(name).to_string()
}

/// Compute a relative path from `base` to `target`.
///
/// Both paths should be absolute. Returns a relative `PathBuf` suitable for
/// display in `cd` hints. Falls back to returning `target` as-is if the
/// relative computation fails (e.g. different prefixes on Windows).
pub fn relative_from(target: &Path, base: &Path) -> PathBuf {
    let mut base_iter = base.components();
    let mut target_iter = target.components();
    let mut common = 0;

    loop {
        match (base_iter.clone().next(), target_iter.clone().next()) {
            (Some(b), Some(t)) if b == t => {
                base_iter.next();
                target_iter.next();
                common += 1;
            }
            _ => break,
        }
    }

    if common == 0 {
        return target.to_path_buf();
    }

    let ups = base_iter.count();
    let mut result = PathBuf::new();
    for _ in 0..ups {
        result.push("..");
    }

    for component in target_iter {
        result.push(component);
    }

    if result.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_simple_branch() {
        assert_eq!(sanitize_branch_name("main"), "main");
    }

    #[test]
    fn sanitize_slashes() {
        assert_eq!(sanitize_branch_name("feature/my-thing"), "feature-my-thing");
    }

    #[test]
    fn sanitize_leading_dot() {
        assert_eq!(sanitize_branch_name(".hidden"), "hidden");
    }

    #[test]
    fn sanitize_consecutive_dashes() {
        assert_eq!(sanitize_branch_name("a//b"), "a-b");
    }

    #[test]
    fn worktree_dir_with_prefix() {
        let bare = Path::new("/repos/myproject/repo.git");
        assert_eq!(
            worktree_dir(bare, "feature/login", "mp"),
            PathBuf::from("/repos/myproject/mp_feature-login")
        );
    }

    #[test]
    fn worktree_dir_without_prefix() {
        let bare = Path::new("/repos/myproject/repo.git");
        assert_eq!(
            worktree_dir(bare, "feature/login", ""),
            PathBuf::from("/repos/myproject/feature-login")
        );
    }

    #[test]
    fn repo_name_from_https_url() {
        assert_eq!(
            repo_name_from_url("https://github.com/user/repo.git"),
            "repo"
        );
    }

    #[test]
    fn repo_name_from_ssh_url() {
        assert_eq!(
            repo_name_from_url("git@github.com:user/my-project.git"),
            "my-project"
        );
    }

    #[test]
    fn repo_name_without_git_suffix() {
        assert_eq!(repo_name_from_url("https://github.com/user/repo"), "repo");
    }

    #[test]
    fn repo_name_trailing_slash() {
        assert_eq!(
            repo_name_from_url("https://github.com/user/repo.git/"),
            "repo"
        );
    }

    #[test]
    fn relative_from_same_dir() {
        assert_eq!(
            relative_from(Path::new("/a/b"), Path::new("/a/b")),
            PathBuf::from(".")
        );
    }

    #[test]
    fn relative_from_child() {
        assert_eq!(
            relative_from(Path::new("/a/b"), Path::new("/a")),
            PathBuf::from("b")
        );
    }

    #[test]
    fn relative_from_sibling() {
        assert_eq!(
            relative_from(Path::new("/a/c"), Path::new("/a/b")),
            PathBuf::from("../c")
        );
    }

    #[test]
    fn relative_from_deeply_nested() {
        assert_eq!(
            relative_from(Path::new("/a/b/c/d"), Path::new("/a/x/y")),
            PathBuf::from("../../b/c/d")
        );
    }

    #[test]
    fn relative_from_parent() {
        assert_eq!(
            relative_from(Path::new("/a"), Path::new("/a/b/c")),
            PathBuf::from("../..")
        );
    }
}

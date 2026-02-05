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

/// Build the worktree directory path: `<bare_repo>/trees/<sanitized_branch>`
pub fn worktree_dir(bare_repo: &Path, branch: &str) -> PathBuf {
    bare_repo.join("trees").join(sanitize_branch_name(branch))
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
    fn worktree_dir_path() {
        let bare = Path::new("/repos/myproject.git");
        assert_eq!(
            worktree_dir(bare, "feature/login"),
            PathBuf::from("/repos/myproject.git/trees/feature-login")
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
}

// Common test utilities shared across test modules

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Gets the path to a fixture directory
pub fn fixture_path(name: &str) -> PathBuf {
    Path::new("tests").join("fixtures").join(name)
}

/// Run cargo-autoinherit in the specified directory with given args
pub fn run_autoinherit<P: AsRef<Path>>(dir: P, args: &[&str]) -> Output {
    Command::new("cargo")
        .arg("autoinherit")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("Failed to execute cargo-autoinherit")
}

/// Returns the stderr of an output as a String
pub fn get_stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

/// Returns the stdout of an output as a String
pub fn get_stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[cfg(test)]
mod tests {
    // Tests for the new-child-dep fixture in check mode

    use super::*;

    /// Test that check mode correctly identifies a dependency in child
    /// that should be added to the workspace
    #[test]
    fn test_identifies_missing_workspace_dependency() {
        let path = fixture_path("new-child-dep");

        // Run in check mode
        let output = run_autoinherit(&path, &["--check"]);

        // Should exit with non-zero status since changes are needed
        assert!(
            !output.status.success(),
            "Expected check to fail but it succeeded"
        );

        // Should indicate that regex needs to be added to workspace
        let stderr = get_stderr(&output);
        assert!(
            stderr.contains("`regex` should be declared as a workspace dependency"),
            "Expected error about missing workspace dependency, but got: {}",
            stderr
        );
    }

    /// Test that check mode can be instructed to ignore a specific package
    #[test]
    fn test_exclude_member_skips_checking() {
        let path = fixture_path("new-child-dep");

        // Run with exclude flag
        let output = run_autoinherit(&path, &["--check", "--exclude-members", "child"]);

        // Should succeed since the problematic package is excluded
        assert!(
            output.status.success(),
            "Expected check to succeed with excluded member, but it failed"
        );

        // Should mention the excluded member
        let stdout = get_stdout(&output);
        assert!(
            stdout.contains("Excluded workspace member `child`"),
            "Expected message about excluded member, but got: {}",
            stdout
        );
    }
    // Tests for the uninherited-existing-child-dep fixture in check mode

    /// Test that check mode correctly identifies a dependency that exists in the workspace
    /// but is redefined in a child package rather than being inherited
    #[test]
    fn test_identifies_uninherited_dependency() {
        let path = fixture_path("uninherited-existing-child-dep");

        // Run in check mode
        let output = run_autoinherit(&path, &["--check"]);

        // Should exit with non-zero status since changes are needed
        assert!(
            !output.status.success(),
            "Expected check to fail but it succeeded"
        );

        // Should indicate that regex in child should inherit from workspace
        let stderr = get_stderr(&output);
        assert!(
            stderr.contains("`regex` should inherit from a workspace dependency"),
            "Expected error about uninherited dependency, but got: {}",
            stderr
        );
    }

    /// Test that "--exclude-members" flag works correctly
    #[test]
    fn test_exclude_flag_allows_uninherited_dependency() {
        let path = fixture_path("uninherited-existing-child-dep");

        // Run with exclude flag
        let output = run_autoinherit(&path, &["--check", "--exclude-members", "child"]);

        // Should succeed since the problematic package is excluded
        assert!(
            output.status.success(),
            "Expected check to succeed with excluded member, but it failed"
        );

        // Should mention the excluded member
        let stdout = get_stdout(&output);
        assert!(
            stdout.contains("Excluded workspace member `child`"),
            "Expected message about excluded member, but got: {}",
            stdout
        );
    }

    /// Test that check mode also works with the "--prefer-simple-dotted" flag
    #[test]
    fn test_check_with_prefer_simple_dotted() {
        let path = fixture_path("uninherited-existing-child-dep");

        // Run with prefer-simple-dotted flag
        let output = run_autoinherit(&path, &["--check", "--prefer-simple-dotted"]);

        // Should still detect issues even with formatting preferences set
        assert!(
            !output.status.success(),
            "Expected check to fail but it succeeded"
        );

        // Should indicate that regex in child should inherit from workspace
        let stderr = get_stderr(&output);
        assert!(
            stderr.contains("`regex` should inherit from a workspace dependency"),
            "Expected error about uninherited dependency, but got: {}",
            stderr
        );
    }

    /// Test that multiple sources are detected and reported properly
    #[test]
    fn test_identifies_multiple_sources() {
        let path = fixture_path("multiple-source-child-deps");

        // Run in check mode
        let output = run_autoinherit(&path, &["--check"]);

        // Should exit with non-zero status since changes are needed
        assert!(
            !output.status.success(),
            "Expected check to fail but it succeeded"
        );

        // Should indicate that rand has multiple sources
        let stderr = get_stderr(&output);
        assert!(
            stderr.contains("`rand` won't be auto-inherited because there are multiple sources for it:"),
            "Expected error about multiple sources, but got: {}",
            stderr
        );

        // Should list each source
        assert!(
            stderr.contains("version: ^0.8.0") && stderr.contains("version: =0.8.0"),
            "Expected stderr to list both version constraints, but got: {}",
            stderr
        );
    }

    /// Test that the --allow-multiple-sources flag lets the check pass
    #[test]
    fn test_allow_multiple_sources_flag() {
        let path = fixture_path("multiple-source-child-deps");

        // Run with allow-multiple-sources flag
        let output = run_autoinherit(&path, &["--check", "--allow-multiple-sources"]);

        // Should exit with status 0 since multiple sources are allowed
        assert!(
            output.status.success(),
            "Expected check to succeed with --allow-multiple-sources, but it failed"
        );

        // Should still indicate the presence of multiple sources in output
        // (but it shouldn't be treated as an error)
        let stderr = get_stderr(&output);
        assert!(
            stderr.contains("`rand` won't be auto-inherited because there are multiple sources for it:"),
            "Expected warning about multiple sources, but got: {}",
            stderr
        );
    }
}

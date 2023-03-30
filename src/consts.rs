use std::time::Duration;

pub const POLL_INTERVAL: Duration = Duration::from_millis(16);
pub const SYSTEM_PROMPT: &str = include_str!("./system_prompt.txt");
pub const PROJECT_NAME: &str = "ushidashi";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_readme_includes_system_prompt() {
        let readme_content = include_str!("../README.md");

        assert!(
            readme_content.contains(SYSTEM_PROMPT),
            "README.md does not contain the contents of system_prompt.txt."
        );
    }
}

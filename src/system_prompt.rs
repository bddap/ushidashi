pub const SYSTEM_PROMPT: &str = include_str!("./system_prompt.txt");

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

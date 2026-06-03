/// Returns a greeting for smoke-testing the workspace.
pub fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

#[cfg(test)]
mod tests {
    use super::greet;

    #[test]
    fn greet_includes_name() {
        assert_eq!(greet("agentcfg"), "Hello, agentcfg!");
    }
}

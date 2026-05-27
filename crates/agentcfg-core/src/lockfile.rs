//! **Lockfile** — records **Locked Desired State** for Configured Items that need
//! repeatable Skill Source resolution.
//!
//! Serialization and read/write are implemented in later milestones (implementation plan M4).

#[cfg(test)]
mod tests {
    #[test]
    fn lockfile_glossary_term_is_module_anchor() {
        // Anchor test so `cargo test lockfile` matches this module after M1.5.5.
        const _: &str = "Lockfile records Locked Desired State";
    }
}

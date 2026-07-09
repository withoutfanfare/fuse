pub(crate) fn is_direct_to_production(
    base_branch: &str,
    head_branch: &str,
    default_branch: &str,
    integration_branch: Option<&str>,
) -> bool {
    if !base_branch.eq_ignore_ascii_case(default_branch) {
        return false;
    }

    match integration_branch
        .map(str::trim)
        .filter(|branch| !branch.is_empty())
    {
        Some(integration_branch) => !head_branch.eq_ignore_ascii_case(integration_branch),
        None => true,
    }
}

pub(crate) fn direct_to_production_error(
    pr_number: i64,
    base_branch: &str,
    default_branch: &str,
    integration_branch: Option<&str>,
) -> String {
    let integration_branch = integration_branch
        .map(str::trim)
        .filter(|branch| !branch.is_empty());

    match integration_branch {
        Some(integration_branch) => format!(
            "Refused to merge: PR #{pr_number} targets configured production branch '{base_branch}' directly. Merge via integration branch '{integration_branch}' before promoting to '{default_branch}'."
        ),
        None => format!(
            "Refused to merge: PR #{pr_number} targets configured production branch '{base_branch}' directly, and no integration branch is configured for this repository."
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::is_direct_to_production;

    #[test]
    fn feature_pr_into_production_blocks() {
        assert!(is_direct_to_production(
            "main",
            "feature/login",
            "main",
            Some("develop"),
        ));
    }

    #[test]
    fn integration_pr_into_production_is_allowed() {
        assert!(!is_direct_to_production(
            "main",
            "develop",
            "main",
            Some("develop"),
        ));
    }

    #[test]
    fn branch_policy_is_case_insensitive() {
        assert!(!is_direct_to_production(
            "MAIN",
            "Develop",
            "main",
            Some("develop"),
        ));
    }

    #[test]
    fn missing_or_empty_integration_blocks_production_targets() {
        assert!(is_direct_to_production("main", "feature/login", "main", None));
        assert!(is_direct_to_production(
            "main",
            "feature/login",
            "main",
            Some(" "),
        ));
    }

    #[test]
    fn custom_production_branch_blocks() {
        assert!(is_direct_to_production(
            "release",
            "feature/login",
            "release",
            Some("integration"),
        ));
    }

    #[test]
    fn non_production_base_is_allowed() {
        assert!(!is_direct_to_production(
            "develop",
            "feature/login",
            "main",
            Some("develop"),
        ));
    }
}

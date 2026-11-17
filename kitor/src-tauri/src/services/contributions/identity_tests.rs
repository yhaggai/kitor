use super::contribution_endpoint;

#[test]
fn blank_build_variable_uses_the_debug_default() {
    for value in [None, Some(""), Some(" \n")] {
        assert_eq!(
            contribution_endpoint(value, true).unwrap(),
            "https://kitor-contributions-staging.yhaggai.workers.dev"
        );
        assert_eq!(
            contribution_endpoint(value, false).unwrap_err(),
            "Contribution sign-in is not configured in this build"
        );
    }
}

#[test]
fn hosted_builds_keep_the_configured_https_origin() {
    for environment in ["staging", "production"] {
        let origin = format!("https://kitor-contributions-{environment}.yhaggai.workers.dev");
        let configured = format!(" {origin}/ ");
        assert_eq!(
            contribution_endpoint(Some(&configured), false).unwrap(),
            origin
        );
    }
}

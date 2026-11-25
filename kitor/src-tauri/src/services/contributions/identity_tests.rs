use super::contribution_endpoint;

#[test]
fn missing_or_blank_build_variable_disables_sign_in() {
    for debug in [true, false] {
        for value in [None, Some(""), Some(" \n")] {
            assert_eq!(
                contribution_endpoint(value, debug).unwrap_err(),
                "Contribution sign-in is not configured in this build"
            );
        }
    }
}

#[test]
fn hosted_builds_keep_the_configured_https_origin() {
    for debug in [true, false] {
        assert_eq!(
            contribution_endpoint(Some(" https://contributions.example.com/ "), debug).unwrap(),
            "https://contributions.example.com"
        );
    }
}

#[test]
fn local_http_endpoint_is_allowed_only_in_debug_builds() {
    let local = "http://127.0.0.1:8787";
    assert_eq!(contribution_endpoint(Some(local), true).unwrap(), local);
    assert!(contribution_endpoint(Some(local), false).is_err());
}

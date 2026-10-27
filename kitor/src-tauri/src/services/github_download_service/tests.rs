use super::*;
#[test]
fn limits_authenticated_requests_to_kitor() {
    assert!(api_url("/repos/yhaggai/kitor/releases/tags/v1").is_ok());
    for path in [
        "/repos/other/repo",
        "/repos/yhaggai/kitor-evil",
        "/repos/yhaggai/kitor/../other",
    ] {
        assert!(api_url(path).is_err());
    }
    assert!(
        asset_parts("https://github.com/yhaggai/kitor/releases/download/v1/patch.zip")
            .is_some()
    );
    for url in [
        "http://github.com/yhaggai/kitor/releases/download/v1/patch.zip",
        "https://github.com/other/repo/releases/download/v1/patch.zip",
        "https://github.com/yhaggai/kitor/releases/download/v1/%2e%2e",
        "https://github.com/yhaggai/kitor/releases/download/v1/patch.zip?token=x",
    ] {
        assert!(asset_parts(url).is_none());
    }
}
#[test]
fn limits_unauthenticated_redirects_to_github_asset_hosts() {
    assert!(trusted_asset_redirect(
        "https://release-assets.githubusercontent.com/asset?sig=test"
    ));
    for url in [
        "http://release-assets.githubusercontent.com/asset",
        "https://evil.example/asset",
        "https://release-assets.githubusercontent.com.evil.example/asset",
        "https://user@objects.githubusercontent.com/asset",
        "https://api.github.com/repos/yhaggai/kitor",
    ] {
        assert!(!trusted_asset_redirect(url));
    }
}

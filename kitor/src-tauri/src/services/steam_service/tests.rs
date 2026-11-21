use super::*;
#[test]
fn distinguishes_private_empty_owned_and_broken_libraries() {
    assert!(parse_owned(r#"{"response":{}}"#).unwrap().is_none());
    assert!(parse_owned(r#"{"response":{"game_count":0}}"#)
        .unwrap()
        .unwrap()
        .is_empty());
    assert!(
        parse_owned(r#"{"response":{"game_count":1,"games":[{"appid":10110}]}}"#)
            .unwrap()
            .unwrap()
            .contains(&10110)
    );
    assert!(parse_owned(r#"{"response":{"game_count":1}}"#).is_err());
    assert!(parse_owned("bad response").is_err());
    assert_eq!(destination(10110, true), "steam://nav/games/details/10110");
    assert_eq!(destination(10110, false), "steam://store/10110");
}
#[test]
fn keys_never_appear_in_status_and_account_changes_clear_ownership() {
    let mut session = Session::default();
    assert!(set_key(&mut session, "bad".into()).is_err());
    let key = "a".repeat(32);
    set_key(&mut session, key.clone()).unwrap();
    session.games = Some(HashSet::from([10110]));
    assert!(!serde_json::to_string(&session.status())
        .unwrap()
        .contains(&key));
    session.begin_login();
    assert!(session.games.is_none());
    assert!(session.checked.is_none());
    assert!(session.connecting);
}
#[test]
fn persists_only_identity_and_disconnect_removes_it() {
    let dir = tempfile::tempdir().unwrap();
    assert!(load_identity(dir.path()).is_none());
    save_identity(dir.path(), "76561198000000000").unwrap();
    assert_eq!(
        load_identity(dir.path()).as_deref(),
        Some("76561198000000000")
    );
    clear_identity(dir.path()).unwrap();
    assert!(load_identity(dir.path()).is_none());
    clear_identity(dir.path()).unwrap();
}
#[tokio::test]
async fn disconnected_accounts_route_to_store_without_network() {
    let state = SteamState::default();
    assert!(refresh(&state).await.owned_app_ids.is_none());
    assert_eq!(game_url(&state, 10110).await, "steam://store/10110");
}

#[test]
#[ignore = "writes an isolated dummy credential to the native OS credential store"]
fn native_credential_survives_reopening_and_can_be_removed() {
    let user = format!(
        "persistence-test-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    );
    let entry = keyring::Entry::new("click.kitor.steam.tests", &user).unwrap();
    let dummy = "test-only-not-a-steam-key";
    entry.set_password(dummy).unwrap();
    drop(entry);
    let reopened = keyring::Entry::new("click.kitor.steam.tests", &user).unwrap();
    let restored = reopened.get_password();
    reopened.delete_credential().unwrap();
    assert!(restored.is_ok_and(|value| value == dummy));
    assert!(matches!(
        reopened.get_password(),
        Err(keyring::Error::NoEntry)
    ));
}

#[test]
fn clearing_session_drops_key_and_prevents_restore() {
    let mut session = Session::default();
    set_key(&mut session, "a".repeat(32)).unwrap();
    session.steam_id = Some("76561198000000000".into());
    session.games = Some(HashSet::from([10110]));
    let generation = session.generation;
    clear_session(&mut session);
    assert!(!session.status().has_api_key);
    assert!(session.steam_id.is_none());
    assert!(session.games.is_none());
    assert!(session.credentials_loaded);
    assert!(session.generation > generation);
}

fn valid_assertion() -> HashMap<String, String> {
    [
        ("openid.ns", NS.to_string()),
        ("openid.mode", "id_res".into()),
        ("openid.op_endpoint", OPENID.into()),
        (
            "openid.return_to",
            "http://127.0.0.1:123/callback/token".into(),
        ),
        (
            "openid.claimed_id",
            "https://steamcommunity.com/openid/id/76561198000000000".into(),
        ),
        (
            "openid.identity",
            "https://steamcommunity.com/openid/id/76561198000000000".into(),
        ),
        (
            "openid.response_nonce",
            format!(
                "{}nonce",
                time::OffsetDateTime::now_utc()
                    .format(time::macros::format_description!(
                        "[year]-[month]-[day]T[hour]:[minute]:[second]Z"
                    ))
                    .unwrap()
            ),
        ),
        (
            "openid.signed",
            "op_endpoint,claimed_id,identity,return_to,response_nonce,assoc_handle".into(),
        ),
    ]
    .into_iter()
    .map(|(k, v)| (k.into(), v))
    .collect()
}
#[test]
fn rejects_forged_replayed_and_misdirected_assertions() {
    let params = valid_assertion();
    let target = "http://127.0.0.1:123/callback/token";
    assert!(assertion(&params, target).is_ok());
    assert!(assertion(&params, "http://127.0.0.1:123/callback/other").is_err());
    for (key, value) in [
        ("openid.op_endpoint", "https://evil.test"),
        ("openid.identity", "other"),
        ("openid.signed", "identity"),
        ("openid.response_nonce", "2000-01-01T00:00:00Zold"),
        ("openid.mode", "cancel"),
    ] {
        let mut invalid = params.clone();
        invalid.insert(key.into(), value.into());
        assert!(assertion(&invalid, target).is_err());
    }
}

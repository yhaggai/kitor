use super::*;
#[test]
fn drafts_survive_restart_and_reject_stale_writes() {
    #[derive(Deserialize)]
    struct Fixture {
        catalog: String,
        source: String,
        edits: BTreeMap<String, String>,
    }

    let fixtures: Vec<Fixture> =
        serde_json::from_str(include_str!("../../../../testdata/contribution-csv.json")).unwrap();
    let root = tempfile::tempdir().unwrap();
    for fixture in fixtures {
        let config = csv_text::catalog(&fixture.catalog).unwrap();
        write(
            root.path(),
            config,
            &Draft {
                schema: 1,
                base_sha: "a".repeat(40),
                base_branch: "v1".into(),
                source: fixture.source,
                edits: BTreeMap::new(),
                revision: 0,
            },
        )
        .unwrap();
        let change_count = fixture.edits.len();
        assert_eq!(
            save(root.path(), config, 0, fixture.edits.clone()).unwrap(),
            1,
            "{}",
            config.id
        );
        assert!(save(root.path(), config, 0, BTreeMap::new()).is_err());
        assert_eq!(read(root.path(), config).unwrap().edits, fixture.edits);
        assert_eq!(
            preview(root.path(), config).unwrap().changes.len(),
            change_count
        );
    }
}

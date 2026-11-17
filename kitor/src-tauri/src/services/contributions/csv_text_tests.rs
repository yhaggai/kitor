use super::*;
const SOURCE: &str =
    include_str!("../../../../../patches-workspace/drifter/he/payload/translation.csv");
use std::path::Path;

#[test]
fn beta_catalog_csv_matches_worker_fixtures() {
    #[derive(Deserialize)]
    struct Fixture {
        catalog: String,
        source: String,
        expected: String,
        edits: BTreeMap<String, String>,
        rejected: BTreeMap<String, String>,
    }
    let fixtures: Vec<Fixture> =
        serde_json::from_str(include_str!("../../../../testdata/contribution-csv.json")).unwrap();
    for fixture in fixtures {
        let config = catalog(&fixture.catalog).unwrap();
        let rows = rows(config, &fixture.source).unwrap();
        assert!(fixture
            .edits
            .keys()
            .all(|key| rows.iter().any(|row| &row.key == key)));
        assert_eq!(
            apply(config, &fixture.source, &fixture.edits, true).unwrap(),
            fixture.expected,
            "{}",
            fixture.catalog
        );
        assert!(apply(config, &fixture.source, &fixture.rejected, true).is_err());
    }
}

#[test]
fn desktop_loads_all_beta_sources() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for config in CATALOGS.iter().filter(|c| c.id != "drifter") {
        let source = std::fs::read_to_string(root.join(&config.path)).unwrap();
        assert!(source.len() <= super::super::MAX_CATALOG_BYTES);
        let rows = rows(config, &source).unwrap();
        assert!(!rows.is_empty());
        assert!(rows.iter().all(|row| row.key.len() <= 2048));
    }
}

#[test]
fn full_roundtrip_and_single_change() {
    let rows = rows(catalog("drifter").unwrap(), SOURCE).unwrap();
    assert_eq!(rows.len(), 9799);
    assert_eq!(
        apply(catalog("drifter").unwrap(), SOURCE, &BTreeMap::new(), true).unwrap(),
        SOURCE
    );
    let row = rows
        .iter()
        .find(|r| r.english == "Ok" && r.context == "Room- Title / ContinueGame")
        .unwrap();
    let edits = BTreeMap::from([(row.key.clone(), "רדסב".into())]);
    let actual = apply(catalog("drifter").unwrap(), SOURCE, &edits, true).unwrap();
    let expected = SOURCE.replacen(
        ",1,Room- Title,ContinueGame,Ok,רושיא",
        ",1,Room- Title,ContinueGame,Ok,רדסב",
        1,
    );
    let first = actual
        .bytes()
        .zip(expected.bytes())
        .position(|(a, b)| a != b);
    assert!(
        actual == expected,
        "byte mismatch {:?}; lengths {} / {}; nearby {:?} / {:?}",
        first,
        actual.len(),
        expected.len(),
        first.map(|p| &actual.as_bytes()[p.saturating_sub(15)..(p + 20).min(actual.len())]),
        first.map(|p| &expected.as_bytes()[p.saturating_sub(15)..(p + 20).min(expected.len())])
    );
}
#[test]
fn invalid_edits_and_quoted_newlines() {
    assert!(apply(
        catalog("drifter").unwrap(),
        SOURCE,
        &BTreeMap::from([("missing".into(), "x".into())]),
        true
    )
    .is_err());
    let row = &rows(catalog("drifter").unwrap(), SOURCE).unwrap()[0];
    assert!(apply(
        catalog("drifter").unwrap(),
        SOURCE,
        &BTreeMap::from([(row.key.clone(), "".into())]),
        true
    )
    .is_err());
    let value = "שורה, \"אחת\"\nושנייה";
    let output = apply(
        catalog("drifter").unwrap(),
        SOURCE,
        &BTreeMap::from([(row.key.clone(), value.into())]),
        true,
    )
    .unwrap();
    assert_eq!(
        rows(catalog("drifter").unwrap(), &output).unwrap()[0].hebrew,
        value
    );
}

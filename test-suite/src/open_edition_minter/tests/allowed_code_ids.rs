use crate::common_setup::templates::open_edition_minter_custom_template;

#[test]
fn invalid_code_id() {
    // Set an invalid code id for the nft contract
    let vt = open_edition_minter_custom_template(
        None,
        None,
        None,
        Some(10),
        Some(5),
        None,
        None,
        None,
        Some(19),
    );
    assert_eq!(
        vt.err()
            .unwrap()
            .err()
            .unwrap()
            .source()
            .unwrap()
            .to_string(),
        "InvalidCollectionCodeId 19".to_string()
    );

    // All the other tests related to Sudo params of the factory contract are tested in the factory
    // tests
}

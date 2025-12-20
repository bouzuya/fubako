use predicates::prelude::PredicateBooleanExt;

#[test]
fn test_no_subcommand() {
    assert_cmd::Command::new(assert_cmd::cargo::cargo_bin!("fubako"))
        .assert()
        .failure()
        .stderr(
            predicates::str::contains("Usage: fubako <COMMAND>")
                .and(predicates::str::contains("Commands:"))
                .and(predicates::str::contains("  edit"))
                .and(predicates::str::contains("  get"))
                .and(predicates::str::contains("  image"))
                .and(predicates::str::contains("  new"))
                .and(predicates::str::contains("  serve"))
                .and(predicates::str::contains("  help"))
                .and(predicates::str::contains("Options:"))
                .and(predicates::str::contains("  -h, --help")),
        );
}

//! Translation hygiene, checked in the test gate so a bad string
//! fails CI instead of reaching a translator: no translation call
//! may join string literals (only the first piece would reach the
//! .pot), and no catalog entry may carry a keyboard shortcut its
//! source string does not have.

use std::path::Path;

#[test]
fn translation_calls_do_not_join_literals() {
	let root = Path::new(env!("CARGO_MANIFEST_DIR"));
	let core = root.join("../../crates/chroma-core/src");
	let app = root.join("src");
	patois_build::lint::check_sources(&[&core, &app]).unwrap();
}

#[test]
fn catalogs_have_no_stale_shortcuts() {
	let po = Path::new(env!("CARGO_MANIFEST_DIR")).join("po");
	patois_build::lint::check_catalogs(&po).unwrap();
}

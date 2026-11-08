//! Regenerate the translation template `po/blind-designer.pot` from
//! every `t("...")` string in the workspace's translatable crates
//! (this app and chroma-core). Needs the gettext tools on PATH.
//!
//! Run: cargo run -p blind-designer --example gen_pot
//! Then update each catalog: msgmerge --update --backup=none
//! --no-fuzzy-matching po/<lang>.po po/blind-designer.pot

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	let workspace_root = manifest_dir.join("../..");
	patois_build::gen_pot(workspace_root, manifest_dir.join("po"), "blind-designer")
}

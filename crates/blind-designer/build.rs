//! Build script: on Windows, embed the application manifest.
//!
//! The manifest declares a dependency on version 6 of the Common
//! Controls library (Microsoft.Windows.Common-Controls 6.0.0.0) —
//! without it wxWidgets warns at startup that the application "doesn't
//! use a correct manifest specifying Common Controls Library v6" and
//! the controls render in their legacy style. Same mechanism as
//! BlindRadio: shipfitter's windows helper wraps the embed-manifest
//! crate, and because the manifest is compiled into the .exe itself,
//! it rides along in the shipped zip with no extra file to lose.
//! Off Windows this is a no-op.

fn main() {
	println!("cargo:rerun-if-changed=build.rs");
	// Compile the gettext catalogs in po/ into locale/, where
	// patois::embed_domain! bakes them into the binary. Without the
	// gettext tools on PATH this warns and the app runs in English.
	patois_build::compile_translations("po", "locale");
	if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
		if let Err(e) = shipfitter::windows::embed_manifest("Blind Designer") {
			println!("cargo:warning=manifest not embedded: {e}");
		}
	}
}

//! Release tooling (same pattern as BlindRadio's and BlindRSS-Rust's xtask).
//!
//! `cargo xtask sign <file>...` writes `<file>.minisig` beside
//! each file, the minisign signature the updater verifies a
//! download against (crates/blind-designer/assets/minisign.pub).
//! The secret key comes from the `MINISIGN_KEY` environment
//! variable — the SigningThings blind-designer pair; it is never
//! stored in git. `MINISIGN_PASSWORD` unlocks the key.
//!
//! `cargo xtask verify <file>...` checks each `<file>.minisig`
//! against the public key the app ships, exactly as the updater
//! will.
//!
//! `cargo xtask update-check <version>` asks Forgejo what a copy
//! running `version` would be offered, downloads that asset and
//! verifies it, exactly as the app does.

use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("sign") => sign_assets(&args[1..]),
        Some("verify") => verify_assets(&args[1..]),
        #[cfg(feature = "update-check")]
        Some("update-check") => update_check(&args[1..]),
        _ => Err(
            "usage: cargo xtask sign <file>... | cargo xtask verify <file>... | cargo xtask update-check <version>"
                .to_string(),
        ),
    };
    if let Err(e) = result {
        eprintln!("xtask: {e}");
        std::process::exit(1);
    }
}

/// `cargo xtask sign <file>...`: writes `<file>.minisig`
/// beside each.
fn sign_assets(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("sign needs at least one file".to_string());
    }
    for file in args {
        if !sign_file(Path::new(file))? {
            return Err("MINISIGN_KEY is not set".to_string());
        }
        println!("xtask: signed {file}");
    }
    Ok(())
}

/// Writes `<file>.minisig` beside `file`. Returns false when
/// there is no key, so a build without one still runs.
fn sign_file(path: &Path) -> Result<bool, String> {
    let Ok(key) = std::env::var("MINISIGN_KEY") else {
        return Ok(false);
    };
    // An empty password would take the encrypted-key branch and
    // fail on an unencrypted key. Empty means absent.
    let password = std::env::var("MINISIGN_PASSWORD")
        .ok()
        .filter(|p| !p.is_empty());
    sign_file_with(path, &key, password.as_deref())?;
    Ok(true)
}

/// The signing itself, parameterised so tests can drive it
/// without the environment.
fn sign_file_with(path: &Path, key_text: &str, password: Option<&str>) -> Result<(), String> {
    let boxed =
        minisign::SecretKeyBox::from_string(key_text).map_err(|e| format!("MINISIGN_KEY: {e}"))?;
    let key = match password {
        Some(password) => boxed.into_secret_key(Some(password.to_string())),
        None => boxed.into_unencrypted_secret_key(),
    }
    .map_err(|e| format!("MINISIGN_KEY: {e}"))?;
    let name = path
        .file_name()
        .ok_or_else(|| format!("{} has no file name", path.display()))?
        .to_string_lossy();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    let trusted = format!("timestamp:{timestamp}\tfile:{name}\thashed");
    let reader = std::io::BufReader::new(
        std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?,
    );
    let signature = minisign::sign(None, &key, reader, Some(&trusted), None)
        .map_err(|e| format!("signing {}: {e}", path.display()))?;
    let mut out = path.as_os_str().to_owned();
    out.push(".minisig");
    std::fs::write(&out, signature.into_string())
        .map_err(|e| format!("{}: {e}", Path::new(&out).display()))?;
    Ok(())
}

/// The workspace root (xtask lives in `crates/`).
fn root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives in crates/")
        .parent()
        .expect("xtask lives in crates/")
        .to_path_buf()
}

/// The base64 key line of the release-signing public key the app
/// ships (`crates/blind-designer/assets/minisign.pub`).
fn shipped_public_key() -> Result<String, String> {
    let text = std::fs::read_to_string(root().join("crates/blind-designer/assets/minisign.pub"))
        .map_err(|e| format!("minisign.pub: {e}"))?;
    Ok(text.lines().nth(1).unwrap_or_default().trim().to_owned())
}

/// `cargo xtask verify <file>...`: checks each `<file>.minisig`
/// against the shipped public key, exactly as the updater will.
fn verify_assets(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("verify needs at least one file".to_string());
    }
    let public = minisign_verify::PublicKey::from_base64(&shipped_public_key()?)
        .map_err(|e| format!("public key: {e}"))?;
    for file in args {
        let signature = std::fs::read_to_string(format!("{file}.minisig"))
            .map_err(|e| format!("{file}.minisig: {e}"))?;
        let signature = minisign_verify::Signature::decode(&signature)
            .map_err(|e| format!("{file}.minisig: {e}"))?;
        let bytes = std::fs::read(file).map_err(|e| format!("{file}: {e}"))?;
        public
            .verify(&bytes, &signature, false)
            .map_err(|e| format!("{file}: {e}"))?;
        println!("xtask: verified {file}");
    }
    Ok(())
}

/// `cargo xtask update-check <version>`: asks Forgejo what a copy
/// running `version` would be offered, downloads that asset and
/// verifies it, exactly as the app does.
#[cfg(feature = "update-check")]
fn update_check(args: &[String]) -> Result<(), String> {
    let Some(current) = args.first() else {
        return Err("usage: cargo xtask update-check <version>".to_string());
    };
    let config = ship_shape::UpdaterConfig::new(
        "serrebi/blind-designer",
        "BlindDesigner",
        "Blind Designer",
        shipped_public_key()?,
        current.trim_start_matches('v'),
    )
    .with_install_kind(ship_shape::InstallKind::Portable)
    .with_api_base_url("https://git.serrebiradio.com/api/v1");
    let update = match ship_shape::check_for_updates(&config, ship_shape::UpdateChannel::Stable)
        .map_err(|e| format!("check: {e}"))?
    {
        ship_shape::UpdateCheckOutcome::UpToDate(version) => {
            println!("xtask: up to date at {version}");
            return Ok(());
        }
        ship_shape::UpdateCheckOutcome::UpdateAvailable(update) => update,
    };
    println!(
        "xtask: {} would be offered {} from {}",
        config.current_version, update.latest_version, update.download_url
    );
    let cancelled = std::sync::atomic::AtomicBool::new(false);
    let path = ship_shape::download_update_file(
        &config,
        &update.download_url,
        &update.signature_url,
        &cancelled,
        |_, _| {},
    )
    .map_err(|e| format!("download: {e}"))?;
    println!("xtask: verified and saved to {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sign a scratch file with a throwaway keypair and verify
    /// the .minisig exactly the way the updater does
    /// (minisign-verify against the public half). The release
    /// key is never involved.
    #[test]
    fn sign_round_trips_through_verify() {
        let kp = minisign::KeyPair::generate_unencrypted_keypair().unwrap();
        let key_text = kp.sk.to_box(None).unwrap().into_string();
        let pub_box = kp.pk.to_box().unwrap().into_string();
        let pub_b64 = pub_box.lines().nth(1).unwrap().trim();

        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let file = std::env::temp_dir().join(format!(
            "blind_designer_xtask_sign_{}_{nanos}.bin",
            std::process::id()
        ));
        std::fs::write(&file, b"release bytes").unwrap();
        sign_file_with(&file, &key_text, None).unwrap();

        let mut sig_path = file.as_os_str().to_owned();
        sig_path.push(".minisig");
        let sig_text = std::fs::read_to_string(&sig_path).unwrap();
        let pk = minisign_verify::PublicKey::from_base64(pub_b64).unwrap();
        let sig = minisign_verify::Signature::decode(&sig_text).unwrap();
        pk.verify(b"release bytes", &sig, false)
            .expect("signature must verify");
        assert!(pk.verify(b"tampered", &sig, false).is_err());

        let _ = std::fs::remove_file(&file);
        let _ = std::fs::remove_file(&sig_path);
    }

    /// An encrypted key signs only with its password: the
    /// release step passes MINISIGN_PASSWORD for exactly
    /// this case.
    #[test]
    fn encrypted_key_needs_its_password() {
        let kp =
            minisign::KeyPair::generate_encrypted_keypair(Some("test-pw".to_string())).unwrap();
        let key_text = kp.sk.to_box(None).unwrap().into_string();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let file = std::env::temp_dir().join(format!(
            "blind_designer_xtask_sign_enc_{}_{nanos}.bin",
            std::process::id()
        ));
        std::fs::write(&file, b"release bytes").unwrap();
        assert!(sign_file_with(&file, &key_text, None).is_err());
        sign_file_with(&file, &key_text, Some("test-pw")).unwrap();
        let _ = std::fs::remove_file(&file);
        let mut sig_path = file.as_os_str().to_owned();
        sig_path.push(".minisig");
        let _ = std::fs::remove_file(&sig_path);
    }
}

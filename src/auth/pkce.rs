use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::distr::Alphanumeric;
use rand::{RngExt, rng};
use sha2::{Digest, Sha256};

const OPAQUE_LENGTH: usize = 32;
const PKCE_VERIFIER_LENGTH: usize = 96;

#[derive(Debug, Clone)]
pub struct PkceBundle {
    pub verifier: String,
    pub challenge: String,
    pub method: &'static str,
}

pub fn generate_pkce() -> PkceBundle {
    let verifier = random_token(PKCE_VERIFIER_LENGTH);
    let challenge = pkce_challenge(&verifier);

    PkceBundle {
        verifier,
        challenge,
        method: "S256",
    }
}

pub fn generate_state() -> String {
    random_token(OPAQUE_LENGTH)
}

pub fn generate_nonce() -> String {
    random_token(OPAQUE_LENGTH)
}

pub fn pkce_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

fn random_token(len: usize) -> String {
    rng()
        .sample_iter(Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

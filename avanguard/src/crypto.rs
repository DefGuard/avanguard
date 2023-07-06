use tiny_keccak::{Hasher, Keccak};

/// Compute the Keccak-256 hash of input bytes.
#[must_use]
pub fn keccak256(bytes: &[u8]) -> [u8; 32] {
    let mut output = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(bytes);
    hasher.finalize(&mut output);
    output
}

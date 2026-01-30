//! Canton Party ID format: partyHint::fingerprint.
//! See research/09, 06.

use canton_core::PartyId;
use canton_crypto::keystore::KeyFingerprint;

/// Build Canton external party ID from hint and fingerprint (hex).
pub fn canton_party_id(party_hint: &str, fingerprint_hex: &str) -> PartyId {
    PartyId::new_unchecked(format!("{}::{}", party_hint, fingerprint_hex))
}

/// Build from KeyFingerprint (requires hex representation from canton-crypto).
pub fn canton_party_id_from_fingerprint(party_hint: &str, fingerprint: &KeyFingerprint) -> PartyId {
    let hex = fingerprint.0.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    PartyId::new_unchecked(format!("{}::{}", party_hint, hex))
}

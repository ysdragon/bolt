// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! Encryption/HMAC — AES-GCM and HMAC-SHA256

use aes_gcm::aead::generic_array::typenum::U12;
use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use hmac::Mac;
use ring_lang_rs::*;

type HmacSha256 = hmac::Hmac<sha2::Sha256>;

const ARGON2_SALT_LEN: usize = 16;

/// Derive a 32-byte AES key from a password-like string key using Argon2id
/// and a random 16-byte salt. String keys are NOT raw AES keys; stretching
/// with a per-encryption salt defeats precomputed password guessing.
fn derive_key_with_argon2(key_str: &str, salt: &[u8; ARGON2_SALT_LEN]) -> [u8; 32] {
    let params = argon2::Params::new(19 * 1024, 2, 1, Some(32)).unwrap();
    let argon2 = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let mut out = [0u8; 32];
    argon2
        .hash_password_into(key_str.as_bytes(), salt, &mut out)
        .expect("argon2 key derivation must not fail");
    out
}

/// Encrypt `plaintext` with `key_str`, returning `salt(16) || nonce(12) || ciphertext`
/// for string keys, or the legacy `nonce(12) || ciphertext` for raw 32-byte keys.
fn aes_encrypt_bytes(plaintext: &[u8], key_str: &str) -> Result<Vec<u8>, String> {
    let kb = key_str.as_bytes();
    let salted = kb.len() != 32;

    let mut salt_bytes = [0u8; ARGON2_SALT_LEN];
    let mut key_bytes = [0u8; 32];
    if salted {
        OsRng.fill_bytes(&mut salt_bytes);
        key_bytes = derive_key_with_argon2(key_str, &salt_bytes);
    } else {
        key_bytes.copy_from_slice(kb);
    }

    let key = <&Key<Aes256Gcm>>::from(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    // NOTE: Random 96-bit nonce. Birthday bound warns of collision at ~2^32 encryptions
    // under the same key. Rotate keys before reaching this limit.
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = <&Nonce<U12>>::from(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|_| "AES encryption failed".to_string())?;

    let mut combined = Vec::with_capacity(salt_bytes.len() + nonce_bytes.len() + ciphertext.len());
    if salted {
        combined.extend_from_slice(&salt_bytes);
    }
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);
    Ok(combined)
}

/// Decrypt the output of `aes_encrypt_bytes`, detecting the format from the
/// key: string keys use the salted format, raw 32-byte keys the legacy one.
fn aes_decrypt_bytes(combined: &[u8], key_str: &str) -> Result<Vec<u8>, String> {
    let kb = key_str.as_bytes();
    let salted = kb.len() != 32;

    let min_len = if salted { ARGON2_SALT_LEN + 12 } else { 12 };
    if combined.len() < min_len {
        return Err("ciphertext too short".to_string());
    }

    let mut key_bytes = [0u8; 32];
    let nonce_offset;
    if salted {
        let mut salt = [0u8; ARGON2_SALT_LEN];
        salt.copy_from_slice(&combined[..ARGON2_SALT_LEN]);
        key_bytes = derive_key_with_argon2(key_str, &salt);
        nonce_offset = ARGON2_SALT_LEN;
    } else {
        key_bytes.copy_from_slice(kb);
        nonce_offset = 0;
    }

    let nonce_bytes = &combined[nonce_offset..nonce_offset + 12];
    let ciphertext = &combined[nonce_offset + 12..];
    let key = <&Key<Aes256Gcm>>::from(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = <&Nonce<U12>>::from(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| "AES decryption failed (wrong key or corrupted data)".to_string())
}

/// bolt_aes_encrypt(plaintext, key) → string (base64 encoded [salt] nonce+ciphertext)
///
/// String keys (anything but a raw 32-byte key) are stretched with Argon2id
/// using a random 16-byte salt prepended to the output:
/// `salt(16) || nonce(12) || ciphertext`. A raw 32-byte key is used directly
/// with the legacy `nonce(12) || ciphertext` format. NOTE: ciphertexts
/// produced with a string key by older versions will NOT decrypt.
ring_func!(bolt_aes_encrypt, |p| {
    ring_check_paracount!(p, 2);
    ring_check_string!(p, 1);
    ring_check_string!(p, 2);
    let plaintext = ring_get_string!(p, 1);
    let key_str = ring_get_string!(p, 2);

    match aes_encrypt_bytes(plaintext.as_bytes(), key_str) {
        Ok(combined) => {
            use base64::Engine;
            let encoded = base64::engine::general_purpose::STANDARD.encode(&combined);
            ring_ret_string!(p, &encoded);
        }
        Err(e) => {
            ring_error!(p, &e);
        }
    }
});

/// bolt_aes_decrypt(ciphertext_b64, key) → string (base64 encoded plaintext)
///
/// Reads the salt prefix when the key is a string key (see `bolt_aes_encrypt`).
/// Raises an error on invalid base64, wrong key, or corrupted data.
ring_func!(bolt_aes_decrypt, |p| {
    ring_check_paracount!(p, 2);
    ring_check_string!(p, 1);
    ring_check_string!(p, 2);
    let ciphertext_b64 = ring_get_string!(p, 1);
    let key_str = ring_get_string!(p, 2);

    use base64::Engine;
    let combined = match base64::engine::general_purpose::STANDARD.decode(ciphertext_b64) {
        Ok(v) => v,
        Err(_) => {
            ring_error!(p, "Invalid base64 ciphertext");
            return;
        }
    };

    match aes_decrypt_bytes(&combined, key_str) {
        Ok(plaintext) => {
            use base64::Engine;
            let encoded = base64::engine::general_purpose::STANDARD.encode(&plaintext);
            ring_ret_string!(p, &encoded);
        }
        Err(e) => {
            // Surface the failure (wrong key / corrupted data) as an error
            // instead of an empty string, so callers can distinguish it from
            // a legitimately empty plaintext.
            ring_error!(p, &e);
        }
    }
});

/// bolt_hmac_sha256(message, key) → string (hex)
ring_func!(bolt_hmac_sha256, |p| {
    ring_check_paracount!(p, 2);
    ring_check_string!(p, 1);
    ring_check_string!(p, 2);
    let message = ring_get_string!(p, 1);
    let key = ring_get_string!(p, 2);

    let mut mac = match <HmacSha256 as Mac>::new_from_slice(key.as_bytes()) {
        Ok(m) => m,
        Err(_) => {
            ring_ret_string!(p, "");
            return;
        }
    };
    mac.update(message.as_bytes());
    let result = mac.finalize();
    let hex_str = hex::encode(result.into_bytes());
    ring_ret_string!(p, &hex_str);
});

/// bolt_hmac_verify(message, key, signature_hex) → 0/1
ring_func!(bolt_hmac_verify, |p| {
    ring_check_paracount!(p, 3);
    ring_check_string!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);
    let message = ring_get_string!(p, 1);
    let key = ring_get_string!(p, 2);
    let sig_hex = ring_get_string!(p, 3);

    let sig_bytes = match hex::decode(sig_hex) {
        Ok(v) => v,
        Err(_) => {
            ring_ret_number!(p, 0.0);
            return;
        }
    };

    let mut mac = match <HmacSha256 as Mac>::new_from_slice(key.as_bytes()) {
        Ok(m) => m,
        Err(_) => {
            ring_ret_number!(p, 0.0);
            return;
        }
    };
    mac.update(message.as_bytes());
    let ok = mac.verify_slice(&sig_bytes).is_ok();
    ring_ret_number!(p, if ok { 1.0 } else { 0.0 });
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argon2_derive_deterministic_with_salt() {
        let salt = [7u8; ARGON2_SALT_LEN];
        let a = derive_key_with_argon2("hunter2", &salt);
        let b = derive_key_with_argon2("hunter2", &salt);
        assert_eq!(a, b);
        let other = [9u8; ARGON2_SALT_LEN];
        let c = derive_key_with_argon2("hunter2", &other);
        assert_ne!(a, c);
        let d = derive_key_with_argon2("hunter3", &salt);
        assert_ne!(a, d);
    }

    #[test]
    fn test_aes_roundtrip_small_string_key() {
        let combined = aes_encrypt_bytes(b"secret payload", "hunter2").unwrap();
        assert_eq!(combined.len(), ARGON2_SALT_LEN + 12 + 14 + 16);
        let plain = aes_decrypt_bytes(&combined, "hunter2").unwrap();
        assert_eq!(plain, b"secret payload");
    }

    #[test]
    fn test_aes_encrypt_twice_different_outputs() {
        let a = aes_encrypt_bytes(b"same input", "hunter2").unwrap();
        let b = aes_encrypt_bytes(b"same input", "hunter2").unwrap();
        assert_ne!(a, b);
        assert_eq!(
            aes_decrypt_bytes(&a, "hunter2").unwrap(),
            aes_decrypt_bytes(&b, "hunter2").unwrap()
        );
    }

    #[test]
    fn test_aes_32_byte_key_legacy_format() {
        let key = "0123456789abcdef0123456789abcdef"; // exactly 32 bytes
        let combined = aes_encrypt_bytes(b"hello", key).unwrap();
        assert_eq!(combined.len(), 12 + 5 + 16); // no salt prefix
        let plain = aes_decrypt_bytes(&combined, key).unwrap();
        assert_eq!(plain, b"hello");
    }

    #[test]
    fn test_aes_wrong_key_fails() {
        let combined = aes_encrypt_bytes(b"data", "hunter2").unwrap();
        assert!(aes_decrypt_bytes(&combined, "wrong-password").is_err());
    }

    #[test]
    fn test_aes_short_ciphertext_fails() {
        assert!(aes_decrypt_bytes(&[1, 2, 3], "hunter2").is_err());
        assert!(aes_decrypt_bytes(&[1, 2, 3], "0123456789abcdef0123456789abcdef").is_err());
    }
}

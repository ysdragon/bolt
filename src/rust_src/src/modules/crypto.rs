// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! Encryption/HMAC — AES-GCM and HMAC-SHA256

use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use hmac::Mac;
use ring_lang_rs::*;

type HmacSha256 = hmac::Hmac<sha2::Sha256>;

/// bolt_aes_encrypt(plaintext, key) → string (base64 encoded nonce+ciphertext)
ring_func!(bolt_aes_encrypt, |p| {
    ring_check_paracount!(p, 2);
    ring_check_string!(p, 1);
    ring_check_string!(p, 2);
    let plaintext = ring_get_string!(p, 1);
    let key_str = ring_get_string!(p, 2);

    let mut key_bytes = [0u8; 32];
    let kb = key_str.as_bytes();
    if kb.len() == 32 {
        key_bytes.copy_from_slice(kb);
    } else {
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(kb);
        key_bytes.copy_from_slice(&hash);
    }

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    match cipher.encrypt(nonce, plaintext.as_bytes().as_ref()) {
        Ok(ciphertext) => {
            let mut combined = nonce_bytes.to_vec();
            combined.extend_from_slice(&ciphertext);
            use base64::Engine;
            let encoded = base64::engine::general_purpose::STANDARD.encode(&combined);
            ring_ret_string!(p, &encoded);
        }
        Err(_) => {
            ring_ret_string!(p, "");
        }
    }
});

/// bolt_aes_decrypt(ciphertext_b64, key) → string (plaintext)
ring_func!(bolt_aes_decrypt, |p| {
    ring_check_paracount!(p, 2);
    ring_check_string!(p, 1);
    ring_check_string!(p, 2);
    let ciphertext_b64 = ring_get_string!(p, 1);
    let key_str = ring_get_string!(p, 2);

    let mut key_bytes = [0u8; 32];
    let kb = key_str.as_bytes();
    if kb.len() == 32 {
        key_bytes.copy_from_slice(kb);
    } else {
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(kb);
        key_bytes.copy_from_slice(&hash);
    }

    use base64::Engine;
    let combined = match base64::engine::general_purpose::STANDARD.decode(ciphertext_b64) {
        Ok(v) => v,
        Err(_) => {
            ring_ret_string!(p, "");
            return;
        }
    };

    if combined.len() < 12 {
        ring_ret_string!(p, "");
        return;
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    match cipher.decrypt(nonce, ciphertext.as_ref()) {
        Ok(plaintext) => {
            let s = String::from_utf8_lossy(&plaintext).to_string();
            ring_ret_string!(p, &s);
        }
        Err(_) => {
            ring_ret_string!(p, "");
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

    let mut mac = <HmacSha256 as Mac>::new_from_slice(key.as_bytes()).unwrap();
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

    let mut mac = <HmacSha256 as Mac>::new_from_slice(key.as_bytes()).unwrap();
    mac.update(message.as_bytes());
    let ok = mac.verify_slice(&sig_bytes).is_ok();
    ring_ret_number!(p, if ok { 1.0 } else { 0.0 });
});

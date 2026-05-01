// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! Password hashing — argon2, bcrypt, scrypt

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use ring_lang_rs::*;
use scrypt::password_hash::{PasswordHasher as ScryptHasher, PasswordVerifier as ScryptVerifier};
use scrypt::phc::PasswordHash as ScryptPasswordHash;

/// bolt_hash_argon2(password) → string — hash with argon2id, return PHC string
ring_func!(bolt_hash_argon2, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);

    let password = ring_get_string!(p, 1);
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hash) => ring_ret_string!(p, &hash.to_string()),
        Err(_) => {
            ring_ret_string!(p, "");
        }
    }
});

/// bolt_verify_argon2(password, hash) → number (0/1)
ring_func!(bolt_verify_argon2, |p| {
    ring_check_paracount!(p, 2);
    ring_check_string!(p, 1);
    ring_check_string!(p, 2);

    let password = ring_get_string!(p, 1);
    let hash_str = ring_get_string!(p, 2);

    let result = PasswordHash::new(hash_str).ok().and_then(|parsed| {
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .ok()
    });

    ring_ret_number!(p, if result.is_some() { 1.0 } else { 0.0 });
});

/// bolt_hash_bcrypt(password) → string — hash with bcrypt (default cost)
ring_func!(bolt_hash_bcrypt, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);

    let password = ring_get_string!(p, 1);

    match bcrypt::hash(password, bcrypt::DEFAULT_COST) {
        Ok(hash) => ring_ret_string!(p, &hash),
        Err(_) => {
            ring_ret_string!(p, "");
        }
    }
});

/// bolt_verify_bcrypt(password, hash) → number (0/1)
ring_func!(bolt_verify_bcrypt, |p| {
    ring_check_paracount!(p, 2);
    ring_check_string!(p, 1);
    ring_check_string!(p, 2);

    let password = ring_get_string!(p, 1);
    let hash_str = ring_get_string!(p, 2);

    let ok = bcrypt::verify(password, hash_str).unwrap_or(false);
    ring_ret_number!(p, if ok { 1.0 } else { 0.0 });
});

/// bolt_hash_scrypt(password) → string — hash with scrypt
ring_func!(bolt_hash_scrypt, |p| {
    ring_check_paracount!(p, 1);
    ring_check_string!(p, 1);

    let password = ring_get_string!(p, 1);
    let scrypt = scrypt::Scrypt::default();

    match scrypt.hash_password(password.as_bytes()) {
        Ok(hash) => ring_ret_string!(p, &hash.to_string()),
        Err(_) => {
            ring_ret_string!(p, "");
        }
    }
});

/// bolt_verify_scrypt(password, hash) → number (0/1)
ring_func!(bolt_verify_scrypt, |p| {
    ring_check_paracount!(p, 2);
    ring_check_string!(p, 1);
    ring_check_string!(p, 2);

    let password = ring_get_string!(p, 1);
    let hash_str = ring_get_string!(p, 2);

    let scrypt = scrypt::Scrypt::default();
    let result = ScryptPasswordHash::new(hash_str)
        .ok()
        .and_then(|parsed| scrypt.verify_password(password.as_bytes(), &parsed).ok());

    ring_ret_number!(p, if result.is_some() { 1.0 } else { 0.0 });
});

use std::iter;
use rand::Rng;

const TOKEN_CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const TOKEN_SIZE: usize = 32;

pub fn generate() -> String {
    generate_token(TOKEN_CHARSET, TOKEN_SIZE)
}

fn generate_token(charset: &[u8], len: usize) -> String {
    let mut rng = rand::rng();
    let charset_len = charset.len();
    let random_char = || charset[rng.random_range(0..charset_len)] as char;

    iter::repeat_with(random_char).take(len).collect()
}

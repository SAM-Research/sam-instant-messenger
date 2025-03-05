use base64::{prelude::BASE64_STANDARD, Engine as _};
use rand::Rng;

pub fn generate_password<R: Rng>(length: usize, rng: &mut R) -> String {
    let mut password = Vec::with_capacity(length);
    for _ in 0..length {
        password.push(rng.gen());
    }
    let password = BASE64_STANDARD.encode(password);
    password[0..password.len() - 2].to_owned()
}

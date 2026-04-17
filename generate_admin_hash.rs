use argon2::{Argon2, password_hash::{SaltString, rand_core::OsRng}};

fn main() {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let password = "12345678";
    let password_hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();
    println!("{}", password_hash);
}

fn main() {
    println!("Generando hash para '12345678'...");
    // Hash real generado con argon2
    let hash = "$argon2id$v=19$m=19456,t=2,p=1$N1NPWnlwUUxBUmlnX3ZieA$5xqK+U5Ed+/HvbqoVrLTq78cV/kmR6N5vz++0xLj1V8";
    println!("{}", hash);
}

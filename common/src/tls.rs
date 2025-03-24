use std::{fs::File, io::BufReader};

use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, private_key};

pub fn load_certs(path: &str) -> Result<Vec<CertificateDer<'static>>, std::io::Error> {
    let cert_file = File::open(path)?;
    let mut reader = BufReader::new(cert_file);
    certs(&mut reader).collect()
}

pub fn load_private_key(path: &str) -> Result<Option<PrivateKeyDer<'static>>, std::io::Error> {
    let key_file = File::open(path)?;
    let mut key_reader = BufReader::new(key_file);
    private_key(&mut key_reader)
}

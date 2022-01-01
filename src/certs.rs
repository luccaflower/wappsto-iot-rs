use openssl::{
    pkey::{PKey, Private},
    rsa::Rsa,
    x509::X509,
};
use std::error::Error;
use uuid::Uuid;
use x509_parser::pem::parse_x509_pem;

pub struct Certs {
    pub id: Uuid,
    pub ca: X509,
    pub certificate: X509,
    pub private_key: PKey<Private>,
}

impl Certs {
    pub fn new(ca: &str, certificate: &str, private_key: &str) -> Result<Self, Box<dyn Error>> {
        let certificate_raw = certificate.as_bytes();
        let pem = parse_x509_pem(certificate_raw)?.1;
        let id = Uuid::parse_str(
            pem.parse_x509()?
                .subject()
                .iter_common_name()
                .next()
                .unwrap()
                .as_str()?,
        )?;
        Ok(Self {
            id,
            ca: X509::from_pem(ca.as_bytes()).unwrap(),
            certificate: X509::from_pem(certificate_raw).unwrap(),
            private_key: PKey::from_rsa(Rsa::private_key_from_pem(private_key.as_bytes()).unwrap())
                .unwrap(),
        })
    }
}

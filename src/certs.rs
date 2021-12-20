use std::error::Error;
use uuid::Uuid;
use x509_parser::pem::parse_x509_pem;

pub struct Certs {
    pub id: Uuid,
    pub ca: String,
    pub certificate: String,
    pub private_key: String,
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
            ca: String::from(ca),
            certificate: String::from(certificate),
            private_key: String::from(private_key),
        })
    }
}

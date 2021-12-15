use std::error::Error;
use uuid::Uuid;
use x509_parser::pem::parse_x509_pem;

pub struct Certs<'a> {
    pub id: Uuid,
    pub ca: &'a str,
    pub certificate: &'a str,
    pub private_key: &'a str,
}

impl<'a> Certs<'a> {
    pub fn new(
        ca: &'a str,
        certificate: &'a str,
        private_key: &'a str,
    ) -> Result<Self, Box<dyn Error>> {
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
            ca,
            certificate,
            private_key,
        })
    }
}

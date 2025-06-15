use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::{BufReader, BufWriter}};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::client::danger::{ServerCertVerified, HandshakeSignatureValid};
use sha2::{Sha256, Digest};

/// The result of a TOFU verification.
pub enum TofuResult {
    /// The host is known and the certificate matches.
    Match,
    /// The host is known but the certificate does not match.
    Mismatch,
    /// The host is unknown.
    Unknown,
    /// The host was just learned.
    New,
}

/// A trust-on-first-use (TOFU) store for hostnames and their certificate fingerprints.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TofuStore {
    path: String,
    known_hosts: HashMap<String, String>, // hostname -> fingerprint
}

impl TofuStore {
    /// Load a TOFU store from a file.
    fn load_from_disk(path: String) -> Self {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let tofu: TofuStore = serde_json::from_reader(reader).unwrap();

        tofu
    }

    /// Create a new TOFU store, loading from a file if it already exists.
    pub fn new(path: String) -> Result<Self, String> {
        if !path.ends_with(".json") {
            return Err("Tofu store path must end with .json".to_string());
        }

        Ok(if std::fs::exists(&path).unwrap() {
            Self::load_from_disk(path)
        } else {
            Self { path, known_hosts: HashMap::new() }
        })
    }

    /// Save the TOFU store to a file.
    fn save_to_disk(&self) -> Result<(), String> {
        let file = File::create(&self.path).map_err(|e| e.to_string())?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self).map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Save a new host and its fingerprint to the store.
    fn learn_host(&mut self, hostname: String, fingerprint: String) -> Result<(), String> {
        self.known_hosts.insert(hostname, fingerprint);

        self.save_to_disk()
    }

    /// Verify that the fingerprint of the received certificate matches the known fingerprint for the hostname.
    fn verify_host(&self, hostname: &String, claimed_fingerprint: &String) -> TofuResult {
        let known_fingerprint = self.known_hosts.get(hostname);

        match known_fingerprint {
            Some(fingerprint) if fingerprint == claimed_fingerprint => TofuResult::Match,
            Some(_) => TofuResult::Mismatch,
            None => TofuResult::Unknown,
        }
    }

    /// Verify that the fingerprint of the received certificate matches the known fingerprint for the hostname, or learn the host if it is unknown.
    /// If the host is known but the certificate does not match, return a mismatch.
    pub fn verify_or_learn_host(&mut self, hostname: &String, claimed_fingerprint: &String) -> Result<TofuResult, String> {
        match self.verify_host(hostname, claimed_fingerprint) {
            TofuResult::Match => Ok(TofuResult::Match),
            TofuResult::Mismatch => Ok(TofuResult::Mismatch),
            TofuResult::Unknown => {
                self.learn_host(hostname.clone(), claimed_fingerprint.clone())?;

                Ok(TofuResult::New)
            }
            TofuResult::New => unreachable!(),
        }
    }
}

/// A TOFU `ServerCertVerifier` for TLS connections.
#[derive(Debug)]
pub struct TofuVerifier {
    store: std::sync::RwLock<TofuStore>,
}

impl TofuVerifier {
    pub fn new(store: TofuStore) -> Self {
        Self { store: std::sync::RwLock::new(store) }
    }
}

impl rustls::client::danger::ServerCertVerifier for TofuVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        // get the hostname from the server name
        let hostname = match server_name {
            ServerName::DnsName(dns_name) => dns_name.as_ref().to_string(),
            _ => return Err(rustls::Error::InvalidCertificate(rustls::CertificateError::NotValidForName)),
        };

        // calculate the certificate fingerprint using SHA-256
        let fingerprint = Sha256::digest(end_entity.as_ref()).to_vec();
        let fingerprint = hex::encode(fingerprint);

        // verify or learn the host
        match self.store.write().unwrap().verify_or_learn_host(&hostname, &fingerprint) {
            Ok(TofuResult::Match) => Ok(ServerCertVerified::assertion()),
            Ok(TofuResult::New) => Ok(ServerCertVerified::assertion()),
            Ok(TofuResult::Mismatch) => Err(rustls::Error::InvalidCertificate(rustls::CertificateError::NotValidForName)),
            Ok(TofuResult::Unknown) => unreachable!(),
            Err(_) => Err(rustls::Error::InvalidCertificate(rustls::CertificateError::NotValidForName)),
        }
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::ED448,
        ]
    }
}
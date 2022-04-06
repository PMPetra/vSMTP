use vsmtp_common::re::{anyhow, log};

use crate::{config::ConfigServerTls, ConfigServerVirtual};

struct TlsLogger;
impl rustls::KeyLog for TlsLogger {
    fn log(&self, label: &str, client_random: &[u8], secret: &[u8]) {
        log::trace!("{} {:?} {:?}", label, client_random, secret);
    }
}

struct CertResolver {
    sni_resolver: rustls::server::ResolvesServerCertUsingSni,
    cert: Option<std::sync::Arc<rustls::sign::CertifiedKey>>,
}

impl rustls::server::ResolvesServerCert for CertResolver {
    fn resolve(
        &self,
        client_hello: rustls::server::ClientHello,
    ) -> Option<std::sync::Arc<rustls::sign::CertifiedKey>> {
        self.sni_resolver
            .resolve(client_hello)
            .or_else(|| self.cert.clone())
    }
}

static JUST_TLS1_2: &[&rustls::SupportedProtocolVersion] = &[&rustls::version::TLS12];

static JUST_TLS1_3: &[&rustls::SupportedProtocolVersion] = &[&rustls::version::TLS13];

static ALL_VERSIONS: &[&rustls::SupportedProtocolVersion] =
    &[&rustls::version::TLS13, &rustls::version::TLS12];

#[doc(hidden)]
pub fn get_rustls_config(
    config: &ConfigServerTls,
    virtual_entries: &[ConfigServerVirtual],
) -> anyhow::Result<rustls::ServerConfig> {
    let protocol_version = match (
        config
            .protocol_version
            .iter()
            .any(|i| i.get_u16() == rustls::ProtocolVersion::TLSv1_2.get_u16()),
        config
            .protocol_version
            .iter()
            .any(|i| i.get_u16() == rustls::ProtocolVersion::TLSv1_3.get_u16()),
    ) {
        (true, true) => Some(ALL_VERSIONS),
        (true, false) => Some(JUST_TLS1_2),
        (false, true) => Some(JUST_TLS1_3),
        (false, false) => None,
    }
    .ok_or_else(|| anyhow::anyhow!("requested version is not supported"))?;

    let mut out = rustls::ServerConfig::builder()
        .with_cipher_suites(rustls::ALL_CIPHER_SUITES)
        .with_kx_groups(&rustls::ALL_KX_GROUPS)
        .with_protocol_versions(protocol_version)
        .map_err(|e| anyhow::anyhow!("cannot initialize tls config: '{e}'"))?
        .with_client_cert_verifier(rustls::server::NoClientAuth::new())
        .with_cert_resolver(std::sync::Arc::new(CertResolver {
            sni_resolver: virtual_entries.iter().fold(
                anyhow::Ok(rustls::server::ResolvesServerCertUsingSni::new()),
                |sni_resolver, entry| {
                    let mut sni_resolver = sni_resolver?;
                    sni_resolver
                        .add(
                            &entry.domain,
                            rustls::sign::CertifiedKey {
                                cert: vec![entry.tls.certificate.clone()],
                                key: rustls::sign::any_supported_type(&entry.tls.private_key)?,
                                ocsp: None,
                                sct_list: None,
                            },
                        )
                        .map_err(|e| anyhow::anyhow!("cannot add sni to resolver: {e}"))?;
                    Ok(sni_resolver)
                },
            )?,
            cert: Some(std::sync::Arc::new(rustls::sign::CertifiedKey {
                cert: vec![config.certificate.clone()],
                key: rustls::sign::any_supported_type(&config.private_key)?,
                ocsp: None,
                sct_list: None,
            })),
        }));

    out.ignore_client_order = config.preempt_cipherlist;

    out.key_log = std::sync::Arc::new(TlsLogger {});

    Ok(out)
}

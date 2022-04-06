/*
 * vSMTP mail transfer agent
 * Copyright (C) 2022 viridIT SAS
 *
 * This program is free software: you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or any later version.
 *
 *  This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program. If not, see https://www.gnu.org/licenses/.
 *
*/

//! vSMTP delivery system

#![doc(html_no_source)]
#![deny(missing_docs)]
//
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
//
#![allow(clippy::doc_markdown)]

/// a few helpers to create systems that will deliver emails.
pub mod transport {
    use anyhow::Context;
    use lettre::Tokio1Executor;
    use trust_dns_resolver::TokioAsyncResolver;
    use vsmtp_common::{address::Address, mail_context::MessageMetadata, rcpt::Rcpt, re::anyhow};
    use vsmtp_config::Config;

    /// allowing the [ServerVSMTP] to deliver a mail.
    #[async_trait::async_trait]
    pub trait Transport {
        /// the deliver method of the [Resolver] trait
        async fn deliver(
            &mut self,
            config: &Config,
            dns: &TokioAsyncResolver,
            metadata: &MessageMetadata,
            from: &Address,
            to: &mut [Rcpt],
            content: &str,
        ) -> anyhow::Result<()>;
    }

    /// delivery transport.
    pub mod deliver;
    /// forwarding transport.
    pub mod forward;
    /// maildir transport.
    pub mod maildir;
    /// mbox transport.
    pub mod mbox;

    /// no transfer will be made if this resolver is selected.
    pub struct NoTransfer;

    #[async_trait::async_trait]
    impl Transport for NoTransfer {
        async fn deliver(
            &mut self,
            _: &Config,
            _: &TokioAsyncResolver,
            _: &MessageMetadata,
            _: &Address,
            _: &mut [Rcpt],
            _: &str,
        ) -> anyhow::Result<()> {
            Ok(())
        }
    }

    /// build a [lettre] envelop using from address & recipients.
    pub(super) fn build_lettre_envelop(
        from: &vsmtp_common::address::Address,
        rcpt: &[Rcpt],
    ) -> anyhow::Result<lettre::address::Envelope> {
        Ok(lettre::address::Envelope::new(
            Some(
                from.full()
                    .parse()
                    .context("failed to parse from address")?,
            ),
            rcpt.iter()
                // NOTE: address that couldn't be converted will be silently dropped.
                .flat_map(|rcpt| rcpt.address.full().parse::<lettre::Address>())
                .collect(),
        )?)
    }

    /// build a transport using opportunistic tls and toml specified certificates.
    /// TODO: resulting transport should be cached.
    fn build_transport(
        config: &Config,
        from: &vsmtp_common::address::Address,
        target: &str,
    ) -> anyhow::Result<lettre::AsyncSmtpTransport<Tokio1Executor>> {
        let parameters =
            lettre::transport::smtp::client::TlsParameters::builder(target.to_string())
                .add_root_certificate(
                    // if let Some(virtual) = config.virtual.get(from.domain()) {
                    //     match virtual.dane {
                    //         "must" => dns.tlsa_lookup(format!("_{port}._{protocol}.{target}").unwrap()
                    //         _ => todo!()
                    //     }
                    // } else {
                    //     todo!()
                    // }

                    // from's domain could match the root domain of the server.
                    if config.server.domain == from.domain() {
                        lettre::transport::smtp::client::Certificate::from_der(
                            config.server.tls.as_ref().unwrap().certificate.0.clone(),
                        )
                        .context("failed to parse certificate as der")?
                    }
                    // or a domain from one of the virtual domains.
                    else if let Some(v) = config
                        .server
                        .r#virtual
                        .iter()
                        .find(|v| v.domain == from.domain())
                    {
                        lettre::transport::smtp::client::Certificate::from_der(
                            v.tls.certificate.0.clone(),
                        )
                        .context("failed to parse certificate as der")?
                    } else {
                        anyhow::bail!("no certificate found for '{}'", from.domain());
                    },
                )
                .build_rustls()
                .context("failed to build tls parameters")?;

        Ok(
            lettre::AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(target)
                .hello_name(lettre::transport::smtp::extension::ClientId::Domain(
                    from.domain().to_string(),
                ))
                .port(lettre::transport::smtp::SMTP_PORT)
                .tls(lettre::transport::smtp::client::Tls::Opportunistic(
                    parameters,
                ))
                .build(),
        )
    }

    /// create a query to request tlsa records.
    fn create_tlsa_query(target: &str, port: u16, protocol: &str) -> String {
        format!("_{port}._{protocol}.{target}")
    }
}

#[cfg(test)]
pub mod test {
    #[must_use]
    /// create an empty email context for testing purposes.
    pub fn get_default_context() -> vsmtp_common::mail_context::MailContext {
        vsmtp_common::mail_context::MailContext {
            body: vsmtp_common::mail_context::Body::Empty,
            connection_timestamp: std::time::SystemTime::now(),
            client_addr: std::net::SocketAddr::new(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
                0,
            ),
            envelop: vsmtp_common::envelop::Envelop::default(),
            metadata: Some(vsmtp_common::mail_context::MessageMetadata {
                timestamp: std::time::SystemTime::now(),
                ..vsmtp_common::mail_context::MessageMetadata::default()
            }),
        }
    }

    use super::transport::build_lettre_envelop;
    use vsmtp_common::{
        address::Address,
        rcpt::Rcpt,
        transfer::{EmailTransferStatus, Transfer},
    };

    #[test]
    fn test_build_lettre_envelop() {
        assert_eq!(
            build_lettre_envelop(
                &Address::try_from("a@a.a".to_string()).unwrap(),
                &[Rcpt {
                    address: Address::try_from("b@b.b".to_string()).unwrap(),
                    transfer_method: Transfer::None,
                    email_status: EmailTransferStatus::Sent
                }]
            )
            .expect("failed to build lettre envelop"),
            lettre::address::Envelope::new(
                Some("a@a.a".parse().unwrap()),
                vec!["b@b.b".parse().unwrap()]
            )
            .unwrap()
        );
    }
}

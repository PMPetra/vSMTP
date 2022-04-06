use vsmtp_common::{code::SMTPReplyCode, collection, re::log};

use crate::Config;

#[test]
fn parse() {
    let toml = include_str!("../../../../examples/config/logging.toml");
    pretty_assertions::assert_eq!(
        Config::from_toml(toml).unwrap(),
        Config::builder()
            .with_version_str("<1.0.0")
            .unwrap()
            .with_hostname()
            .with_default_system()
            .with_ipv4_localhost()
            .with_logs_settings(
                "/var/log/vsmtp/vsmtp.log",
                "{d(%Y-%m-%d %H:%M:%S)} {h({l:<5} {I})} ((line:{L:<3})) $ {m}{n}",
                collection! {
                    "default".to_string() => log::LevelFilter::Warn,
                    "receiver".to_string() => log::LevelFilter::Info,
                    "rule_engine".to_string() => log::LevelFilter::Warn,
                    "delivery".to_string()=> log::LevelFilter::Error,
                }
            )
            .with_default_delivery()
            .without_tls_support()
            .with_default_smtp_options()
            .with_default_smtp_error_handler()
            .with_smtp_codes(collection! {
                SMTPReplyCode::Help => "214 my custom help message\r\n".to_string(),
                SMTPReplyCode::Greetings => "220 {domain} ESMTP Service ready\r\n".to_string(),
            })
            .without_auth()
            .with_default_app()
            .with_default_vsl_settings()
            .with_app_logs_level_and_format(
                "/var/log/vsmtp/app.log",
                log::LevelFilter::Trace,
                "{d} - {m}{n}"
            )
            .without_services()
            .with_system_dns()
            .without_virtual_entries()
            .validate()
            .unwrap()
    );
}

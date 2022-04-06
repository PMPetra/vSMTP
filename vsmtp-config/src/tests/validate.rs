use crate::Config;
use vsmtp_common::{auth::Mechanism, code::SMTPReplyCode};

fn get_mechanism_from_config(config: &Config, tls: bool) -> Vec<Mechanism> {
    let plain_esmtp = config
        .server
        .smtp
        .codes
        .get(if tls {
            &SMTPReplyCode::Code250SecuredEsmtp
        } else {
            &SMTPReplyCode::Code250PlainEsmtp
        })
        .unwrap();

    let auth = plain_esmtp
        .split("\r\n")
        .find(|s| s.starts_with("250-AUTH") || s.starts_with("250 AUTH"))
        .unwrap();
    let mechanism = auth
        .strip_prefix("250-AUTH")
        .or_else(|| auth.strip_prefix("250 AUTH"))
        .unwrap();

    mechanism
        .split_whitespace()
        .map(<Mechanism as std::str::FromStr>::from_str)
        .collect::<Result<Vec<_>, <Mechanism as std::str::FromStr>::Err>>()
        .unwrap()
}

fn get_both(config: &Config) -> (Vec<Mechanism>, Vec<Mechanism>) {
    (
        get_mechanism_from_config(config, false),
        get_mechanism_from_config(config, true),
    )
}

macro_rules! assert_mechanism_list {
    ($required:expr, $clair_expected:expr, $secured_expected:expr) => {
        let config = Config::builder()
            .with_current_version()
            .with_hostname()
            .with_default_system()
            .with_ipv4_localhost()
            .with_default_logs_settings()
            .with_default_delivery()
            .without_tls_support()
            .with_default_smtp_options()
            .with_default_smtp_error_handler()
            .with_default_smtp_codes()
            .with_auth(false, false, $required.to_vec(), -1)
            .with_default_app()
            .with_default_vsl_settings()
            .with_default_app_logs()
            .without_services()
            .with_system_dns()
            .without_virtual_entries()
            .validate()
            .unwrap();

        let (clair, secured) = get_both(&config);
        assert_eq!(clair, $clair_expected);
        assert_eq!(secured, $secured_expected);
    };
}

#[test]
fn auth_mechanism() {
    assert_mechanism_list!([], [], []);

    assert_mechanism_list!(
        [Mechanism::Login, Mechanism::Plain],
        [],
        [Mechanism::Login, Mechanism::Plain]
    );

    assert_mechanism_list!(
        [Mechanism::Login, Mechanism::Plain, Mechanism::CramMd5],
        [],
        [Mechanism::Login, Mechanism::Plain, Mechanism::CramMd5]
    );
}

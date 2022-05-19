/*
 * vSMTP mail transfer agent
 * Copyright (C) 2022 viridIT SAS
 *
 * This program is free software: you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or any later version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program. If not, see https://www.gnu.org/licenses/.
 *
*/

use anyhow::Context;
use vsmtp_common::{re::anyhow, CodesID};
use vsmtp_config::Config;
use vsmtp_rule_engine::rule_engine::RuleEngine;
use vsmtp_server::{auth, handle_connection, re::tokio, Connection, ConnectionKind, OnMail};

/// A type implementing Write+Read to emulate sockets
pub struct Mock<'a, T: AsRef<[u8]> + Unpin> {
    read_cursor: std::io::Cursor<T>,
    write_cursor: std::io::Cursor<&'a mut Vec<u8>>,
}

impl<'a, T: AsRef<[u8]> + Unpin> Mock<'a, T> {
    /// Create an new instance
    pub fn new(read: T, write: &'a mut Vec<u8>) -> Self {
        Self {
            read_cursor: std::io::Cursor::new(read),
            write_cursor: std::io::Cursor::new(write),
        }
    }
}

impl<T: AsRef<[u8]> + Unpin> tokio::io::AsyncRead for Mock<'_, T> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::result::Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut self.read_cursor).poll_read(cx, buf)
    }
}

impl<T: AsRef<[u8]> + Unpin> tokio::io::AsyncWrite for Mock<'_, T> {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        std::task::Poll::Ready(std::io::Write::write(&mut self.write_cursor, buf))
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(std::io::Write::flush(&mut self.write_cursor))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}

/// used for testing, does not do anything once the email is received.
pub struct DefaultMailHandler;

#[async_trait::async_trait]
impl OnMail for DefaultMailHandler {
    async fn on_mail<S: tokio::io::AsyncWrite + tokio::io::AsyncRead + Send + Unpin>(
        &mut self,
        conn: &mut Connection<S>,
        mail: Box<vsmtp_common::mail_context::MailContext>,
        helo_domain: &mut Option<String>,
    ) -> anyhow::Result<()> {
        *helo_domain = Some(mail.envelop.helo.clone());
        conn.send_code(CodesID::Ok).await?;
        Ok(())
    }
}

/// run a connection and assert output produced by vSMTP and @expected_output
///
/// # Errors
///
/// * the outcome of [`handle_connection`]
///
/// # Panics
///
/// * argument provided are ill-formed
pub async fn test_receiver_inner<M>(
    address: &str,
    mail_handler: &mut M,
    smtp_input: &[u8],
    expected_output: &[u8],
    config: std::sync::Arc<Config>,
    rsasl: Option<std::sync::Arc<tokio::sync::Mutex<auth::Backend>>>,
) -> anyhow::Result<()>
where
    M: OnMail + Send,
{
    let mut written_data = Vec::new();
    let mut mock = Mock::new(smtp_input.to_vec(), &mut written_data);
    let mut conn = Connection::new(
        ConnectionKind::Opportunistic,
        address.parse().unwrap(),
        config.clone(),
        &mut mock,
    );

    let rule_engine = std::sync::Arc::new(std::sync::RwLock::new(
        RuleEngine::new(&config, &config.app.vsl.filepath.clone())
            .context("failed to initialize the engine")
            .unwrap(),
    ));

    let result = handle_connection(&mut conn, None, rsasl, rule_engine, mail_handler).await;
    tokio::io::AsyncWriteExt::flush(&mut conn.inner.inner)
        .await
        .unwrap();

    pretty_assertions::assert_eq!(
        std::str::from_utf8(expected_output),
        std::str::from_utf8(&written_data),
    );

    result
}

/// Call test_receiver_inner
#[macro_export]
macro_rules! test_receiver {
    ($input:expr, $output:expr) => {
        test_receiver! {
            on_mail => &mut $crate::receiver::DefaultMailHandler {},
            with_config => $crate::config::local_test(),
            $input,
            $output
        }
    };
    (on_mail => $resolver:expr, $input:expr, $output:expr) => {
        test_receiver! {
            on_mail => $resolver,
            with_config => $crate::config::local_test(),
            $input,
            $output
        }
    };
    (with_config => $config:expr, $input:expr, $output:expr) => {
        test_receiver! {
            on_mail => &mut $crate::receiver::DefaultMailHandler {},
            with_config => $config,
            $input,
            $output
        }
    };
    (on_mail => $resolver:expr, with_config => $config:expr, $input:expr, $output:expr) => {
        $crate::receiver::test_receiver_inner(
            "127.0.0.1:0",
            $resolver,
            $input.as_bytes(),
            $output.as_bytes(),
            std::sync::Arc::new($config),
            None,
        )
        .await
    };
    (with_auth => $auth:expr, with_config => $config:expr, $input:expr, $output:expr) => {
        test_receiver! {
            with_auth => $auth,
            with_config => $config,
            on_mail => &mut $crate::receiver::DefaultMailHandler {},
            $input,
            $output
        }
    };
    (with_auth => $auth:expr, with_config => $config:expr, on_mail => $resolver:expr, $input:expr, $output:expr) => {
        $crate::receiver::test_receiver_inner(
            "127.0.0.1:0",
            $resolver,
            $input.as_bytes(),
            $output.as_bytes(),
            std::sync::Arc::new($config),
            Some(std::sync::Arc::new(tokio::sync::Mutex::new($auth))),
        )
        .await
    };
}

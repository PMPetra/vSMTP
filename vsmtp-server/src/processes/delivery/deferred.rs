use crate::{processes::delivery::send_email, queue::Queue};
use trust_dns_resolver::TokioAsyncResolver;
use vsmtp_common::{
    mail_context::MailContext,
    rcpt::Rcpt,
    re::{
        anyhow::{self, Context},
        log,
    },
    transfer::EmailTransferStatus,
};
use vsmtp_config::{log_channel::DELIVER, Config};

pub async fn flush_deferred_queue(config: &Config, dns: &TokioAsyncResolver) -> anyhow::Result<()> {
    let dir_entries = std::fs::read_dir(Queue::Deferred.to_path(&config.server.queues.dirpath)?)?;
    for path in dir_entries {
        if let Err(e) = handle_one_in_deferred_queue(config, dns, &path?.path()).await {
            log::warn!("{}", e);
        }
    }

    Ok(())
}

// NOTE: emails stored in the deferred queue are likely to slow down the process.
//       the pickup process of this queue should be slower than pulling from the delivery queue.
//       https://www.postfix.org/QSHAPE_README.html#queues
async fn handle_one_in_deferred_queue(
    config: &Config,
    dns: &TokioAsyncResolver,
    path: &std::path::Path,
) -> anyhow::Result<()> {
    let message_id = path.file_name().and_then(std::ffi::OsStr::to_str).unwrap();

    log::debug!(
        target: DELIVER,
        "vDeliver (deferred) processing email '{}'",
        message_id
    );

    let mut ctx = MailContext::from_file(path).with_context(|| {
        format!(
            "failed to deserialize email in deferred queue '{}'",
            &message_id
        )
    })?;

    let max_retry_deferred = config.server.queues.delivery.deferred_retry_max;

    let metadata = ctx
        .metadata
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("email metadata not available in deferred email"))?;

    // TODO: at this point, only HeldBack recipients should be present in the queue.
    //       check if it is true or not.
    ctx.envelop.rcpt = send_email(
        config,
        dns,
        metadata,
        &ctx.envelop.mail_from,
        &ctx.envelop.rcpt,
        &ctx.body,
    )
    .await
    .context("failed to send emails from the deferred queue")?;

    // updating retry count, set status to Failed if threshold reached.
    ctx.envelop.rcpt = ctx
        .envelop
        .rcpt
        .into_iter()
        .map(|rcpt| Rcpt {
            email_status: match rcpt.email_status {
                EmailTransferStatus::HeldBack(count) if count >= max_retry_deferred => {
                    EmailTransferStatus::Failed(format!(
                        "maximum retry count of '{max_retry_deferred}' reached"
                    ))
                }
                EmailTransferStatus::HeldBack(count) => EmailTransferStatus::HeldBack(count + 1),
                status => EmailTransferStatus::Failed(format!(
                    "wrong recipient status '{status}' found in the deferred queue"
                )),
            },
            ..rcpt
        })
        .collect();

    if ctx
        .envelop
        .rcpt
        .iter()
        .any(|rcpt| matches!(rcpt.email_status, EmailTransferStatus::HeldBack(..)))
    {
        // if there is still recipients left to send the email to, we just update the recipient list on disk.
        Queue::Deferred.write_to_queue(config, &ctx)?;
    } else {
        // otherwise, we remove the file from the deferred queue.
        std::fs::remove_file(&path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::handle_one_in_deferred_queue;
    use crate::queue::Queue;
    use vsmtp_common::{
        address::Address,
        envelop::Envelop,
        mail_context::{Body, MailContext, MessageMetadata},
        rcpt::Rcpt,
        transfer::{EmailTransferStatus, Transfer},
    };
    use vsmtp_config::build_dns;
    use vsmtp_test::config;

    #[tokio::test]
    async fn basic() {
        let mut config = config::local_test();
        config.server.queues.dirpath = "./tmp".into();
        config.app.vsl.filepath = "./src/tests/empty_main.vsl".into();

        let now = std::time::SystemTime::now();

        Queue::Deferred
            .write_to_queue(
                &config,
                &MailContext {
                    connection_timestamp: now,
                    client_addr: "127.0.0.1:80".parse().unwrap(),
                    envelop: Envelop {
                        helo: "client.com".to_string(),
                        mail_from: Address::try_from("from@client.com".to_string()).unwrap(),
                        rcpt: vec![
                            Rcpt {
                                address: Address::try_from("to+1@client.com".to_string()).unwrap(),
                                transfer_method: Transfer::Maildir,
                                email_status: EmailTransferStatus::Waiting,
                            },
                            Rcpt {
                                address: Address::try_from("to+2@client.com".to_string()).unwrap(),
                                transfer_method: Transfer::Maildir,
                                email_status: EmailTransferStatus::Waiting,
                            },
                        ],
                    },
                    body: Body::Raw("Date: bar\r\nFrom: foo\r\nHello world\r\n".to_string()),
                    metadata: Some(MessageMetadata {
                        timestamp: now,
                        message_id: "test".to_string(),
                        skipped: None,
                    }),
                },
            )
            .unwrap();

        let dns = build_dns(&config).unwrap();

        handle_one_in_deferred_queue(
            &config,
            &dns,
            &config.server.queues.dirpath.join("deferred/test"),
        )
        .await
        .unwrap();

        pretty_assertions::assert_eq!(
            MailContext::from_file(&config.server.queues.dirpath.join("deferred/test")).unwrap(),
            MailContext {
                connection_timestamp: now,
                client_addr: "127.0.0.1:80".parse().unwrap(),
                envelop: Envelop {
                    helo: "client.com".to_string(),
                    mail_from: Address::try_from("from@client.com".to_string()).unwrap(),
                    rcpt: vec![
                        Rcpt {
                            address: Address::try_from("to+1@client.com".to_string()).unwrap(),
                            transfer_method: Transfer::Maildir,
                            email_status: EmailTransferStatus::HeldBack(1),
                        },
                        Rcpt {
                            address: Address::try_from("to+2@client.com".to_string()).unwrap(),
                            transfer_method: Transfer::Maildir,
                            email_status: EmailTransferStatus::HeldBack(1),
                        },
                    ],
                },
                body: Body::Raw("Date: bar\r\nFrom: foo\r\nHello world\r\n".to_string()),
                metadata: Some(MessageMetadata {
                    timestamp: now,
                    message_id: "test".to_string(),
                    skipped: None,
                }),
            }
        );
    }
}

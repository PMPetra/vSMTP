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
use crate::{queue::Queue, ProcessMessage};
use anyhow::Context;
use vsmtp_common::{
    mail_context::{Body, MailContext},
    re::{anyhow, log},
    state::StateSMTP,
    status::Status,
};
use vsmtp_config::{log_channel::DELIVER, Config};
use vsmtp_mail_parser::MailMimeParser;
use vsmtp_rule_engine::rule_engine::{RuleEngine, RuleState};

/// process that treats incoming email offline with the postq stage.
///
/// # Errors
pub async fn start(
    config: std::sync::Arc<Config>,
    rule_engine: std::sync::Arc<std::sync::RwLock<RuleEngine>>,
    mut working_receiver: tokio::sync::mpsc::Receiver<ProcessMessage>,
    delivery_sender: tokio::sync::mpsc::Sender<ProcessMessage>,
) -> anyhow::Result<()> {
    loop {
        if let Some(pm) = working_receiver.recv().await {
            if let Err(err) = tokio::spawn(handle_one_in_working_queue(
                config.clone(),
                rule_engine.clone(),
                pm,
                delivery_sender.clone(),
            ))
            .await
            {
                log::error!("{}", err);
            }
        }
    }
}

///
/// # Errors
///
/// # Panics
async fn handle_one_in_working_queue(
    config: std::sync::Arc<Config>,
    rule_engine: std::sync::Arc<std::sync::RwLock<RuleEngine>>,
    process_message: ProcessMessage,
    delivery_sender: tokio::sync::mpsc::Sender<ProcessMessage>,
) -> anyhow::Result<()> {
    log::debug!(
        target: DELIVER,
        "vMIME process received a new message id: {}",
        process_message.message_id,
    );

    let file_to_process = Queue::Working
        .to_path(&config.server.queues.dirpath)?
        .join(&process_message.message_id);

    log::debug!(target: DELIVER, "vMIME opening file: {:?}", file_to_process);

    let mut ctx = MailContext::from_file(&file_to_process).context(format!(
        "failed to deserialize email in working queue '{}'",
        file_to_process.display()
    ))?;

    if let Body::Raw(raw) = &ctx.body {
        ctx.body = Body::Parsed(Box::new(MailMimeParser::default().parse(raw.as_bytes())?));
    }
    let mut state = RuleState::with_context(config.as_ref(), ctx);

    let result = rule_engine
        .read()
        .map_err(|_| anyhow::anyhow!("rule engine mutex poisoned"))?
        .run_when(&mut state, &StateSMTP::PostQ);

    if result == Status::Deny {
        Queue::Dead.write_to_queue(config.as_ref(), &state.get_context().read().unwrap())?;
    } else {
        // using a bool to prevent the lock guard to reach the await call below.
        let delivered = {
            let ctx = state.get_context();
            let ctx = ctx.read().unwrap();

            if ctx
                .envelop
                .rcpt
                .iter()
                .all(|rcpt| rcpt.transfer_method == vsmtp_common::transfer::Transfer::None)
            {
                // skipping mime & delivery processes.
                log::warn!(
                    target: DELIVER,
                    "delivery skipped because all recipient's transfer method is set to None."
                );
                Queue::Dead.write_to_queue(config.as_ref(), &ctx)?;
                false
            } else {
                Queue::Deliver
                    .write_to_queue(&config, &ctx)
                    .context(format!(
                        "failed to move '{}' from delivery queue to deferred queue",
                        process_message.message_id
                    ))?;
                true
            }
        };

        if delivered {
            delivery_sender
                .send(ProcessMessage {
                    message_id: process_message.message_id.to_string(),
                })
                .await?;
        }
    };

    std::fs::remove_file(&file_to_process).context(format!(
        "failed to remove '{}' from the working queue",
        process_message.message_id
    ))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::handle_one_in_working_queue;
    use crate::{queue::Queue, ProcessMessage};
    use vsmtp_common::{
        address::Address,
        envelop::Envelop,
        mail_context::{Body, MailContext, MessageMetadata},
        rcpt::Rcpt,
        re::anyhow::Context,
        transfer::{EmailTransferStatus, Transfer},
    };
    use vsmtp_rule_engine::rule_engine::RuleEngine;
    use vsmtp_test::config;

    #[tokio::test]
    async fn cannot_deserialize() {
        let mut config = config::local_test();
        config.app.vsl.filepath = "./src/tests/empty_main.vsl".into();

        let (delivery_sender, _delivery_receiver) =
            tokio::sync::mpsc::channel::<ProcessMessage>(10);

        let config = std::sync::Arc::new(config);

        assert!(handle_one_in_working_queue(
            config.clone(),
            std::sync::Arc::new(std::sync::RwLock::new(
                RuleEngine::new(&Some(config.app.vsl.filepath.clone()))
                    .context("failed to initialize the engine")
                    .unwrap(),
            )),
            ProcessMessage {
                message_id: "not_such_message_named_like_this".to_string(),
            },
            delivery_sender,
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn basic() {
        let mut config = config::local_test();
        config.server.queues.dirpath = "./tmp".into();
        config.app.vsl.filepath = "./src/tests/empty_main.vsl".into();

        Queue::Working
            .write_to_queue(
                &config,
                &MailContext {
                    connection_timestamp: std::time::SystemTime::now(),
                    client_addr: "127.0.0.1:80".parse().unwrap(),
                    envelop: Envelop {
                        helo: "client.com".to_string(),
                        mail_from: Address::try_from("from@client.com".to_string()).unwrap(),
                        rcpt: vec![
                            Rcpt {
                                address: Address::try_from("to+1@client.com".to_string()).unwrap(),
                                transfer_method: Transfer::Deliver,
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
                        timestamp: std::time::SystemTime::now(),
                        message_id: "test".to_string(),
                        skipped: None,
                    }),
                },
            )
            .unwrap();

        let (delivery_sender, mut delivery_receiver) =
            tokio::sync::mpsc::channel::<ProcessMessage>(10);

        let config = std::sync::Arc::new(config);
        handle_one_in_working_queue(
            config.clone(),
            std::sync::Arc::new(std::sync::RwLock::new(
                RuleEngine::new(&Some(config.app.vsl.filepath.clone()))
                    .context("failed to initialize the engine")
                    .unwrap(),
            )),
            ProcessMessage {
                message_id: "test".to_string(),
            },
            delivery_sender,
        )
        .await
        .unwrap();

        assert_eq!(delivery_receiver.recv().await.unwrap().message_id, "test");
    }
}

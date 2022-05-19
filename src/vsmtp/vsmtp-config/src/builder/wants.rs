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
#![allow(clippy::module_name_repetitions)]

use vsmtp_common::{re::log, CodesID, Reply};

use crate::{
    config::{
        ConfigQueueDelivery, ConfigQueueWorking, ConfigServerDNS, ConfigServerSMTPError,
        ConfigServerSMTPTimeoutClient, ConfigServerTls,
    },
    ConfigServerSMTPAuth, ConfigServerVirtual,
};

///
pub struct WantsVersion(pub(crate) ());

///
pub struct WantsServer {
    #[allow(dead_code)]
    pub(crate) parent: WantsVersion,
    pub(super) version_requirement: semver::VersionReq,
}

///
pub struct WantsServerSystem {
    pub(crate) parent: WantsServer,
    pub(super) domain: String,
    pub(super) client_count_max: i64,
}

///
pub struct WantsServerInterfaces {
    pub(crate) parent: WantsServerSystem,
    pub(super) user: users::User,
    pub(super) group: users::Group,
    pub(super) group_local: Option<users::Group>,
    pub(super) thread_pool_receiver: usize,
    pub(super) thread_pool_processing: usize,
    pub(super) thread_pool_delivery: usize,
}

///
pub struct WantsServerLogs {
    pub(crate) parent: WantsServerInterfaces,
    pub(super) addr: Vec<std::net::SocketAddr>,
    pub(super) addr_submission: Vec<std::net::SocketAddr>,
    pub(super) addr_submissions: Vec<std::net::SocketAddr>,
}

///
pub struct WantsServerQueues {
    pub(crate) parent: WantsServerLogs,
    pub(super) filepath: std::path::PathBuf,
    pub(super) format: String,
    pub(super) level: std::collections::BTreeMap<String, log::LevelFilter>,
    pub(super) size_limit: u64,
    pub(super) archive_count: u32,
}

///
pub struct WantsServerTLSConfig {
    pub(crate) parent: WantsServerQueues,
    pub(super) dirpath: std::path::PathBuf,
    pub(super) working: ConfigQueueWorking,
    pub(super) delivery: ConfigQueueDelivery,
}

///
pub struct WantsServerSMTPConfig1 {
    pub(crate) parent: WantsServerTLSConfig,
    pub(super) tls: Option<ConfigServerTls>,
}

///
pub struct WantsServerSMTPConfig2 {
    pub(crate) parent: WantsServerSMTPConfig1,
    pub(super) rcpt_count_max: usize,
    pub(super) disable_ehlo: bool,
    pub(super) required_extension: Vec<String>,
}

///
pub struct WantsServerSMTPConfig3 {
    pub(crate) parent: WantsServerSMTPConfig2,
    pub(super) error: ConfigServerSMTPError,
    pub(super) timeout_client: ConfigServerSMTPTimeoutClient,
}

///
pub struct WantsServerSMTPAuth {
    pub(crate) parent: WantsServerSMTPConfig3,
    pub(super) codes: std::collections::BTreeMap<CodesID, Reply>,
}

///
pub struct WantsApp {
    pub(crate) parent: WantsServerSMTPAuth,
    pub(super) auth: Option<ConfigServerSMTPAuth>,
}

///
pub struct WantsAppVSL {
    pub(crate) parent: WantsApp,
    pub(super) dirpath: std::path::PathBuf,
}

///
pub struct WantsAppLogs {
    pub(crate) parent: WantsAppVSL,
    pub(super) filepath: Option<std::path::PathBuf>,
}

///
pub struct WantsServerDNS {
    pub(crate) parent: WantsAppLogs,
    pub(super) filepath: std::path::PathBuf,
    pub(super) level: log::LevelFilter,
    pub(super) format: String,
    pub(super) size_limit: u64,
    pub(super) archive_count: u32,
}

///
pub struct WantsServerVirtual {
    pub(crate) parent: WantsServerDNS,
    pub(super) config: ConfigServerDNS,
}

///
pub struct WantsValidate {
    pub(crate) parent: WantsServerVirtual,
    pub(super) r#virtual: std::collections::BTreeMap<String, ConfigServerVirtual>,
}

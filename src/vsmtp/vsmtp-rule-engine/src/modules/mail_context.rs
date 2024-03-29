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
use crate::modules::EngineResult;
use crate::server_api::ServerAPI;
use rhai::plugin::{
    Dynamic, EvalAltResult, FnAccess, FnNamespace, Module, NativeCallContext, PluginFunction,
    RhaiResult, TypeId,
};
use vsmtp_common::mail_context::MailContext;
use vsmtp_common::Address;

#[doc(hidden)]
#[allow(dead_code)]
#[rhai::plugin::export_module]
pub mod mail_context {

    #[rhai_fn(global, get = "client_ip", return_raw, pure)]
    pub fn client_ip(
        this: &mut std::sync::Arc<std::sync::RwLock<MailContext>>,
    ) -> EngineResult<std::net::IpAddr> {
        Ok(this
            .read()
            .map_err::<Box<EvalAltResult>, _>(|e| e.to_string().into())?
            .client_addr
            .ip())
    }

    #[rhai_fn(global, get = "client_port", return_raw, pure)]
    pub fn client_port(
        this: &mut std::sync::Arc<std::sync::RwLock<MailContext>>,
    ) -> EngineResult<i64> {
        Ok(i64::from(
            this.read()
                .map_err::<Box<EvalAltResult>, _>(|e| e.to_string().into())?
                .client_addr
                .port(),
        ))
    }

    #[rhai_fn(global, get = "connection_timestamp", return_raw, pure)]
    pub fn connection_timestamp(
        this: &mut std::sync::Arc<std::sync::RwLock<MailContext>>,
    ) -> EngineResult<std::time::SystemTime> {
        Ok(this
            .read()
            .map_err::<Box<EvalAltResult>, _>(|e| e.to_string().into())?
            .connection
            .timestamp)
    }

    #[rhai_fn(global, get = "server_name", return_raw, pure)]
    pub fn server_name(
        this: &mut std::sync::Arc<std::sync::RwLock<MailContext>>,
    ) -> EngineResult<String> {
        Ok(this
            .read()
            .map_err::<Box<EvalAltResult>, _>(|e| e.to_string().into())?
            .connection
            .server_name
            .clone())
    }

    #[rhai_fn(global, get = "is_secured", return_raw, pure)]
    pub fn is_secured(
        this: &mut std::sync::Arc<std::sync::RwLock<MailContext>>,
    ) -> EngineResult<bool> {
        Ok(this
            .read()
            .map_err::<Box<EvalAltResult>, _>(|e| e.to_string().into())?
            .connection
            .is_secured)
    }

    #[rhai_fn(global, get = "is_authenticated", return_raw, pure)]
    pub fn is_authenticated(
        this: &mut std::sync::Arc<std::sync::RwLock<MailContext>>,
    ) -> EngineResult<bool> {
        Ok(this
            .read()
            .map_err::<Box<EvalAltResult>, _>(|e| e.to_string().into())?
            .connection
            .is_authenticated)
    }

    #[rhai_fn(global, get = "auth", return_raw, pure)]
    pub fn auth(
        this: &mut std::sync::Arc<std::sync::RwLock<MailContext>>,
    ) -> EngineResult<vsmtp_common::mail_context::AuthCredentials> {
        Ok(this
            .read()
            .map_err::<Box<EvalAltResult>, _>(|e| e.to_string().into())?
            .connection
            .credentials
            .clone()
            .ok_or("is none")?)
    }

    #[rhai_fn(global, get = "type", pure)]
    pub fn get_type(my_enum: &mut vsmtp_common::mail_context::AuthCredentials) -> String {
        match my_enum {
            vsmtp_common::mail_context::AuthCredentials::Verify { .. } => "Verify".to_string(),
            vsmtp_common::mail_context::AuthCredentials::Query { .. } => "Query".to_string(),
        }
    }

    #[rhai_fn(global, get = "authid", pure)]
    pub fn get_authid(my_enum: &mut vsmtp_common::mail_context::AuthCredentials) -> String {
        match my_enum {
            vsmtp_common::mail_context::AuthCredentials::Query { authid }
            | vsmtp_common::mail_context::AuthCredentials::Verify { authid, .. } => authid.clone(),
        }
    }

    #[rhai_fn(global, get = "authpass", return_raw, pure)]
    pub fn get_authpass(
        my_enum: &mut vsmtp_common::mail_context::AuthCredentials,
    ) -> EngineResult<String> {
        match my_enum {
            vsmtp_common::mail_context::AuthCredentials::Verify { authpass, .. } => {
                Ok(authpass.clone())
            }
            vsmtp_common::mail_context::AuthCredentials::Query { .. } => {
                Err("no `authpass` available in credentials of type `Query`"
                    .to_string()
                    .into())
            }
        }
    }

    #[rhai_fn(global, get = "helo", return_raw, pure)]
    pub fn helo(this: &mut std::sync::Arc<std::sync::RwLock<MailContext>>) -> EngineResult<String> {
        Ok(this
            .read()
            .map_err::<Box<EvalAltResult>, _>(|e| e.to_string().into())?
            .envelop
            .helo
            .clone())
    }

    #[rhai_fn(global, get = "mail_from", return_raw, pure)]
    pub fn mail_from(
        this: &mut std::sync::Arc<std::sync::RwLock<MailContext>>,
    ) -> EngineResult<Address> {
        Ok(this
            .read()
            .map_err::<Box<EvalAltResult>, _>(|e| e.to_string().into())?
            .envelop
            .mail_from
            .clone())
    }

    #[rhai_fn(global, get = "rcpt", return_raw, pure)]
    pub fn rcpt(
        this: &mut std::sync::Arc<std::sync::RwLock<MailContext>>,
    ) -> EngineResult<Vec<Address>> {
        Ok(this
            .read()
            .map_err::<Box<EvalAltResult>, _>(|e| e.to_string().into())?
            .envelop
            .rcpt
            .iter()
            .map(|rcpt| rcpt.address.clone())
            .collect())
    }

    #[rhai_fn(global, get = "mail_timestamp", return_raw, pure)]
    pub fn mail_timestamp(
        this: &mut std::sync::Arc<std::sync::RwLock<MailContext>>,
    ) -> EngineResult<std::time::SystemTime> {
        Ok(this
            .read()
            .map_err::<Box<EvalAltResult>, _>(|e| e.to_string().into())?
            .metadata
            .as_ref()
            .ok_or_else::<Box<EvalAltResult>, _>(|| {
                "metadata are not available in this stage".into()
            })?
            .timestamp)
    }

    #[rhai_fn(global, get = "message_id", return_raw, pure)]
    pub fn message_id(
        this: &mut std::sync::Arc<std::sync::RwLock<MailContext>>,
    ) -> EngineResult<String> {
        Ok(this
            .read()
            .map_err::<Box<EvalAltResult>, _>(|e| e.to_string().into())?
            .metadata
            .as_ref()
            .ok_or_else::<Box<EvalAltResult>, _>(|| {
                "metadata are not available in this stage".into()
            })?
            .message_id
            .clone())
    }

    #[rhai_fn(global, get = "mail", return_raw, pure)]
    pub fn mail(this: &mut std::sync::Arc<std::sync::RwLock<MailContext>>) -> EngineResult<String> {
        Ok(
            match &this
                .read()
                .map_err::<Box<EvalAltResult>, _>(|e| e.to_string().into())?
                .body
            {
                vsmtp_common::mail_context::Body::Empty => String::default(),
                vsmtp_common::mail_context::Body::Raw(raw) => raw.clone(),
                vsmtp_common::mail_context::Body::Parsed(parsed) => parsed.to_raw(),
            },
        )
    }

    #[rhai_fn(global, name = "to_string", pure)]
    pub fn ctx_to_string(_: &mut std::sync::Arc<std::sync::RwLock<MailContext>>) -> String {
        "MailContext".to_string()
    }

    #[rhai_fn(global, name = "to_debug", pure)]
    pub fn ctx_to_debug(this: &mut std::sync::Arc<std::sync::RwLock<MailContext>>) -> String {
        ctx_to_string(this)
    }

    #[rhai_fn(global, name = "to_string", pure)]
    pub fn srv_to_string(_: &mut std::sync::Arc<ServerAPI>) -> String {
        "Server".to_string()
    }

    #[rhai_fn(global, name = "to_debug", pure)]
    pub fn srv_to_debug(this: &mut std::sync::Arc<ServerAPI>) -> String {
        srv_to_string(this)
    }
}

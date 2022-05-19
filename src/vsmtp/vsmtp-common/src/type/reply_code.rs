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

/// Codes as the start of each lines of a reply
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplyCode {
    /// smtp codes as defined in https://datatracker.ietf.org/doc/html/rfc5321#section-4.2
    Code(u16),
    /// enhanced codes
    Enhanced(u16, String),
}

impl ReplyCode {
    ///
    #[must_use]
    pub const fn is_error(&self) -> bool {
        match self {
            ReplyCode::Code(code) | ReplyCode::Enhanced(code, _) => code.rem_euclid(100) >= 4,
        }
    }
}

///
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize)]
pub enum CodesID {
    //
    // Specials Messages
    //
    /// First message sent by the server
    Greetings,
    ///
    Help,
    ///
    Closing,
    ///
    EhloPain,
    ///
    EhloSecured,
    ///
    DataStart,
    //
    // SessionStatus
    //
    /// Accepted
    Ok,
    ///
    Denied,
    //
    // Parsing Command
    //
    ///
    UnrecognizedCommand,
    ///
    SyntaxErrorParams,
    ///
    ParameterUnimplemented,
    ///
    Unimplemented,
    ///
    BadSequence,
    //
    // TLS extension
    //
    ///
    TLSNotAvailable,
    ///
    AlreadyUnderTLS,
    /// The policy of the server require the client to be in a secured connection for a mail transaction,
    /// must use `STARTTLS`
    TLSRequired,
    //
    // Auth extension
    //
    ///
    AuthSucceeded,
    ///
    AuthMechNotSupported,
    ///
    AuthClientMustNotStart,
    ///
    AuthMechanismMustBeEncrypted,
    ///
    AuthInvalidCredentials,
    /// The policy of the server require the client to be authenticated for a mail transaction
    AuthRequired,
    ///
    AuthClientCanceled,
    ///
    AuthErrorDecode64,
    //
    // Security mechanism
    //
    /// The number of connection maximum accepted as the same time as been reached
    ConnectionMaxReached,
    /// The threshold `error_count` has been passed, then server will shutdown the connection
    TooManyError,
    ///
    Timeout,
    ///
    TooManyRecipients,
}

// ///
// pub const UNRECOGNIZED_COMMAND: ReplyCode = ReplyCode::Code(500);
// ///
// pub const SYNTAX_ERROR_PARAMS: ReplyCode = ReplyCode::Code(501);
// ///
// pub const UNIMPLEMENTED: ReplyCode = ReplyCode::Code(504);
//
// ///
// pub static AUTH_MECH_NOT_SUPPORTED: ReplyCode = ReplyCode::Enhanced(504, "5.5.4".to_string());
//
// ///
// pub static SUPPORTED_CODES: &[&ReplyCode; 4] = &[
//     &UNRECOGNIZED_COMMAND,
//     &SYNTAX_ERROR_PARAMS,
//     &UNIMPLEMENTED,
//     &AUTH_MECH_NOT_SUPPORTED,
// ];

use crate::Reply;

impl<'de> serde::Deserialize<'de> for ReplyCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}

impl serde::Serialize for ReplyCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

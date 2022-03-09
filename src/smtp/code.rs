/**
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
**/

// TODO: EnhancedStatusCodes,
// see https://datatracker.ietf.org/doc/html/rfc2034

/// 2yz  Positive Completion reply
/// 3yz  Positive Intermediate reply
/// 4yz  Transient Negative Completion reply
/// 5yz  Permanent Negative Completion reply

/// x0z  Syntax: These replies refer to syntax errors, syntactically
/// correct commands that do not fit any functional category, and
/// unimplemented or superfluous commands.
///
/// x1z  Information: These are replies to requests for information, such
/// as status or help.
///
/// x2z  Connections: These are replies referring to the transmission
/// channel.
///
/// x3z  Unspecified.
/// x4z  Unspecified.
///
/// x5z  Mail system: These replies indicate the status of the receiver
/// mail system vis-a-vis the requested transfer or other mail system
/// action.
#[allow(clippy::module_name_repetitions)]
#[derive(
    Debug,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Copy,
    Clone,
    enum_iterator::IntoEnumIterator,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(untagged)]
#[serde(into = "String")]
#[serde(try_from = "String")]
pub enum SMTPReplyCode {
    /// system status, or system help reply
    // Code211,
    /// help message
    Code214,
    /// service ready
    Code220,
    /// service closing transmission channel
    Code221,
    /// requested mail action okay, completed
    Code250,
    /// ehlo message
    Code250PlainEsmtp,
    /// esmtp ehlo message
    Code250SecuredEsmtp,
    /// user not local; will forward
    // Code251,
    /// cannot verify the user, but it will try to deliver the message anyway
    // Code252,
    ///
    /// start mail input
    Code354,
    ///
    /// service not available, closing transmission channel
    // Code421,
    /// requested mail action not taken: mailbox unavailable
    // Code450,
    /// requested action aborted: local error in processing
    Code451,
    Code451Timeout,
    Code451TooManyError,
    /// requested action not taken: insufficient system storage
    Code452,
    Code452TooManyRecipients,
    // TLS not available due to temporary reason
    Code454,
    /// server unable to accommodate parameters
    // Code455,
    ///
    /// syntax error, command unrecognized
    Code500,
    /// syntax error in parameters or arguments
    Code501,
    /// command not implemented
    Code502unimplemented,
    /// bad sequence of commands
    Code503,
    /// command parameter is not implemented
    Code504,
    /// server does not accept mail
    // Code521,
    /// encryption Needed
    // Code523,

    /// 530 Must issue a STARTTLS command first
    Code530,
    /// requested action not taken: mailbox unavailable
    // Code550,
    /// user not local; please try <forward-path>
    // Code551,
    /// requested mail action aborted: exceeded storage allocation
    // Code552,
    /// requested action not taken: mailbox name not allowed
    // Code553,
    /// connection has been denied.
    Code554,
    /// transaction has failed
    Code554tls,
    // Code555,
    // domain does not accept mail
    // Code556,
    /// 554
    ConnectionMaxReached,
}

impl SMTPReplyCode {
    pub(crate) fn is_error(self) -> bool {
        match self {
            SMTPReplyCode::Code214
            | SMTPReplyCode::Code220
            | SMTPReplyCode::Code221
            | SMTPReplyCode::Code250
            | SMTPReplyCode::Code250PlainEsmtp
            | SMTPReplyCode::Code250SecuredEsmtp
            | SMTPReplyCode::Code354 => false,
            //
            SMTPReplyCode::Code451Timeout
            | SMTPReplyCode::Code451
            | SMTPReplyCode::Code452
            | SMTPReplyCode::Code452TooManyRecipients
            | SMTPReplyCode::Code454
            | SMTPReplyCode::Code500
            | SMTPReplyCode::Code501
            | SMTPReplyCode::Code502unimplemented
            | SMTPReplyCode::Code503
            | SMTPReplyCode::Code530
            | SMTPReplyCode::Code554
            | SMTPReplyCode::Code554tls
            | SMTPReplyCode::ConnectionMaxReached => true,
            //
            _ => unimplemented!(),
        }
    }
}

impl std::fmt::Display for SMTPReplyCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SMTPReplyCode::Code214 => "Code214",
            SMTPReplyCode::Code220 => "Code220",
            SMTPReplyCode::Code221 => "Code221",
            SMTPReplyCode::Code250 => "Code250",
            SMTPReplyCode::Code250PlainEsmtp => "Code250PlainEsmtp",
            SMTPReplyCode::Code250SecuredEsmtp => "Code250SecuredEsmtp",
            SMTPReplyCode::Code354 => "Code354",
            SMTPReplyCode::Code451 => "Code451",
            SMTPReplyCode::Code451Timeout => "Code451Timeout",
            SMTPReplyCode::Code451TooManyError => "Code451TooManyError",
            SMTPReplyCode::Code452 => "Code452",
            SMTPReplyCode::Code452TooManyRecipients => "Code452TooManyRecipients",
            SMTPReplyCode::Code454 => "Code454",
            SMTPReplyCode::Code500 => "Code500",
            SMTPReplyCode::Code501 => "Code501",
            SMTPReplyCode::Code502unimplemented => "Code502unimplemented",
            SMTPReplyCode::Code503 => "Code503",
            SMTPReplyCode::Code504 => "Code504",
            SMTPReplyCode::Code530 => "Code530",
            SMTPReplyCode::Code554 => "Code554",
            SMTPReplyCode::Code554tls => "Code554tls",
            SMTPReplyCode::ConnectionMaxReached => "ConnectionMaxReached",
        })
    }
}

impl From<SMTPReplyCode> for String {
    fn from(code: SMTPReplyCode) -> Self {
        format!("{}", code)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SMTPReplyCodeFromStrError;

impl std::fmt::Display for SMTPReplyCodeFromStrError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("SMTPReplyCodeFromStrError")
    }
}

impl std::str::FromStr for SMTPReplyCode {
    type Err = SMTPReplyCodeFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Code214" => Ok(Self::Code214),
            "Code220" => Ok(Self::Code220),
            "Code221" => Ok(Self::Code221),
            "Code250" => Ok(Self::Code250),
            "Code250PlainEsmtp" => Ok(Self::Code250PlainEsmtp),
            "Code250SecuredEsmtp" => Ok(Self::Code250SecuredEsmtp),
            "Code354" => Ok(Self::Code354),
            "Code451" => Ok(Self::Code451),
            "Code451Timeout" => Ok(Self::Code451Timeout),
            "Code451TooManyError" => Ok(Self::Code451TooManyError),
            "Code452" => Ok(Self::Code452),
            "Code452TooManyRecipients" => Ok(Self::Code452TooManyRecipients),
            "Code454" => Ok(Self::Code454),
            "Code500" => Ok(Self::Code500),
            "Code501" => Ok(Self::Code501),
            "Code502unimplemented" => Ok(Self::Code502unimplemented),
            "Code503" => Ok(Self::Code503),
            "Code504" => Ok(Self::Code504),
            "Code530" => Ok(Self::Code530),
            "Code554" => Ok(Self::Code554),
            "Code554tls" => Ok(Self::Code554tls),
            "ConnectionMaxReached" => Ok(Self::ConnectionMaxReached),
            _ => Err(SMTPReplyCodeFromStrError),
        }
    }
}

impl TryFrom<String> for SMTPReplyCode {
    type Error = SMTPReplyCodeFromStrError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        <Self as std::str::FromStr>::from_str(&value)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::SMTPReplyCode;

    #[test]
    fn error() {
        assert_eq!(
            format!("{}", SMTPReplyCode::from_str("root").unwrap_err()),
            "SMTPReplyCodeFromStrError"
        );
    }

    #[test]
    fn same() {
        for s in <SMTPReplyCode as enum_iterator::IntoEnumIterator>::into_enum_iter() {
            assert_eq!(SMTPReplyCode::from_str(&format!("{}", s)).unwrap(), s);
        }
    }
}
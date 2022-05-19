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

use crate::ReplyCode;

/// SMTP message send by the server to the client as defined in RFC5321#4.2
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Reply {
    ///
    pub code: ReplyCode,
    ///
    pub text_string: String,
}

impl Reply {
    ///
    #[must_use]
    pub fn fold(&self) -> String {
        // let size_to_remove = "xyz ".len() + enhanced.map_or(0, |_| "X.Y.Z ".len()) + "\r\n".len();
        // let size_to_remove = 2 + &match self.code {
        //     ReplyCode::Code(_) => 0,
        //     ReplyCode::Enhanced(_, enhanced) => enhanced.len(),
        // }; // ("xyz" + " " + [ enhanced + " " ] + "\r\n").len()

        let mut prefix = vec![];
        prefix.extend_from_slice(
            &match &self.code {
                ReplyCode::Code(code) => format!("{code} "),
                ReplyCode::Enhanced(code, enhanced) => format!("{code} {enhanced} "),
            }
            .chars()
            .collect::<Vec<_>>(),
        );

        let output = self
            .text_string
            .split("\r\n")
            .filter(|s| !s.is_empty())
            .flat_map(|line| {
                line.chars()
                    .collect::<Vec<char>>()
                    .chunks(80 - (prefix.len() + 2))
                    .flat_map(|c| [&prefix, c, &"\r\n".chars().collect::<Vec<_>>()].concat())
                    .collect::<String>()
                    .chars()
                    .collect::<Vec<_>>()
            })
            .collect::<String>();

        let mut output = output
            .split("\r\n")
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();

        let len = output.len();
        for i in &mut output[0..len - 1] {
            i.replace_range(3..4, "-");
        }

        output
            .into_iter()
            .flat_map(|mut l| {
                l.push_str("\r\n");
                l.chars().collect::<Vec<_>>()
            })
            .collect::<String>()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Reply, ReplyCode};

    #[test]
    fn no_fold() {
        let output = Reply {
            code: ReplyCode::Code(220),
            text_string: "this is a custom code.".to_string(),
        }
        .fold();
        pretty_assertions::assert_eq!(output, "220 this is a custom code.\r\n".to_string());
        for i in output.split("\r\n") {
            assert!(i.len() <= 78);
        }
    }

    #[test]
    fn one_line() {
        let output = Reply {
            code: ReplyCode::Enhanced(220, "2.0.0".to_string()),
            text_string: [
                "this is a long message, a very very long message ...",
                " carriage return will be properly added automatically.",
            ]
            .concat(),
        }
        .fold();
        pretty_assertions::assert_eq!(
            output,
            [
                "220-2.0.0 this is a long message, a very very long message ... carriage return\r\n",
                "220 2.0.0  will be properly added automatically.\r\n",
            ]
            .concat()
        );
        for i in output.split("\r\n") {
            assert!(i.len() <= 78);
        }
    }

    #[test]
    fn two_line() {
        let output = Reply {
            code: ReplyCode::Enhanced(220, "2.0.0".to_string()),
            text_string: [
                "this is a long message, a very very long message ...",
                " carriage return will be properly added automatically. Made by",
                " vSMTP mail transfer agent\nCopyright (C) 2022 viridIT SAS",
            ]
            .concat(),
        }
        .fold();
        pretty_assertions::assert_eq!(
            output,
            [
                "220-2.0.0 this is a long message, a very very long message ... carriage return\r\n",
                "220-2.0.0  will be properly added automatically. Made by vSMTP mail transfer a\r\n",
                "220 2.0.0 gent\nCopyright (C) 2022 viridIT SAS\r\n",
            ]
            .concat()
        );
        for i in output.split("\r\n") {
            assert!(i.len() <= 78);
        }
    }

    #[test]
    fn ehlo_response() {
        let output = Reply {
            code: ReplyCode::Code(250),
            text_string: [
                "testserver.com\r\n",
                "AUTH PLAIN LOGIN CRAM-MD5\r\n",
                "8BITMIME\r\n",
                "SMTPUTF8\r\n",
            ]
            .concat(),
        }
        .fold();
        pretty_assertions::assert_eq!(
            output,
            [
                "250-testserver.com\r\n",
                "250-AUTH PLAIN LOGIN CRAM-MD5\r\n",
                "250-8BITMIME\r\n",
                "250 SMTPUTF8\r\n",
            ]
            .concat()
        );
        for i in output.split("\r\n") {
            assert!(i.len() <= 78);
        }
    }
}

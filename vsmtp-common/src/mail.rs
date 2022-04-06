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
use super::mime_type::Mime;

/// we use Vec instead of a HashMap because header ordering is important.
#[allow(clippy::module_name_repetitions)]
pub type MailHeaders = Vec<(String, String)>;

/// see rfc5322 (section 2.1 and 2.3)
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum BodyType {
    /// Text message body
    Regular(Vec<String>),
    /// Mime
    Mime(Box<Mime>),
    /// Empty message body
    Undefined,
}

impl ToString for BodyType {
    fn to_string(&self) -> String {
        match self {
            Self::Regular(content) => content
                .iter()
                .map(|l| {
                    if l.starts_with('.') {
                        ".".to_owned() + l
                    } else {
                        l.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
            Self::Mime(content) => content.to_raw(),
            Self::Undefined => String::default(),
        }
    }
}

/// Message body representation
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Mail {
    /// Message body 's headers
    pub headers: MailHeaders,
    /// Message body content
    pub body: BodyType,
}

impl Default for Mail {
    fn default() -> Self {
        Self {
            headers: vec![],
            body: BodyType::Undefined,
        }
    }
}

impl Mail {
    /// get the original header section of the email.
    #[must_use]
    pub fn raw_headers(&self) -> String {
        self.headers
            .iter()
            .map(|(header, value)| format!("{}: {}", header, value))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// get the original body section of the email.
    #[must_use]
    pub fn raw_body(&self) -> String {
        self.body.to_string()
    }

    /// return the original text representation of the email.
    #[must_use]
    pub fn to_raw(&self) -> String {
        if let BodyType::Mime(_) = &self.body {
            // in case of a mime type, mime headers are merged into the rfc822 mail header section.
            format!("{}\n{}", self.raw_headers(), self.raw_body())
        } else {
            format!("{}\n\n{}", self.raw_headers(), self.raw_body())
        }
    }

    /// change the from field of the header
    pub fn rewrite_mail_from(&mut self, value: &str) {
        if let Some((_, old)) = self.headers.iter_mut().find(|(header, _)| header == "from") {
            *old = value.to_string();
        } else {
            self.headers.push(("from".to_string(), value.to_string()));
        }
    }

    /// change one recipients value from @old to @new
    pub fn rewrite_rcpt(&mut self, old: &str, new: &str) {
        if let Some((_, rcpts)) = self.headers.iter_mut().find(|(header, _)| header == "to") {
            *rcpts = rcpts.replace(old, new);
        } else {
            self.headers.push(("to".to_string(), new.to_string()));
        }
    }

    /// add a recipients
    pub fn add_rcpt(&mut self, new: &str) {
        if let Some((_, rcpts)) = self.headers.iter_mut().find(|(header, _)| header == "to") {
            *rcpts = format!("{rcpts}, {new}");
        } else {
            self.headers.push(("to".to_string(), new.to_string()));
        }
    }

    /// remove a recipients
    pub fn remove_rcpt(&mut self, old: &str) {
        self.headers
            .iter_mut()
            .find(|(header, _)| header == "to")
            .and_then::<(), _>(|(_, rcpts)| {
                if rcpts.find(old) == Some(0) {
                    *rcpts = rcpts.replace(format!("{old}, ").as_str(), "");
                } else {
                    *rcpts = rcpts.replace(format!(", {old}").as_str(), "");
                }
                None
            });
    }

    /// prepend new headers to the email, folding if necessary.
    pub fn prepend_headers(&mut self, headers: Vec<(String, String)>) {
        self.headers.splice(..0, headers);
    }

    /// push new headers to the email, folding if necessary.
    pub fn push_headers(&mut self, headers: Vec<(String, String)>) {
        self.headers.extend(headers);
    }
}

#[cfg(test)]
mod test {

    use super::{BodyType, Mail};
    use crate::{
        mail,
        mime_type::{Mime, MimeBodyType, MimeHeader},
    };

    #[test]
    fn test_construct_mail() {
        let empty_mail = Mail {
            headers: vec![("from".to_string(), "a@a".to_string())],
            body: BodyType::Undefined,
        };

        // on newline added to separate the body, one for the empty body.
        // anyway, this example should not happen in a real scenario.
        assert_eq!(
            empty_mail.to_raw(),
            r#"from: a@a

"#
            .to_string()
        );

        let regular_mail = Mail {
            headers: vec![("from".to_string(), "a@a".to_string())],
            body: BodyType::Regular(vec!["This is a regular body.".to_string()]),
        };

        assert_eq!(
            regular_mail.to_raw(),
            r#"from: a@a

This is a regular body."#
                .to_string()
        );

        let mime_mail = Mail {
            headers: vec![
                ("from".to_string(), "a@a".to_string()),
                ("mime-version".to_string(), "1.0".to_string()),
            ],
            body: BodyType::Mime(Box::new(Mime {
                headers: vec![MimeHeader {
                    name: "content-type".to_string(),
                    value: "text/plain".to_string(),
                    args: std::collections::HashMap::new(),
                }],
                content: MimeBodyType::Regular(vec!["this is a regular mime body.".to_string()]),
            })),
        };

        // mime headers should be merged with the rfc822 message header section.
        assert_eq!(
            mime_mail.to_raw(),
            r#"from: a@a
mime-version: 1.0
content-type: text/plain

this is a regular mime body."#
                .to_string()
        );
    }

    #[test]
    fn test_add_headers() {
        let mut mail = Mail {
            body: BodyType::Regular(vec!["email content".to_string()]),
            ..mail::Mail::default()
        };

        mail.push_headers(vec![
            ("subject".to_string(), "testing an email".to_string()),
            ("mime-version".to_string(), "1.0".to_string()),
        ]);

        assert_eq!(
            mail.to_raw(),
            r#"subject: testing an email
mime-version: 1.0

email content"#
                .to_string()
        );

        mail.prepend_headers(vec![
            ("from".to_string(), "b@b".to_string()),
            (
                "date".to_string(),
                "tue, 30 nov 2021 20:54:27 +0100".to_string(),
            ),
            ("to".to_string(), "john@doe.com, green@foo.bar".to_string()),
        ]);

        assert_eq!(
            mail.to_raw(),
            r#"from: b@b
date: tue, 30 nov 2021 20:54:27 +0100
to: john@doe.com, green@foo.bar
subject: testing an email
mime-version: 1.0

email content"#
                .to_string()
        );
    }

    #[test]
    fn test_rcpt_mutation() {
        let mut mail = Mail::default();

        // rewrite when the header does not exists inserts the header.
        mail.rewrite_mail_from("a@a");
        assert_eq!(mail.headers, vec![("from".to_string(), "a@a".to_string())]);

        mail.rewrite_mail_from("b@b");
        assert_eq!(mail.headers, vec![("from".to_string(), "b@b".to_string())]);

        mail.rewrite_rcpt("b@b", "a@a");
        assert_eq!(
            mail.headers,
            vec![
                ("from".to_string(), "b@b".to_string()),
                ("to".to_string(), "a@a".to_string())
            ]
        );

        mail.add_rcpt("green@foo.bar");
        assert_eq!(
            mail.headers,
            vec![
                ("from".to_string(), "b@b".to_string()),
                ("to".to_string(), "a@a, green@foo.bar".to_string())
            ]
        );

        mail.rewrite_rcpt("a@a", "john@doe");
        assert_eq!(
            mail.headers,
            vec![
                ("from".to_string(), "b@b".to_string()),
                ("to".to_string(), "john@doe, green@foo.bar".to_string())
            ]
        );

        mail.remove_rcpt("john@doe");
        assert_eq!(
            mail.headers,
            vec![
                ("from".to_string(), "b@b".to_string()),
                ("to".to_string(), "green@foo.bar".to_string())
            ]
        );
    }
}

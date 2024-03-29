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
use super::mail::Mail;

/// header of a mime section
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct MimeHeader {
    ///
    pub name: String,
    ///
    pub value: String,
    /// parameter ordering does not matter.
    pub args: std::collections::HashMap<String, String>,
}

///
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum MimeBodyType {
    ///
    Regular(Vec<String>),
    ///
    Multipart(MimeMultipart),
    ///
    Embedded(Mail),
}

///
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct MimeMultipart {
    ///
    pub preamble: String,
    ///
    pub parts: Vec<Mime>,
    ///
    pub epilogue: String,
}

impl std::fmt::Display for MimeHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let args = self
            .args
            .iter()
            .map(|(name, value)| format!("{}=\"{}\"", name, value))
            .collect::<Vec<_>>()
            .join("; ");

        if args.is_empty() {
            f.write_fmt(format_args!("{}: {}", self.name, self.value))
        } else {
            f.write_fmt(format_args!("{}: {}; {}", self.name, self.value, args))
        }
    }
}

impl MimeMultipart {
    ///  preamble
    ///  --boundary
    ///  *{ headers \n body \n boundary}
    ///  epilogue || nothing
    ///  --end-boundary--
    fn to_raw(&self, boundary: &str) -> String {
        if self.epilogue.is_empty() {
            format!(
                "\n{}\n--{}\n{}\n--{}--\n",
                self.preamble,
                boundary,
                self.parts
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(&format!("\n--{}\n", boundary)),
                boundary,
            )
        } else {
            format!(
                "\n{}\n--{}\n{}\n{}\n--{}--\n",
                self.preamble,
                boundary,
                self.parts
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(&format!("\n--{}\n", boundary)),
                self.epilogue,
                boundary,
            )
        }
    }
}

///
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Mime {
    ///
    pub headers: Vec<MimeHeader>,
    ///
    pub content: MimeBodyType,
}

impl std::fmt::Display for Mime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}\n\n{}",
            self.raw_headers(),
            self.raw_body()
        ))
    }
}

impl Mime {
    /// get the original mime header section of the part.
    #[must_use]
    pub fn raw_headers(&self) -> String {
        self.headers
            .iter()
            .map(MimeHeader::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// get the original body section of the part.
    ///
    /// # Panics
    ///
    /// * @self is a multipart ill formed (no boundary in header)
    #[must_use]
    pub fn raw_body(&self) -> String {
        match &self.content {
            MimeBodyType::Regular(regular) => regular.join("\n"),
            MimeBodyType::Multipart(multipart) => {
                let boundary = self
                    .headers
                    .iter()
                    .find(|header| header.args.get("boundary").is_some());
                multipart.to_raw(
                    boundary
                        .expect("multipart mime message is missing it's boundary argument.")
                        .args
                        .get("boundary")
                        .unwrap(),
                )
            }
            MimeBodyType::Embedded(mail) => mail.to_raw(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mime_type() {
        let input = MimeHeader {
            name: "Content-Type".to_string(),
            value: "text/plain".to_string(),
            args: std::collections::HashMap::from([
                ("charset".to_string(), "us-ascii".to_string()),
                ("another".to_string(), "argument".to_string()),
            ]),
        };

        let order1 = input.to_string()
            == "Content-Type: text/plain; charset=\"us-ascii\"; another=\"argument\"";
        let order2 = input.to_string()
            == "Content-Type: text/plain; another=\"argument\"; charset=\"us-ascii\"";

        // arguments can be in any order.
        assert!(order1 || order2);

        let input = MimeHeader {
            name: "Content-Type".to_string(),
            value: "application/foobar".to_string(),
            args: std::collections::HashMap::default(),
        };

        assert_eq!(
            input.to_string(),
            "Content-Type: application/foobar".to_string()
        );
    }
}

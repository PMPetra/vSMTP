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

pub mod parsing;

use vsmtp_common::{
    re::{addr, anyhow, log},
    status::InfoPacket,
    Address,
};

/// Objects are rust's representation of rule engine variables.
/// multiple types are supported.
#[derive(Debug, Clone)]
pub enum Object {
    /// ip v4 address. (a.b.c.d)
    Ip4(std::net::Ipv4Addr),
    /// ip v6 address. (x:x:x:x:x:x:x:x)
    Ip6(std::net::Ipv6Addr),
    /// an ip v4 range. (a.b.c.d/range)
    Rg4(iprange::IpRange<ipnet::Ipv4Net>),
    /// an ip v6 range. (x:x:x:x:x:x:x:x/range)
    Rg6(iprange::IpRange<ipnet::Ipv6Net>),
    /// an email address (jones@foo.com)
    Address(Address),
    /// a valid fully qualified domain name (foo.com)
    Fqdn(String),
    /// a regex (^[a-z0-9.]+@foo.com$)
    Regex(regex::Regex),
    /// the content of a file.
    File(Vec<Object>),
    /// a group of objects declared inline.
    Group(Vec<std::sync::Arc<Object>>),
    /// a user.
    Identifier(String),
    /// a simple string.
    Str(String),
    /// a custom smtp reply code.
    Code(InfoPacket),
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ip4(l0), Self::Ip4(r0)) => l0 == r0,
            (Self::Ip6(l0), Self::Ip6(r0)) => l0 == r0,
            (Self::Rg4(l0), Self::Rg4(r0)) => l0 == r0,
            (Self::Rg6(l0), Self::Rg6(r0)) => l0 == r0,
            (Self::Address(l0), Self::Address(r0)) => l0 == r0,
            (Self::Fqdn(l0), Self::Fqdn(r0)) => l0 == r0,
            (Self::File(l0), Self::File(r0)) => l0 == r0,
            (Self::Group(l0), Self::Group(r0)) => l0 == r0,
            (Self::Identifier(l0), Self::Identifier(r0)) | (Self::Str(l0), Self::Str(r0)) => {
                l0 == r0
            }
            _ => false,
        }
    }
}

impl Object {
    /// get a specific value from a rhai map and convert it to a specific type.
    /// returns an error if the cast failed.
    pub(crate) fn value<S, T>(
        map: &std::collections::BTreeMap<S, rhai::Dynamic>,
        key: &str,
    ) -> anyhow::Result<T>
    where
        S: std::str::FromStr + std::cmp::Ord,
        T: Clone + 'static,
    {
        match map.get(
            &S::from_str(key)
                .map_err(|_| anyhow::anyhow!("failed to get {key} key from an object"))?,
        ) {
            Some(value) => value.clone().try_cast::<T>().ok_or_else(|| {
                anyhow::anyhow!("{} is not of type {}.", key, std::any::type_name::<T>())
            }),
            None => anyhow::bail!("'{}' key not found in object.", key),
        }
    }

    /// create an object from a raw rhai Map data structure.
    /// this map must have the "value" and "type" keys to be parsed
    /// successfully.
    #[allow(clippy::too_many_lines)]
    pub(crate) fn from<S>(
        map: &std::collections::BTreeMap<S, rhai::Dynamic>,
    ) -> anyhow::Result<Self>
    where
        S: std::fmt::Debug + std::str::FromStr + std::cmp::Ord + 'static,
    {
        let t = Self::value::<S, String>(map, "type")?;

        match t.as_str() {
            "ip4" => Ok(Self::Ip4(
                <std::net::Ipv4Addr as std::str::FromStr>::from_str(&Self::value::<S, String>(
                    map, "value",
                )?)?,
            )),

            "ip6" => Ok(Self::Ip6(
                <std::net::Ipv6Addr as std::str::FromStr>::from_str(&Self::value::<S, String>(
                    map, "value",
                )?)?,
            )),

            "rg4" => Ok(Self::Rg4(
                [Self::value::<S, String>(map, "value")?.parse::<ipnet::Ipv4Net>()?]
                    .into_iter()
                    .collect(),
            )),

            "rg6" => Ok(Self::Rg6(
                [Self::value::<S, String>(map, "value")?.parse::<ipnet::Ipv6Net>()?]
                    .into_iter()
                    .collect(),
            )),

            "fqdn" => {
                let value = Self::value::<S, String>(map, "value")?;
                match addr::parse_domain_name(&value) {
                    Ok(domain) => Ok(Self::Fqdn(domain.to_string())),
                    Err(_) => anyhow::bail!("'{}' is not a valid fqdn.", value),
                }
            }

            "address" => {
                let value = Self::value::<S, String>(map, "value")?;
                Ok(Self::Address(Address::try_from(value)?))
            }

            "ident" => Ok(Self::Identifier(Self::value::<S, String>(map, "value")?)),

            "string" => Ok(Self::Str(Self::value::<S, String>(map, "value")?)),

            "regex" => Ok(Self::Regex(<regex::Regex as std::str::FromStr>::from_str(
                &Self::value::<S, String>(map, "value")?,
            )?)),

            // the file object as an extra "content_type" parameter.
            "file" => {
                let value = Self::value::<S, String>(map, "value")?;
                let content_type = Self::value::<S, String>(map, "content_type")?;
                let reader = std::io::BufReader::new(std::fs::File::open(&value)?);
                let mut content = Vec::with_capacity(20);

                for line in std::io::BufRead::lines(reader) {
                    match line {
                        Ok(line) => match content_type.as_str() {
                            "ip4" => content.push(Self::Ip4(
                                <std::net::Ipv4Addr as std::str::FromStr>::from_str(&line)?,
                            )),
                            "ip6" => content.push(Self::Ip6(
                                <std::net::Ipv6Addr as std::str::FromStr>::from_str(&line)?,
                            )),
                            "fqdn" => match addr::parse_domain_name(&line) {
                                Ok(domain) => content.push(Self::Fqdn(domain.to_string())),
                                Err(_) => anyhow::bail!("'{}' is not a valid fqdn.", value),
                            },
                            "address" => {
                                content.push(Self::Address(Address::try_from(line)?));
                            }
                            "string" => content.push(Self::Str(line)),
                            "ident" => content.push(Self::Identifier(line)),
                            "regex" => content.push(Self::Regex(
                                <regex::Regex as std::str::FromStr>::from_str(&line)?,
                            )),
                            _ => {}
                        },
                        Err(error) => log::error!("couldn't read line in '{}': {}", value, error),
                    };
                }

                Ok(Self::File(content))
            }

            "group" => {
                let mut group = vec![];
                let elements = Self::value::<S, rhai::Array>(map, "value")?;
                let name = Self::value::<S, String>(map, "name")?;

                for element in elements {
                    group.push(
                        element
                            .clone()
                            .try_cast::<std::sync::Arc<Self>>()
                            .ok_or_else(|| {
                                anyhow::anyhow!(
                                    "the element '{:?}' inside the '{}' group is not an object",
                                    element,
                                    name
                                )
                            })?,
                    );
                }

                Ok(Self::Group(group))
            }

            "code" => {
                if let Ok(code) = Self::value::<S, String>(map, "value") {
                    Ok(Self::Code(InfoPacket::Str(code)))
                } else {
                    Ok(Self::Code(InfoPacket::Code {
                        base: Self::value::<S, i64>(map, "base")?,
                        enhanced: Self::value::<S, String>(map, "enhanced")?,
                        text: Self::value::<S, String>(map, "text")?,
                    }))
                }
            }
            _ => anyhow::bail!("'{}' is an unknown object type.", t),
        }
    }

    /// get the type of the object.
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Object::Ip4(_) => "ip4",
            Object::Ip6(_) => "ip6",
            Object::Rg4(_) => "rg4",
            Object::Rg6(_) => "rg6",
            Object::Address(_) => "address",
            Object::Fqdn(_) => "fqdn",
            Object::Regex(_) => "regex",
            Object::File(_) => "file",
            Object::Group(_) => "group",
            Object::Identifier(_) => "identifier",
            Object::Str(_) => "string",
            Object::Code(_) => "code",
        }
    }
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Object::Ip4(ip) => ip.to_string(),
                Object::Ip6(ip) => ip.to_string(),
                Object::Rg4(range) => format!("{range:?}"),
                Object::Rg6(range) => format!("{range:?}"),
                Object::Address(addr) => addr.to_string(),
                Object::Fqdn(fqdn) => fqdn.clone(),
                Object::Regex(regex) => regex.to_string(),
                Object::File(file) => format!("{file:?}"),
                Object::Group(group) => format!("{group:?}"),
                Object::Identifier(string) | Object::Str(string) => string.clone(),
                Object::Code(InfoPacket::Str(message)) => message.clone(),
                Object::Code(InfoPacket::Code {
                    base,
                    enhanced,
                    text,
                }) => format!("{base} {enhanced} {text}"),
            }
        )
    }
}

#[cfg(test)]
mod test {

    use super::Object;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_from() {
        let ip4 = Object::from(&rhai::Map::from_iter([
            ("name".into(), rhai::Dynamic::from("ip4".to_string())),
            ("type".into(), rhai::Dynamic::from("ip4".to_string())),
            ("value".into(), rhai::Dynamic::from("127.0.0.1".to_string())),
        ]))
        .unwrap();

        let ip6 = Object::from(&rhai::Map::from_iter([
            ("ip6".into(), rhai::Dynamic::from("ip6".to_string())),
            ("type".into(), rhai::Dynamic::from("ip6".to_string())),
            (
                "value".into(),
                rhai::Dynamic::from("2001:0db8:0000:85a3:0000:0000:ac1f:8001".to_string()),
            ),
        ]))
        .unwrap();

        let rg4 = Object::from(&rhai::Map::from_iter([
            ("name".into(), rhai::Dynamic::from("rg4".to_string())),
            ("type".into(), rhai::Dynamic::from("rg4".to_string())),
            (
                "value".into(),
                rhai::Dynamic::from("192.168.0.0/24".to_string()),
            ),
        ]))
        .unwrap();

        let rg6 = Object::from(&rhai::Map::from_iter([
            ("name".into(), rhai::Dynamic::from("rg6".to_string())),
            ("type".into(), rhai::Dynamic::from("rg6".to_string())),
            (
                "value".into(),
                rhai::Dynamic::from("2001:db8:1234::/48".to_string()),
            ),
        ]))
        .unwrap();

        let fqdn = Object::from(&rhai::Map::from_iter([
            ("name".into(), rhai::Dynamic::from("fqdn".to_string())),
            ("type".into(), rhai::Dynamic::from("fqdn".to_string())),
            (
                "value".into(),
                rhai::Dynamic::from("example.com".to_string()),
            ),
        ]))
        .unwrap();

        let address = Object::from(&rhai::Map::from_iter([
            ("name".into(), rhai::Dynamic::from("address".to_string())),
            ("type".into(), rhai::Dynamic::from("address".to_string())),
            (
                "value".into(),
                rhai::Dynamic::from("john@doe.com".to_string()),
            ),
        ]))
        .unwrap();

        let identifier = Object::from(&rhai::Map::from_iter([
            ("name".into(), rhai::Dynamic::from("ident".to_string())),
            ("type".into(), rhai::Dynamic::from("ident".to_string())),
            ("value".into(), rhai::Dynamic::from("john".to_string())),
        ]))
        .unwrap();

        let string = Object::from(&rhai::Map::from_iter([
            ("name".into(), rhai::Dynamic::from("string".to_string())),
            ("type".into(), rhai::Dynamic::from("string".to_string())),
            (
                "value".into(),
                rhai::Dynamic::from("a text string".to_string()),
            ),
        ]))
        .unwrap();

        let regex = Object::from(&rhai::Map::from_iter([
            ("name".into(), rhai::Dynamic::from("regex".to_string())),
            ("type".into(), rhai::Dynamic::from("regex".to_string())),
            (
                "value".into(),
                rhai::Dynamic::from("^[a-z0-9.]+.com$".to_string()),
            ),
        ]))
        .unwrap();

        // TODO: test all possible content types.
        let file = Object::from(&rhai::Map::from_iter([
            ("name".into(), rhai::Dynamic::from("file".to_string())),
            ("type".into(), rhai::Dynamic::from("file".to_string())),
            (
                "content_type".into(),
                rhai::Dynamic::from("address".to_string()),
            ),
            (
                "value".into(),
                rhai::Dynamic::from("./src/tests/types/address/whitelist.txt".to_string()),
            ),
        ]))
        .unwrap();

        let group = Object::from(&rhai::Map::from_iter([
            ("name".into(), rhai::Dynamic::from("group".to_string())),
            ("type".into(), rhai::Dynamic::from("group".to_string())),
            (
                "value".into(),
                rhai::Dynamic::from(rhai::Array::from_iter([
                    rhai::Dynamic::from(std::sync::Arc::new(ip4.clone())),
                    rhai::Dynamic::from(std::sync::Arc::new(rg6.clone())),
                    rhai::Dynamic::from(std::sync::Arc::new(fqdn.clone())),
                ])),
            ),
        ]))
        .unwrap();

        let code = Object::from(&rhai::Map::from_iter([
            ("name".into(), rhai::Dynamic::from("code".to_string())),
            ("type".into(), rhai::Dynamic::from("code".to_string())),
            ("base".into(), rhai::Dynamic::from(550_i64)),
            ("enhanced".into(), rhai::Dynamic::from("5.7.2".to_string())),
            (
                "text".into(),
                rhai::Dynamic::from("nice to meet you, client".to_string()),
            ),
        ]))
        .unwrap();

        Object::from(&rhai::Map::from_iter([
            (
                "name".into(),
                rhai::Dynamic::from("inline code".to_string()),
            ),
            ("type".into(), rhai::Dynamic::from("code".to_string())),
            ("value".into(), rhai::Dynamic::from("250 ok".to_string())),
        ]))
        .unwrap();

        println!(
            "{}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}",
            ip4.as_str(),
            ip6.as_str(),
            rg4.as_str(),
            rg6.as_str(),
            fqdn.as_str(),
            address.as_str(),
            identifier.as_str(),
            string.as_str(),
            regex.as_str(),
            file.as_str(),
            group.as_str(),
            code.as_str(),
        );
    }
}

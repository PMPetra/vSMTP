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

use crate::{Config, ConfigServerDNS, ResolverOptsWrapper};
use trust_dns_resolver::{config::ResolverConfig, error::ResolveError, TokioAsyncResolver};

/// construct a [trust-dns] `ResolverOpts` struct from our wrapper.
///
/// # Notes
/// * cannot use the From trait here because `ResolverOpts` is from an external library.
/// * cannot use struct {} declaration because `ResolverOpts` is non-exhaustive.
pub fn resolver_opts_from_config(
    config: &ResolverOptsWrapper,
) -> trust_dns_resolver::config::ResolverOpts {
    let mut opts = trust_dns_resolver::config::ResolverOpts::default();

    opts.timeout = config.timeout;
    opts.attempts = config.attempts;
    opts.rotate = config.rotate;
    opts.validate = config.dnssec;
    opts.ip_strategy = config.ip_strategy;
    opts.cache_size = config.cache_size;
    opts.use_hosts_file = config.use_hosts_file;
    opts.num_concurrent_reqs = config.num_concurrent_reqs;

    opts
}

/// build the default resolver from the dns config, and multiple resolvers
/// for each virtual domains.
///
/// # Errors
pub fn build_resolvers(
    config: &Config,
) -> Result<std::collections::HashMap<String, TokioAsyncResolver>, ResolveError> {
    let mut resolvers = std::collections::HashMap::<String, TokioAsyncResolver>::with_capacity(
        config.server.r#virtual.len() + 1,
    );

    resolvers.insert(
        config.server.domain.clone(),
        build_dns_from_config(&config.server.dns)?,
    );

    for virtual_domain in &config.server.r#virtual {
        resolvers.insert(
            virtual_domain.domain.clone(),
            build_dns_from_config(&virtual_domain.dns)?,
        );
    }

    Ok(resolvers)
}

/// build an async dns using tokio & trust_dns from configuration.
///
/// # Errors
///
/// * Failed to create the resolver.
fn build_dns_from_config(config: &ConfigServerDNS) -> Result<TokioAsyncResolver, ResolveError> {
    match &config {
        crate::config::ConfigServerDNS::System => TokioAsyncResolver::tokio_from_system_conf(),
        crate::config::ConfigServerDNS::Google { options } => {
            TokioAsyncResolver::tokio(ResolverConfig::google(), resolver_opts_from_config(options))
        }

        crate::config::ConfigServerDNS::CloudFlare { options } => TokioAsyncResolver::tokio(
            ResolverConfig::cloudflare(),
            resolver_opts_from_config(options),
        ),
        crate::config::ConfigServerDNS::Custom { config, options } => {
            TokioAsyncResolver::tokio(config.clone(), resolver_opts_from_config(options))
        }
    }
}

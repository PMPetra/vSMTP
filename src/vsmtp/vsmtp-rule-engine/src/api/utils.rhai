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
/// get the domain used to identify a recipient.
private fn __in_domain(rcpt) {
    switch rcpt.type_of() {
        "vsmtp_common::type::address::Address" => {
            rcpt.domain in toml::server.virtual || rcpt.domain == toml::server.domain
        },
        "string" => {
            // converting to an address could fail.
            try { return __in_domain(sys::new_address(rcpt)); } catch { return false; }
        },
        _ => false,
    }
}

//! DNS-over-HTTPS JSON implementation
//!
//! This module is _exclusively_ about the JSON-based REST specification that is **not yet**
//! standardised, but that major players have decided to implement following
//! [Google initial implementation](https://developers.google.com/speed/public-dns/docs/dns-over-https).
//!
//! Please look at `dns::provider` for details about providers of this service, that this module
//! can interface with.
//!
//! As I type this, standardization work has began _only_ for
//! ["raw" DNS-over-HTTPS](https://datatracker.ietf.org/doc/rfc8484/): sending DNS message as binary
//! body in an HTTPS request. This modules is ignoring this specification (for now).

pub mod protocol;
pub mod provider;

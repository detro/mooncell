[![Build Status](https://travis-ci.org/detro/mooncell.svg?branch=master)](https://travis-ci.org/detro/mooncell)

# Mooncell

A DNS over HTTPS proxy/bridge.

It receives DNS requests (over UDP and TCP) and resolves them using a user selected provider.

## Requirements for `1.0`

* [x] Full end-to-end resolution
* [x] Configurable port to listen on
* [x] Support for UDP requests
* [ ] Support for TCP requests
* [x] Built in list of providers to pick from
* [x] DNS-over-HTTPS via JSON
* [ ] Handle resolution errors by returning an empty response

## Follow-up features

* [ ] Switch to Rust 2018
* [ ] Adopt Rust official tooling for code formatting/styling
* [ ] A configurable, local cache (in memory to begin with, then look into file backed)
* [ ] DNS-over-HTTPS via binary message
* [ ] User-configurable provider
* [ ] Reach providers via IP, not via FQDN (i.e. resolve at launch, then send `Host` header)

## Related documentation

### IETF
* [IETF draft](https://tools.ietf.org/html/draft-ietf-doh-dns-over-https-14)
* [IETF draft tracker](https://datatracker.ietf.org/doc/rfc8484/)

### (Stable) Providers of DNS-over-HTTPS

* [Google](https://developers.google.com/speed/public-dns/docs/dns-over-https)
* [Cloudflare](https://developers.cloudflare.com/1.1.1.1/dns-over-https/json-format/)
* [Quad9](https://www.quad9.net/doh-quad9-dns-servers/)
* [Rubyfish](https://www.rubyfish.cn/dns-query)
* [BlahDNS](https://blahdns.com/)

### DNS protocol

* [Protocol and format](http://www-inf.int-evry.fr/~hennequi/CoursDNS/NOTES-COURS_eng/msg.html)
* [Message Header and Question Section Format](http://www.tcpipguide.com/free/t_DNSMessageHeaderandQuestionSectionFormat.htm)
* [Let's hand write DNS messages](https://routley.io/tech/2017/12/28/hand-writing-dns-messages.html)

### Other

* [curl DoH wiki/tracker](https://github.com/curl/curl/wiki/DNS-over-HTTPS)
* [Rust doh-proxy](https://github.com/jedisct1/rust-doh) (similar project to this one)

## Compiling

### Windows (x64)

1. Install OpenSSL for Windows 64 bit via the [large dev binaries](http://slproweb.com/products/Win32OpenSSL.html), or in one of the other possible, painful ways
2. Set varialbe `set OPENSSL_DIR=c:\OpenSSL-Win64` (assuming you installed it in the default path)
3. `cargo build` should now work

## Personal notes

* Both `Processor` and `Server` are services (similar to Guava services): 
  you are suppose to start them, stop them and (optionally) wait for them to terminate.
  I think there is a good case here for implementing a tiny crate that provides Trait(s) for services _a la_ Guava.
* I made everything with Threads, but by the end I expect to rewrite everything using proper Rust **async/await**.
  I just could not surmount the Tokio + Hyper learning curve while also doing the same for the Rust language itself.
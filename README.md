# Mooncell

A DNS over HTTPS proxy/bridge.

It receives DNS requests (over UDP and TCP) and resolves them using a user selected provider.

## Requirements for `1.0`

* [x] Configurable port to listen on
* [x] Support for UDP requests
* [ ] Support for TCP requests
* [x] Built in list of providers to pick from
* [x] DNS-over-HTTPS via JSON
* [ ] Reach providers via IP, not via FQDN (i.e. resolve at launch, then send `Host` header)

## Follow-up features

* [ ] DNS-over-HTTPS via binary message
* [ ] User-configurable provider

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
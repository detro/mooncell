# WORK IN PROGRESS: for now, it's ok to stay away...

---

# Mooncell

DNS over HTTPS, for us.

## Reading resources

* https://developers.google.com/speed/public-dns/docs/dns-over-https (Google DNS documentation)
* https://developers.cloudflare.com/1.1.1.1/dns-over-https/json-format/ (Cloudflare DNS documentation)

* http://www-inf.int-evry.fr/~hennequi/CoursDNS/NOTES-COURS_eng/msg.html (DNS Protocol Message format)

* https://datatracker.ietf.org/doc/draft-ietf-doh-dns-over-https/
* https://github.com/curl/curl/wiki/DNS-over-HTTPS
* https://github.com/jedisct1/rust-doh (similar project)

## Notes

It should launch and find a resolve a suitable IP for the given Provider's Hostnames.

This means that instead of using the "hostname" to connect to the provider DoH resolver, it would connect using the
pre-resolved IP.

I tested with Google and it works as long as the header "Host: dns.google.com" is added to the request. 

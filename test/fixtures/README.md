This directory contains test fixtures: in the specific, DNS messages.

The fixtures are created using [netcat (nc)](http://netcat.sourceforge.net/)
and [dig](https://linux.die.net/man/1/dig): `nc` is used to listen for requests and drops
the received packets on the standard-output; `dig` executes the actual DNS queries.

NOTE: `nc` will listen on port `1053` to avoid having to summon raise privileges 
(i.e. `root` in Unix, `Administrator` on Windows). We will direct `dig` queries to the same port.

### `nc`

* `nc -u -l -p 1053 > output`: listen for UDP packets on port 1053

#### `nc` vs TCP

Even though `nc` can listen for TCP connections and receive request traffic, it seems it's unable
to separate the body of the request from the datagram used to establish the connection.
So this approach to capturing packets, for TCP, doesn't work. 

### `dig`

* `dig +retry=0 -p 1053 @127.0.0.1 example.com`: sends a single DNS query over UDP at port 1053
* `dig +retry=0 -p 1053 @127.0.0.1 +noedns example.com`: sends a single DNS query over UDP, without EDNS extensions, at port 1053
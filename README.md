# Rudis
This is a lightweight Redis clone written in Rust.

# Benchmark Results
Gets ~295k requests per second, outperforming stock Redis (v8.2.1) 
running under identical conditions by approx. 8% in a single-threaded, in-memory-only benchmark 
using "redis-benchmark".
```
// Rudis (redis-benchmark -p 3780 -q)
PING_INLINE: 294985.25 requests per second, p50=0.095 msec
PING_MBULK: 295858.00 requests per second, p50=0.095 msec
SET: 294117.66 requests per second, p50=0.095 msec
GET: 297619.06 requests per second, p50=0.095 msec
INCR: 299401.22 requests per second, p50=0.095 msec
LPUSH: 296735.91 requests per second, p50=0.095 msec
RPUSH: 294117.66 requests per second, p50=0.095 msec
LPOP: 293255.12 requests per second, p50=0.095 msec
RPOP: 295858.00 requests per second, p50=0.095 msec

// Standard Redis (redis-benchmark -q)
PING_INLINE: 271739.12 requests per second, p50=0.095 msec
PING_MBULK: 266666.66 requests per second, p50=0.095 msec
SET: 280898.88 requests per second, p50=0.087 msec
GET: 276243.09 requests per second, p50=0.095 msec
INCR: 281690.16 requests per second, p50=0.087 msec
LPUSH: 289855.06 requests per second, p50=0.087 msec
RPUSH: 282485.88 requests per second, p50=0.095 msec
LPOP: 287356.34 requests per second, p50=0.087 msec
RPOP: 286532.94 requests per second, p50=0.087 msec
```

# TODO
- [ ] Implement more commands

## Efficiency
- [x] Reduce number of now() calls and UNIX comparisons in memory.rs (and perhaps even in command.rs)
  - As of 5/8/2026, 7.84% of redis-benchmark -t ping is now() calls
- [ ] Reduce number of to_string() in Command::execute()
  - As of 5/8/2026, 1.25% of redis-benchmark -t ping is Execute, more than half of that (0.62%) being to_string()
- [ ] Maybe find a better way to handle Command::from_resp()
  - As of 5/8/2026, 2.82% of ping is from_resp, 20% of that being into_iter, and a bit more being ignore_ascii_case (25% ish)
- [ ] Make Server::handle_connection() more efficient
  - As of 5/8/2026, 9.82% total is Server::run(), with Server::handle_connection() being 7.41% total
- [ ] Reduce malloc in Command::parse()
  - As of 5/8/2026, 1/3 of parse's 3% total is malloc from parse_inline

# Rudis
This is a lightweight Redis clone written in Rust.

# TODO
- [ ] Implement more commands

## Efficiency
- [ ] Reduce number of now() calls and UNIX comparisons in memory.rs (and perhaps even in command.rs)
  - As of 5/8/2026, 7.84% of redis-benchmark -t ping is now() calls
- [ ] Reduce number of to_string() in Command::execute()
  - As of 5/8/2026, 1.25% of redis-benchmark -t ping is Execute, more than half of that (0.62%) being to_string()
- [ ] Maybe find a better way to handle Command::from_resp()
  - As of 5/8/2026, 2.82% of ping is from_resp, 20% of that being into_iter, and a bit more being ignore_ascii_case (25% ish)
- [ ] Make Server::handle_connection() more efficient
  - As of 5/8/2026, 9.82% total is Server::run(), with Server::handle_connection() being 7.41% total
- [ ] Reduce malloc in Command::parse()
  - As of 5/8/2026, 1/3 of parse's 3% total is malloc from parse_inline

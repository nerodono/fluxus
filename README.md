# Summary

Fluxus is the open source network tunnel server.

# Plan

## Authorization

- [ ] Database
- [x] Universal password

## Supported protocols

- [ ] **HTTP** - `high priority` (in progress)
- [x] **TCP**  - `high priority`
- [ ] **UDP**  - `low priority`

## Internals

- [x] Packet compression
  - [ ] Ability to change algorithm (Currently algorithm is semi-hardcoded)
  - [ ] Enable dictionary loading (mainly zstd)
- [ ] Protocol similar to the `ENet` instead of plain `TCP`

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Bit operations: `BitIndex`, `DIGIT_BITS_USIZE`, `count_ones`.
- Addition.
- Subtraction.
- Small bit shifts.
- More sign extension functions.

### Changed

- Function names now describe every operand explicitly rather than defaulting to unsigned, using
  the vocabulary `unsigned`/`signed`/`digit`/`idigit`/`carry`/`icarry`/`borrow` (e.g.
  `min_len` → `min_len_unsigned`, `bit` → `bit_unsigned`, `to_bytes` → `to_bytes_unsigned`).
- Renamed `next_power_of_two_in_place` to `next_power_of_two`.
- `bit_unsigned`, `bit_signed` and `set_bit` now take a `BitIndex` instead of a bit position
  `usize`.
- Replaced `bit_width`, `trailing_zeros` and `trailing_ones` with `highest_one`, `lowest_one`
  and `lowest_zero`, which return `Option<BitIndex>`.
- `sign_extension` now operates on a signed slice (returning its `IDigit` sign digit); the
  single-digit version is `sign_extension_idigit`.

## [0.0.1] - 2026-06-05

Initial version with very little functionality.

### Added

- The `Digit`/`IDigit` types: the platform's native unsigned/signed integer.
- Bit operations: bit access and assignment, bit width, trailing zeros/ones, power-of-two test, and rounding up to the next power of two.
- Bitwise logic on equal-length slices: NOT, AND, OR, XOR, and AND-NOT.
- Conversions to and from little- and big-endian bytes, for both unsigned magnitudes and signed two's-complement values.
- Length and sign helpers: canonical digit/byte lengths, sign extension, and the sign test for two's-complement values.

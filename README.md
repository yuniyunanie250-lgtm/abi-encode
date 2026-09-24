# abi-encode

Static ABI encoding in Rust: typed values in, 32-byte words out, no dependencies.

The counterpart to any decoder. Static types are the ones that fit in a single
32-byte word, which covers most calldata you will ever build by hand: transfers,
approvals, simple setters.

## Usage

```rust
use abi_encode::{encode, to_hex};

let out = encode(&[
    ("address", "0xd8da6bf26964af9d7eed9e03e53415d37aa96045"),
    ("uint256", "1000000000000000000"),
]).unwrap();
assert_eq!(out.len(), 64);
```

## Behaviours worth knowing

- **Dynamic types are rejected, not guessed at.** A `string`, `bytes` or array
  argument's encoding depends on every other argument in the call, because of
  head/tail offsets. Encoding one in isolation yields a payload that is silently
  wrong for the real function, which is worse than an error.
- **Narrow uints are range-checked.** `uint8` with `256` is an error, not a
  truncated word.
- **Addresses longer than 20 bytes are refused** rather than silently clipped.
- **`bytes<M>` is width-checked** and left-aligned, matching the spec's padding.
- **Decimal integers are parsed digit by digit** into a 256-bit accumulator, so a
  value beyond `u128` still works and overflow past 2^256 is caught.

## What it does not do

- **No signed encoding.** `int<M>` needs two's complement on a 256-bit value;
  add it if you need it.
- **No dynamic types or tuples.** See above.
- **No selector prefixing.** Concatenate the 4-byte selector yourself, from
  `selector-db` or `keccak256`.

## Development

```bash
cargo test
```

## License

MIT

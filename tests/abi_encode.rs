use abi_encode::{address, boolean, encode, fixed_bytes, to_hex, uint, EncodeError};

#[test]
fn one_ether_encodes_to_the_expected_word() {
    assert_eq!(
        to_hex(&uint("uint256", "1000000000000000000").unwrap()),
        "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000"
    );
}

#[test]
fn an_address_lands_in_the_low_twenty_bytes() {
    let w = address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();
    assert_eq!(
        to_hex(&w),
        "0x000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045"
    );
}

#[test]
fn addresses_longer_than_twenty_bytes_are_rejected() {
    let too_long = format!("0x{}", "11".repeat(21));
    assert!(matches!(
        address(&too_long),
        Err(EncodeError::AddressTooLong(21))
    ));
}

#[test]
fn bool_is_zero_or_one() {
    assert_eq!(boolean(false)[31], 0);
    assert_eq!(boolean(true)[31], 1);
}

#[test]
fn narrow_uint_widths_are_range_checked() {
    assert!(uint("uint8", "255").is_ok());
    assert!(matches!(
        uint("uint8", "256"),
        Err(EncodeError::WrongWidth(8))
    ));
    assert!(uint(
        "uint256",
        "115792089237316195423570985008687907853269984665640564039457584007913129639935"
    )
    .is_ok());
}

#[test]
fn overflow_past_256_bits_is_refused() {
    let too_big = "115792089237316195423570985008687907853269984665640564039457584007913129639936";
    assert!(uint("uint256", too_big).is_err());
}

#[test]
fn fixed_bytes_are_left_aligned_and_width_checked() {
    assert_eq!(
        to_hex(&fixed_bytes("bytes4", "0xa9059cbb").unwrap()),
        "0xa9059cbb00000000000000000000000000000000000000000000000000000000"
    );
    assert!(matches!(
        fixed_bytes("bytes4", "0xa9059cbb00"),
        Err(EncodeError::WrongWidth(4))
    ));
}

#[test]
fn dynamic_types_are_rejected_not_mis_encoded() {
    let err = encode(&[("string", "hi")]).unwrap_err();
    assert!(matches!(err, EncodeError::UnsupportedType(_)));
}

#[test]
fn a_transfer_call_encodes_end_to_end() {
    let out = encode(&[
        ("address", "0xd8da6bf26964af9d7eed9e03e53415d37aa96045"),
        ("uint256", "1000000000000000000"),
    ])
    .unwrap();
    assert_eq!(out.len(), 64);
    assert_eq!(
        to_hex(&out),
        "0x000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045\
           0000000000000000000000000000000000000000000000000de0b6b3a7640000"
    );
}

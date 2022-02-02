use bech32::{convert_bits, decode, encode, CheckBase32, Error, Variant};

// decodes a bech32 encoded string and converts to base64 encoded bytes
fn decode_and_convert(bech: &str) -> Result<(String, Vec<u8>), Error> {
    let decoded = decode(bech)?;
    let hrp = decoded.0;
    let data = decoded.1;
    let converted = convert_bits(&data, 5, 8, false)?;

    Ok((hrp, converted))
}

// converts from a base64 encoded byte string to base32 encoded byte string and then to bech32
fn convert_and_encode(hrp: String, data: Vec<u8>) -> Result<String, Error> {
    let converted = convert_bits(&data, 8, 5, true)?;
    encode(&hrp, converted.check_base32()?, Variant::Bech32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_stars() {
        let decoded = decode_and_convert("cosmos1ey69r37gfxvxg62sh4r0ktpuc46pzjrmz29g45").unwrap();
        let addr = convert_and_encode("stars".to_string(), decoded.1).unwrap();
        assert_eq!(
            addr,
            "stars1ey69r37gfxvxg62sh4r0ktpuc46pzjrmkkj479".to_string()
        );
    }
}

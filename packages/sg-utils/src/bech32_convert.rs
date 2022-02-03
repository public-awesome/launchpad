use bech32::{convert_bits, decode, encode, CheckBase32, Error, Variant};
use cosmwasm_std::Addr;

const PREFIX: &str = "stars";

trait ToStars {
    fn to_stars(&self) -> String;
}

impl ToStars for String {
    fn to_stars(&self) -> String {
        let decoded = decode_and_convert(self).unwrap();
        convert_and_encode(PREFIX.to_string(), decoded.1).unwrap()
    }
}

impl ToStars for str {
    fn to_stars(&self) -> String {
        let decoded = decode_and_convert(self).unwrap();
        convert_and_encode(PREFIX.to_string(), decoded.1).unwrap()
    }
}

impl ToStars for Addr {
    fn to_stars(&self) -> String {
        let decoded = decode_and_convert(&self.to_string()).unwrap();
        convert_and_encode(PREFIX.to_string(), decoded.1).unwrap()
    }
}

// decodes a bech32 encoded string and converts to base64 encoded bytes
pub fn decode_and_convert(bech: &str) -> Result<(String, Vec<u8>), Error> {
    let decoded = decode(bech)?;
    let hrp = decoded.0;
    let data = decoded.1;
    let converted = convert_bits(&data, 5, 8, false)?;

    Ok((hrp, converted))
}

// converts from a base64 encoded byte string to base32 encoded byte string and then to bech32
pub fn convert_and_encode(hrp: String, data: Vec<u8>) -> Result<String, Error> {
    let converted = convert_bits(&data, 8, 5, true)?;
    encode(&hrp, converted.check_base32()?, Variant::Bech32)
}

#[cfg(test)]
mod tests {
    use crate::bech32_convert::{convert_and_encode, decode_and_convert, ToStars};
    use cosmwasm_std::Addr;

    #[test]
    fn decode_encode() {
        let decoded = decode_and_convert("cosmos1ey69r37gfxvxg62sh4r0ktpuc46pzjrmz29g45").unwrap();
        let addr = convert_and_encode("stars".to_string(), decoded.1).unwrap();
        assert_eq!(
            addr,
            "stars1ey69r37gfxvxg62sh4r0ktpuc46pzjrmkkj479".to_string()
        );
    }

    #[test]
    fn string_to_stars() {
        let addr = "cosmos1ey69r37gfxvxg62sh4r0ktpuc46pzjrmz29g45"
            .to_string()
            .to_stars();
        assert_eq!(
            addr,
            "stars1ey69r37gfxvxg62sh4r0ktpuc46pzjrmkkj479".to_string()
        );
    }

    #[test]
    fn str_to_stars() {
        let addr = "cosmos1ey69r37gfxvxg62sh4r0ktpuc46pzjrmz29g45".to_stars();
        assert_eq!(
            addr,
            "stars1ey69r37gfxvxg62sh4r0ktpuc46pzjrmkkj479".to_string()
        );
    }

    #[test]
    fn addr_to_stars() {
        let addr = Addr::unchecked("cosmos1ey69r37gfxvxg62sh4r0ktpuc46pzjrmz29g45").to_stars();
        assert_eq!(
            addr,
            "stars1ey69r37gfxvxg62sh4r0ktpuc46pzjrmkkj479".to_string()
        );
    }
}

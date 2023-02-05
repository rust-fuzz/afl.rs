use afl::fuzz;
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    #[must_use]
    pub fn as_hex(&self) -> Hex {
        let Rgb { r, g, b } = self;
        Hex(format!("{r:02X}{g:02X}{b:02X}"))
    }
}

pub struct Hex(String);

impl Hex {
    fn as_rgb(&self) -> Rgb {
        let s = self.0.as_str();

        let r = u8::from_str_radix(&s[..2], 16).unwrap();
        let g = u8::from_str_radix(&s[2..4], 16).unwrap();
        let b = u8::from_str_radix(&s[4..6], 16).unwrap();

        Rgb { r, g, b }
    }
}

pub fn main() {
    fuzz!(|color: Rgb| {
        let hex = color.as_hex();
        let rgb = hex.as_rgb();

        assert_eq!(color, rgb);
    });
}

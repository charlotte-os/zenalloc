pub struct ZenAsciiChar(u8);

impl ZenAsciiChar {
    pub fn new(c: char) -> Option<Self> {
        if c.is_ascii() {
            Some(ZenAsciiChar(c as u8))
        } else {
            None
        }
    }

    pub fn as_char(self) -> char {
        self.0 as char
    }

    pub fn as_u8(self) -> u8 {
        self.0
    }
}

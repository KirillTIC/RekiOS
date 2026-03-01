extern crate alloc;
use alloc::collections::BTreeMap;

#[repr(C, packed)]
struct Psf2Header {
    magic: [u8; 4],
    version: u32,
    header_size: u32,
    flags: u32,
    num_glyphs: u32,
    bytes_per_glyph: u32,
    height: u32,
    width: u32,
}

pub struct Psf2Font {
    header: &'static Psf2Header,
    glyphs: &'static [u8],
    unicode_map: BTreeMap<char, usize>,
}

impl Psf2Font {
    pub fn new(data: &'static [u8]) -> Self {
        let header = unsafe { &*(data.as_ptr() as *const Psf2Header) };
        assert_eq!(header.magic, [0x72, 0xB5, 0x4A, 0x86]);

        let glyph_data_size = header.num_glyphs as usize * header.bytes_per_glyph as usize;
        let glyphs =
            &data[header.header_size as usize..header.header_size as usize + glyph_data_size];

        let unicode_table = &data[header.header_size as usize + glyph_data_size..];
        let unicode_map = Self::parse_unicode_table(unicode_table, header.num_glyphs as usize);

        Self {
            header,
            glyphs,
            unicode_map,
        }
    }

    fn parse_unicode_table(table: &[u8], num_glyphs: usize) -> BTreeMap<char, usize> {
        let mut map = BTreeMap::new();
        let mut glyph_idx = 0;
        let mut i = 0;

        while i < table.len() && glyph_idx < num_glyphs {
            if table[i] == 0xFF {
                glyph_idx += 1;
                i += 1;
            } else if table[i] == 0xFE {
                i += 1;
            } else {
                let (c, len) = Self::parse_utf8(&table[i..]);
                if let Some(c) = c {
                    map.insert(c, glyph_idx);
                }
                i += len;
            }
        }
        map
    }

    fn parse_utf8(bytes: &[u8]) -> (Option<char>, usize) {
        if bytes.is_empty() {
            return (None, 1);
        }
        match core::str::from_utf8(bytes) {
            Ok(s) => {
                if let Some(c) = s.chars().next() {
                    (Some(c), c.len_utf8())
                } else {
                    (None, 1)
                }
            }
            Err(e) => {
                let valid = e.valid_up_to();
                if valid > 0 {
                    let c = core::str::from_utf8(&bytes[..valid])
                        .ok()
                        .and_then(|s| s.chars().next());
                    (c, valid)
                } else {
                    (None, 1)
                }
            }
        }
    }

    pub fn get_glyph(&self, c: char) -> &[u8] {
        let idx = self.unicode_map.get(&c).copied().unwrap_or(0);
        let start = idx * self.header.bytes_per_glyph as usize;
        &self.glyphs[start..start + self.header.bytes_per_glyph as usize]
    }

    pub fn width(&self) -> u32 {
        self.header.width
    }

    pub fn height(&self) -> u32 {
        self.header.height
    }
}

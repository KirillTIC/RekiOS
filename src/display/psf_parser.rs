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
}

impl Psf2Font {
    pub fn new(data: &'static [u8]) -> Self {
        let header = unsafe { &*(data.as_ptr() as *const Psf2Header) };
        let glyphs = &data[header.header_size as usize..];

        assert_eq!(header.magic, [0x72, 0xB5, 0x4A, 0x86]);
        Self { header, glyphs }
    }
    pub fn get_glyph(&self, c: char) -> &[u8] {
        let idx = (c as usize).min(self.header.num_glyphs as usize - 1);
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

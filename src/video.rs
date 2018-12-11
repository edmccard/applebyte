use machine_int::MachineInt;

type Addr = MachineInt<u16>;

#[repr(u32)]
pub enum Page {
    One = 0,
    Two = 1,
}

#[derive(Copy, Clone)]
pub struct ScrPos {
    line: Addr,
    col: Addr,
}

impl ScrPos {
    pub fn addr_lores(self, page: Page, ii: bool) -> Addr {
        let addr = self.scan_pos().display_addr_lores(page);
        if ii && self.col < 25 {
            addr | 0x1000
        } else {
            addr
        }
    }

    pub fn addr_hires(self, page: Page) -> Addr {
        self.scan_pos().display_addr_hires(page)
    }

    fn scan_pos(self) -> ScrPos {
        ScrPos {
            // lines 256-261 scan the same memory as lines 250-255
            line: if self.line > 255 {
                self.line - 6
            } else {
                self.line
            },
            // columns 0 and 1 scan the same address
            col: std::cmp::max(self.col, MachineInt(1)),
        }
    }

    fn display_addr_lores(self, page: Page) -> Addr {
        // 8 scanlines are grouped into one row.
        let row = self.line >> 3;

        // Screen memory is divided into 8 128-byte blocks.
        let base = (row & 7) << 7;

        // Each 128 byte block is divided into top 40, middle 40,
        // bottom 40, unused 8.
        let region = (self.line >> 6) * 40;

        // Horizontal scanning wraps at 128-byte boundaries.
        let offset = (region + (self.col - 25)) & 0x7f;

        (MachineInt(0x0400u16) << (page as u32)) | (base + offset)
    }

    fn display_addr_hires(self, page: Page) -> Addr {
        // Screen memory is divided into 8 1-kb blocks.
        let blk_1k = (self.line & 7) << 10;

        // Each 1k block is divided into 8 128-byte blocks.
        let blk_128 = ((self.line & 63) >> 3) << 7;

        // Each 128 byte block is divided into top 40, middle 40,
        // bottom 40, unused 8.
        let region = (self.line >> 6) * 40;

        // Horizontal scanning wraps at 128-byte boundaries.
        let offset = (region + (self.col - 25)) & 0x7f;

        (MachineInt(0x2000u16) << (page as u32)) | (blk_1k + blk_128 + offset)
    }

    pub fn scanline_lores(addr: Addr) -> Addr {
        let addr = addr & 0x3ff;
        let row = addr >> 7;
        let region = (addr & 127) / 40;
        ((region << 3) + row) << 3
    }

    pub fn scanline_hires(addr: Addr) -> Addr {
        let addr = addr & 0x1fff;
        let blk_1k = addr >> 10;
        let blk_128 = (addr & 0x3ff) >> 7;
        let region = (addr & 127) / 40;
        (region << 6) + (blk_128 << 3) + blk_1k
    }
}

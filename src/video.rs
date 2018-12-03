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
        // region 0,1,2,3 = top, middle, bottom, vblank
        let region = self.line >> 6;

        // row groups 8 scanlines into one row
        let row = self.line >> 3;

        // Horizontal scanning wraps at 128-byte boundaries ("& 0x7f")
        let base = ((region << 5) + (region << 3) + (self.col - 25)) & 0x7f;
        let offset = (row & 7) << 7;
        (MachineInt(0x0400u16) << (page as u32)) | (base + offset)
    }

    fn display_addr_hires(self, page: Page) -> Addr {
        // region 0,1,2,3 = top, middle, bottom, vblank
        let region = self.line >> 6;

        // block divides each 64-line region into blocks of 8 lines
        let block = (self.line & 63) >> 3;

        // Horizontal scanning wraps at 128-byte boundaries ("& 0x7f")
        let base = ((region << 5) + (region << 3) + (self.col - 25)) & 0x7f;
        let offset = (block << 7) + ((self.line & 7) << 10);

        (MachineInt(0x2000u16) << (page as u32)) | (base + offset)
    }
}

#[cfg(test)]
mod test {
    use super::{Page, ScrPos};
    use machine_int::MachineInt;
    #[test]
    fn hires_base() {
        for x in 0..262 {
            let pos = ScrPos {
                line: MachineInt(x),
                col: MachineInt(25),
            };
            println!("{}: {:04x}", x, pos.addr_hires(Page::One));
        }
    }
}

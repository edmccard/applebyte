use crate::video::{Addr, Page};
use crate::Model;
use machine_int::MachineInt;

#[derive(Copy, Clone, Debug)]
pub(super) struct ScanPos {
    pub(super) line: Addr,
    pub(super) col: Addr,
}

impl ScanPos {
    /// The memory address scanned in LORES/TEXT mode for this scan
    /// position.
    pub fn addr_lores(self, page: Page, model: Model) -> Addr {
        let addr = self.uniq_pos().display_addr_lores(page);
        if model < Model::IIe && self.col < 25 {
            addr | 0x1000
        } else {
            addr
        }
    }

    /// The memory address scanned in HIRES mode for this scan
    /// position.
    pub fn addr_hires(self, page: Page) -> Addr {
        self.uniq_pos().display_addr_hires(page)
    }

    fn uniq_pos(self) -> ScanPos {
        ScanPos {
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

    pub fn display_addr_lores(self, page: Page) -> Addr {
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

    pub fn display_addr_hires(self, page: Page) -> Addr {
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

#[cfg(test)]
mod test_scanpos {
    use super::*;

    #[test]
    fn lores_addresses() {
        let counters = counters();
        let posns = posns();
        for i in 0..17030 {
            let expected = addr_lores(counters[i], Page::One, Model::IIe);
            let got = posns[i].addr_lores(Page::One, Model::IIe);
            assert_eq!(
                expected, got.0,
                "expected {:04x}, got {:04x} for {}.{}",
                expected, got.0, posns[i].line.0, posns[i].col.0
            );
        }
    }

    #[test]
    fn hires_addresses() {
        let counters = counters();
        let posns = posns();
        for i in 0..17030 {
            let expected = addr_hires(counters[i], Page::One);
            let got = posns[i].addr_hires(Page::One);
            assert_eq!(
                expected, got.0,
                "expected {:04x}, got {:04x} for {}.{}",
                expected, got.0, posns[i].line.0, posns[i].col.0
            );
        }
    }

    fn addr_base(counter: (u16, u16)) -> u16 {
        let vert = counter.0 & 0b11111111;
        let horiz = counter.1 & 0b111111;
        let a2a1a0 = horiz & 0b111;
        let a9a8a7 = ((vert >> 3) & 0b111) << 7;
        let v4v3v4v3 = ((vert & 0b11000000) >> 4) | (vert >> 6);
        let h5h4h3 = horiz >> 3;
        let a6a5a4a3 = ((0b1101 + h5h4h3 + v4v3v4v3) & 0b1111) << 3;
        a9a8a7 | a6a5a4a3 | a2a1a0
    }

    fn addr_lores(counter: (u16, u16), page: Page, model: Model) -> u16 {
        let addr = addr_base(counter) | (0x400 << (page as u32));
        if model < Model::IIe && (counter.1 < 0b1011000) {
            addr | 0x1000
        } else {
            addr
        }
    }

    fn addr_hires(counter: (u16, u16), page: Page) -> u16 {
        let addr = addr_base(counter) | (0x2000 << (page as u32));
        addr | ((counter.0 & 0b111) << 10)
    }

    fn counters() -> Vec<(u16, u16)> {
        let mut counters = vec![];
        let mut v_count = 0b100000000;
        let mut h_count = 0b0000000;
        loop {
            counters.push((v_count, h_count));
            if h_count == 0 {
                h_count = 0b1000000;
            } else if h_count == 0b1111111 {
                h_count = 0;
                if v_count == 0b111111111 {
                    v_count = 0b011111010;
                } else if v_count == 0b011111111 {
                    break;
                } else {
                    v_count = v_count + 1;
                }
            } else {
                h_count = h_count + 1;
            }
        }
        counters
    }

    fn posns() -> Vec<ScanPos> {
        let mut posns = vec![];
        for line in 0..262 {
            for col in 0..65 {
                posns.push(ScanPos {
                    line: MachineInt(line),
                    col: MachineInt(col),
                });
            }
        }
        posns
    }
}

pub(super) struct DrawIter {
    start: ScanPos,
    end: ScanPos,
}

impl DrawIter {
    // Assumes start.line <= end.line
    pub(super) fn new(start: ScanPos, end: ScanPos) -> DrawIter {
        let start = ScanPos {
            line: start.line,
            col: std::cmp::max(start.col, MachineInt(25)),
        };
        let end = if end.line < 192 {
            ScanPos {
                line: end.line,
                col: if end.col < 25 { MachineInt(0) } else { end.col },
            }
        } else {
            ScanPos {
                line: MachineInt(192),
                col: MachineInt(0),
            }
        };
        DrawIter { start, end }
    }
}

impl Iterator for DrawIter {
    type Item = (Addr, Addr, Addr);

    fn next(&mut self) -> Option<(Addr, Addr, Addr)> {
        if self.start.line < self.end.line {
            let result = (self.start.line, self.start.col, MachineInt(65));
            self.start.line += 1;
            self.start.col = MachineInt(25);
            Some(result)
        } else if self.start.line > self.end.line
            || self.start.col >= self.end.col
        {
            None
        } else {
            // start.line == end.line
            let result = (self.start.line, self.start.col, self.end.col);
            self.start.line += 1;
            Some(result)
        }
    }
}

#[cfg(test)]
mod test_drawiter {
    use super::*;

    #[test]
    fn blank() {
        assert!(lines(new_pos(0, 0), new_pos(0, 25)).is_empty());
        assert!(lines(new_pos(0, 25), new_pos(0, 24)).is_empty());
        assert!(lines(new_pos(1, 25), new_pos(0, 26)).is_empty());
        assert!(lines(new_pos(192, 25), new_pos(193, 25)).is_empty());
    }

    #[test]
    fn non_blank() {
        let draw_lines = lines(new_pos(0, 0), new_pos(0, 26));
        assert_eq!(draw_lines.len(), 1);
        assert!(draw_lines.iter().all(visible));

        let draw_lines = lines(new_pos(0, 25), new_pos(1, 0));
        assert_eq!(draw_lines.len(), 1);
        assert!(draw_lines.iter().all(visible));

        let draw_lines = lines(new_pos(0, 0), new_pos(262, 0));
        assert_eq!(draw_lines.len(), 192);
        assert!(draw_lines.iter().all(visible));
    }

    fn new_pos(line: u16, col: u16) -> ScanPos {
        ScanPos {
            line: MachineInt(line),
            col: MachineInt(col),
        }
    }

    fn lines(start: ScanPos, end: ScanPos) -> Vec<(Addr, Addr, Addr)> {
        let iter = DrawIter::new(start, end);
        iter.collect()
    }

    fn visible(draw_line: &(Addr, Addr, Addr)) -> bool {
        draw_line.0 < 192
            && (draw_line.1 >= 25 && draw_line.1 <= 65)
            && (draw_line.2 >= 25 && draw_line.2 <= 65)
    }
}

use machine_int::MachineInt;

use crate::video::Addr;

const PIXEL: [u8; 2] = [0, 0xff];

// video calls draw at least once per scanline
// at the start of the scanline in hires, figure out delay bit
// (and clear 0-6 if not dhgr)
// (prev hbl for ii, 0 for iie)
// the signal methods return the next delay bit
// maybe prev_bit is a member?

struct Signal {
    pixels: Vec<u8>,
    pos: MachineInt<u32>,
}

impl Signal {
    fn dot_14m(&mut self, pos: MachineInt<u32>, pixel: MachineInt<u8>) {
        self.pixels[usize::from(pos)] = PIXEL[usize::from(pixel & 0b1)];
    }

    fn byte_14m(&mut self, pixel: MachineInt<u8>) {
        self.dot_14m(self.pos, pixel);
        self.dot_14m(self.pos + 1, pixel >> 1);
        self.dot_14m(self.pos + 2, pixel >> 2);
        self.dot_14m(self.pos + 3, pixel >> 3);
        self.dot_14m(self.pos + 4, pixel >> 4);
        self.dot_14m(self.pos + 5, pixel >> 5);
        self.dot_14m(self.pos + 6, pixel >> 6);
        self.pos += 7;
    }

    fn dot_7m(&mut self, pos: MachineInt<u32>, pixel: MachineInt<u8>) {
        self.dot_14m(pos, pixel);
        self.dot_14m(pos + 1, pixel);
    }

    fn byte_7m(&mut self, byte: MachineInt<u8>) {
        self.dot_7m(self.pos, byte);
        self.dot_7m(self.pos + 2, byte >> 1);
        self.dot_7m(self.pos + 4, byte >> 2);
        self.dot_7m(self.pos + 6, byte >> 3);
        self.dot_7m(self.pos + 8, byte >> 4);
        self.dot_7m(self.pos + 10, byte >> 5);
        self.dot_7m(self.pos + 12, byte >> 6);
        self.pos += 7;
    }

    fn start_scanline(&mut self, line: Addr, is_80: bool) {
        const BLACK: MachineInt<u8> = MachineInt(0);
        let line: MachineInt<u32> = line.into();
        self.pos = line * 567;
        if !is_80 {
            self.byte_14m(BLACK);
        }
    }

    fn hires(&mut self, mem: &[u8], prev_bit: u8) -> u8 {
        let mut prev_bit = MachineInt(prev_bit);
        for &byte in mem {
            let byte = MachineInt(byte);

            self.dot_14m(self.pos, prev_bit);
            prev_bit = (byte >> 6);

            self.pos += (byte >> 7);
            self.byte_7m(byte);
            self.pos -= (byte >> 7);
        }
        prev_bit.0
    }

    fn hires_80(&mut self, aux: &[u8], main: &[u8]) -> u8 {
        for i in 0..aux.len() {
            let pattern = MachineInt(aux[i]);
            self.byte_14m(pattern);
            let pattern = MachineInt(main[i]);
            self.byte_14m(pattern);
        }
        main[main.len() - 1] >> 6
    }

    fn lores(&mut self, mem: &[u8], line: Addr, start_addr: Addr) -> u8 {
        // rot = 2 for odd addresses, 0 for even
        let mut rot = (start_addr << 1) & 2;
        for &byte in mem {
            let nibble = (MachineInt(byte) >> u32::from(line & 0b100)) & 0b1111;

            let pattern = (nibble | (nibble << 4)).rotate_right(rot);
            self.byte_14m(pattern);
            self.byte_14m(pattern.rotate_left(1));

            rot ^= 2;
        }
        // The last value of Q0/QH will be either bit 1 (for even
        // addresses) or bit 3 (for odd) of the last scanned data.
        // (Note that rot must be flipped again to undo the flip that
        // happens *after* the last byte is processed.)
        mem[mem.len() - 1] >> u32::from((rot ^ 2) + 1)
    }
}

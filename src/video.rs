use machine_int::MachineInt;

mod scanpos;
mod signal;

use self::scanpos::ScanPos;

type Addr = MachineInt<u16>;

#[repr(u32)]
pub enum Page {
    One = 0,
    Two = 1,
}

struct Video {
    lores_main: Vec<u8>,
    hires_main: Vec<u8>,
    lores_aux: Vec<u8>,
    hires_aux: Vec<u8>,

    last_pos: ScanPos,
    pos: ScanPos,

    mixed: bool,
}

impl Video {
    // Returns None if it is time to draw a frame.
    fn pre_tick(&self) -> Option<()> {
        if self.pos.line == 0 && self.pos.col == 0 {
            None
        } else {
            Some(())
        }
    }

    fn post_tick(&mut self) {
        self.pos.col += 1;
        if self.pos.col == 65 {
            // New scanline
            self.pos.col = MachineInt(0);
            self.pos.line += 1;
            if self.mixed {
                match self.pos.line {
                    // to TEXT
                    MachineInt(160) => (),
                    // to GRAPHICS in VBL
                    MachineInt(192) => (),
                    // to TEXT in VBL
                    MachineInt(224) => (),
                    // TODO: PAL
                    MachineInt(262) => {
                        // to GRAPHICS
                        self.pos.line = MachineInt(0);
                    }
                    _ => (),
                }
            } else {
                // TODO: PAL
                if self.pos.line == 262 {
                    self.pos.line = MachineInt(0);
                    // end of frame
                }
            }
            // TODO: signal.start_scanline(self.pos.line, is_80)
        }
    }

    // fn draw() {
        // feed last_scanned to signal gen
        // for foo in drawiter
        //    feed range to signal gen
        // figure out last_scanned (maybe None) and prev_bit
    // }

    // tick vs. memory access: what order?
    //  pre-tick: -- anything that can interrupt goes here
    //  mem access
    //  post-tick:

}

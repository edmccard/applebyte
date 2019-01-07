use machine_int::MachineInt;

mod scanpos;

pub use self::scanpos::ScanPos;

type Addr = MachineInt<u16>;

#[repr(u32)]
pub enum Page {
    One = 0,
    Two = 1,
}

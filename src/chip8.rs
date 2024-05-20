#![allow(unused)]

const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

struct Chip8 {
    opcode: u16, // Operation Code
    i: u16,      // Index Register
    pc: u16,     // Program Counter
    dt: u8,      // Delay Timer
    st: u8,      // Sound Timer
    sp: u16,     // Stack Pointer
    mem: [u8; 4096],
    display: [u8; 64 * 32],
    registers: [u8; 16],
    stack: [u16; 16],
    keys: [u8; 16],
}
impl Chip8 {
    pub fn new() -> Self {
        Chip8 {
            opcode: 0,
            i: 0,
            pc: 0x200,
            dt: 0,
            st: 0,
            sp: 0,
            mem: [0; 4096],
            display: [0; 64 * 32],
            registers: [0; 16],
            stack: [0; 16],
            keys: [0; 16],
        }
    }

    pub fn load(&mut self) -> Result<(), String> {
        // Load program into memory
        todo!()
    }

    pub fn fetch(&mut self) -> Result<(), String> {
        let first = self
            .mem
            .get(self.pc as usize)
            .ok_or("Failed to fetch first byte.")?;
        let second = self
            .mem
            .get(1 + self.pc as usize)
            .ok_or("Failed to fetch second byte.")?;

        self.opcode = ((first << 8) | second) as u16;
        self.increment_pc();

        Ok(())
    }

    pub fn execute(&mut self) -> Result<(), String> {
        let instruction = (self.opcode & 0xF000);
        match instruction {
            0 => todo!(),
            0x00E0 => self.display.fill(0),           // CLS
            0x1000 => self.pc = self.opcode & 0x0FFF, // JP
            0x6000 => {
                // LD Vx, byte
                let reg = (self.opcode & 0xF000) << 12;
                let value = (self.opcode & 0x00FF) << 8;
                self.registers[reg as usize] = value as u8;
            }
            0xA000 => self.i = self.opcode & 0x0FFF, // LD I, addr
            _ => return Err(format!("Unknow Instruction: {}", instruction)),
        }
        // todo: Handle Timers
        Ok(())
    }

    fn increment_pc(&mut self) {
        self.pc += 2
    }
}

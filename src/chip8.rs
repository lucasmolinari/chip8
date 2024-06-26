use std::{
    fs::{self},
    path::PathBuf,
};

use rand::Rng;

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

const FLAG: usize = 0xF;

pub struct Chip8 {
    opcode: u16, // Operation Code
    i: u16,      // Index Register
    pc: u16,     // Program Counter
    dt: u8,      // Delay Timer
    st: u8,      // Sound Timer
    sp: u16,     // Stack Pointer
    mem: [u8; 4096],
    registers: [u8; 16],
    stack: [u16; 16],
    keys: [u8; 16],
    pub display: [u8; 64 * 32],
}
impl Chip8 {
    pub fn new() -> Self {
        let mut chip = Chip8 {
            opcode: 0,
            i: 0,
            pc: 0x200,
            dt: 0,
            st: 0,
            sp: 0,
            mem: [0; 4096],
            registers: [0; 16],
            stack: [0; 16],
            keys: [0; 16],
            display: [0; 64 * 32],
        };
        chip.mem[..80].copy_from_slice(&FONTSET);

        chip
    }
    pub fn tick(&mut self) -> Result<(), String> {
        self.fetch()?;
        self.execute()?;
        Ok(())
    }

    pub fn keypress(&mut self, i: usize, pressed: u8) {
        self.keys[i] = pressed;
    }

    pub fn load(&mut self, path: PathBuf) -> Result<(), String> {
        let data = fs::read(path).map_err(|e| e.to_string())?;
        self.mem[0x200..0x200 + data.len()].copy_from_slice(&data);

        Ok(())
    }

    pub fn fetch(&mut self) -> Result<(), String> {
        let first = *self
            .mem
            .get(self.pc as usize)
            .ok_or("Failed to fetch first byte.")? as u16;
        let second = *self
            .mem
            .get(1 + self.pc as usize)
            .ok_or("Failed to fetch second byte.")? as u16;

        self.opcode = (first << 8) | second;
        self.increment_pc();

        Ok(())
    }

    pub fn execute(&mut self) -> Result<(), String> {
        let instruction = self.opcode & 0xF000;
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;
        match instruction {
            0x0000 => {
                match self.opcode & 0x000F {
                    // CLS
                    0x0 => self.display.fill(0),
                    // RET
                    0xE => {
                        self.pc = self.stack[self.sp as usize];
                        self.sp -= 1;
                    }
                    _ => return Err(format!("Unknow Instruction: {:X}", self.opcode)),
                }
            }
            // JMP addr
            0x1000 => self.pc = self.opcode & 0x0FFF,
            // CALL addr
            0x2000 => {
                self.sp += 1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = self.opcode & 0x0FFF;
            }
            // SE Vx, byte
            0x3000 => {
                let byte = (self.opcode & 0x00FF) as u8;
                if self.registers[vx] == byte {
                    self.increment_pc();
                }
            }
            // SNE Vx, byte
            0x4000 => {
                let byte = (self.opcode & 0x00FF) as u8;
                if self.registers[vx] != byte {
                    self.increment_pc();
                }
            }
            // SE Vx, Vy
            0x5000 => {
                if self.registers[vx] == self.registers[vy] {
                    self.increment_pc();
                }
            }
            // LD Vx, byte
            0x6000 => {
                let byte = self.opcode & 0x00FF;
                self.registers[vx] = byte as u8;
            }
            // ADD Vx, byte
            0x7000 => {
                let val = (self.opcode & 0x00FF) as u8;
                self.registers[vx] = self.registers[vx].wrapping_add(val);
            }
            0x8000 => self.handle_8xy()?,
            // SNE Vx, Vy
            0x9000 => {
                if self.registers[vx] != self.registers[vy] {
                    self.increment_pc();
                }
            }
            // LD I, addr
            0xA000 => self.i = self.opcode & 0x0FFF,
            // JMP V0, addr
            0xB000 => self.pc = self.registers[0] as u16 + (self.opcode & 0x0FFF),
            // RND Vx, byte
            0xC000 => {
                let rnd = rand::thread_rng().gen_range(0..256) as u8;
                self.registers[vx] = rnd & (self.opcode & 0x00FF) as u8;
            }
            // DRW Vx, Vy, nibble
            0xD000 => self.display(),
            0xE000 => match self.opcode & 0x00FF {
                // SKP Vx
                0x9E => {
                    let x = self.registers[vx];
                    if self.keys[x as usize] == 1 {
                        self.increment_pc()
                    }
                }
                // SKNP Vx
                0xA1 => {
                    let x = self.registers[vx];
                    if self.keys[x as usize] == 0 {
                        self.increment_pc()
                    }
                }
                _ => return Err(format!("Unknow Instruction: {:X}", self.opcode)),
            },
            0xF000 => self.handle_fx()?,
            _ => return Err(format!("Unknow Instruction:  {:X}", self.opcode)),
        }
        self.timers();
        Ok(())
    }

    fn handle_8xy(&mut self) -> Result<(), String> {
        let instruction = self.opcode & 0x000F;
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;
        let x = self.registers[vx as usize];
        let y = self.registers[vy as usize];
        match instruction {
            // LD Vx Vy
            0x0 => {
                self.registers[vx] = self.registers[vy];
            }
            // OR Vx Vy
            0x1 => {
                self.registers[vx] |= self.registers[vy];
                self.registers[FLAG] = 0;
            }
            // AND Vx Vy
            0x2 => {
                self.registers[vx] &= self.registers[vy];
                self.registers[FLAG] = 0;
            }
            // XOR Vx Vy
            0x3 => {
                self.registers[vx] ^= self.registers[vy];
                self.registers[FLAG] = 0;
            }
            // ADD Vx Vy
            0x4 => {
                let (r, c) = x.overflowing_add(y);
                self.registers[vx] = r;
                self.registers[FLAG] = c as u8;
            }
            // SUB Vx, Vy
            0x5 => {
                self.registers[vx] = x.wrapping_sub(y);
                self.registers[FLAG] = (x >= y) as u8;
            }
            // SHR Vx {, Vy}
            0x6 => {
                self.registers[vx] >>= 1;
                self.registers[FLAG] = x & 1; // lsb
            }
            // SUBN Vx, Vy
            0x7 => {
                self.registers[vx] = y.wrapping_sub(x);
                self.registers[FLAG] = (y >= x) as u8;
            }
            // SHL Vx {, Vy}
            0xE => {
                self.registers[vx] <<= 1;
                self.registers[FLAG] = (x >> 7) & 1; // msb
            }
            _ => return Err(format!("Unknow Instruction: {:X}", self.opcode)),
        }
        Ok(())
    }

    fn handle_fx(&mut self) -> Result<(), String> {
        let instruction = self.opcode & 0x00FF;
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        match instruction {
            // LD Vx, DT
            0x07 => self.registers[vx] = self.dt,
            // LD Vx, K
            0x0A => {
                let mut pressed = false;
                for key in self.keys {
                    if key != 0 {
                        self.registers[vx] = key;
                        pressed = true;
                        break;
                    }
                }
                if !pressed {
                    self.decrement_pc();
                }
            }

            // LD DT, Vx
            0x15 => self.dt = self.registers[vx],
            // LD ST, Vx
            0x18 => self.st = self.registers[vx],
            // ADD I, Vx
            0x1E => self.i += self.registers[vx] as u16,
            // LD F, Vx
            0x29 => self.i = self.registers[vx] as u16 * 5,
            // LD B, Vx
            0x33 => {
                self.mem[self.i as usize] = self.registers[vx] / 100;
                self.mem[1 + self.i as usize] = (self.registers[vx] / 10) % 10;
                self.mem[2 + self.i as usize] = self.registers[vx] % 10;
            }
            // LD [I], Vx
            0x55 => {
                for i in 0..=vx {
                    self.mem[i + self.i as usize] = self.registers[i];
                }
                // self.i += vx as u16 + 1; TODO: Toggle legacy mode
            }
            // LD Vx, [I]
            0x65 => {
                for i in 0..=vx {
                    self.registers[i] = self.mem[i + self.i as usize];
                }
                // self.i += vx as u16 + 1; TODO: Toggle legacy mode
            }
            _ => return Err(format!("Unknow Instruction: {:X}", self.opcode)),
        };
        Ok(())
    }

    fn display(&mut self) {
        self.registers[FLAG] = 0;
        let vx = (self.opcode & 0x0F00) >> 8; // Vx Adress
        let vy = (self.opcode & 0x00F0) >> 4; // Vy Adress
        let n = self.opcode & 0x000F; // Height

        let x = self.registers[vx as usize];
        let y = self.registers[vy as usize];

        for yline in 0..n {
            let pixel = self.mem[(self.i + yline) as usize];
            for xline in 0..8 {
                let msb = 0x80;

                if (pixel & (msb >> xline)) != 0 {
                    let ax = (x + xline) % 64;
                    let ay = (y as u16 + yline) % 32;
                    let i = (ax as u16 + ay * 64) as usize;

                    self.display[i] ^= 1;
                    if self.display[i] == 0 {
                        self.registers[0xF] = 1;
                    }
                }
            }
        }
    }

    fn timers(&mut self) {
        if self.dt != 0 {
            self.dt -= 1;
        }
        if self.st != 0 {
            if self.st == 1 {
                println!("Beep");
            }
            self.st -= 1;
        }
    }

    fn increment_pc(&mut self) {
        self.pc += 2
    }

    fn decrement_pc(&mut self) {
        self.pc -= 2
    }
}

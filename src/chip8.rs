#![allow(unused)]

use std::{
    fs::{self, File},
    path::PathBuf,
};

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

pub struct Chip8 {
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
        let mut chip = Chip8 {
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
        };
        chip.mem[..80].copy_from_slice(&FONTSET);

        chip
    }

    pub fn get_display(&self) -> &[u8; 64 * 32] {
        &self.display
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
        let instruction = (self.opcode & 0xF000);
        match instruction {
            0x0000 => {
                match self.opcode & 0x000F {
                    0x0000 => self.display.fill(0), // CLS
                    0x000E => todo!(),              // RET
                    _ => return Err(format!("Unknow Instruction: {}", instruction)),
                }
            }
            0x1000 => self.pc = self.opcode & 0x0FFF, // JP
            0x6000 => {
                let reg = (self.opcode & 0x0F00) >> 8; // Vx
                let val = self.opcode & 0x00FF; // byte
                self.registers[reg as usize] = val as u8; // LD Vx
            }
            0x7000 => {
                let reg = (self.opcode & 0x0F00) >> 8; // Vx
                let val = self.opcode & 0x00FF; // byte
                self.registers[reg as usize] += val as u8; // ADD Vx
            }
            0xD000 => self.display(),                // Dxyn
            0xA000 => self.i = self.opcode & 0x0FFF, // LD I, addr
            _ => return Err(format!("Unknow Instruction: {}", instruction)),
        }
        self.tick();
        Ok(())
    }

    fn display(&mut self) {
        self.registers[0xF] = 0; // VF
        let xx = (self.opcode & 0x0F00) >> 8; // Vx Adress
        let yy = (self.opcode & 0x00F0) >> 4; // Vy Adress
        let n = self.opcode & 0x000F; // Height

        let x = self.registers[xx as usize];
        let y = self.registers[yy as usize];

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

    fn tick(&mut self) {
        if self.dt != 0 {
            self.dt -= 1;
        }
        if self.st != 0 {
            self.st -= 1;
            println!("Beep");
        }
    }

    fn increment_pc(&mut self) {
        self.pc += 2
    }
}

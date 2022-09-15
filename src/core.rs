use crate::instruction::{Instruction, Instruction::*, self};
use rand::Rng;

const HIGH_MASK: u8 = 0xF0;
const LOW_MASK: u8 = 0x0F;
pub const DISPLAY_HEIGHT: usize = 32;
pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_BUFFER: usize = DISPLAY_HEIGHT * DISPLAY_WIDTH;

pub struct CPU {
    memory: [u8; 4096],
    pc: usize,
    stack: [usize; 48],
    sp: usize,
    index_register: u16,
    pub delay_timer: u8,
    pub sound_timer: u8,
    general_registers: [u8; 16],
    pub display: [u8; DISPLAY_BUFFER],
    pub keys: [u8; 16]
}

impl CPU {
    pub fn new() -> Self {
        // Load interpreter

        // Load font
        let mut c = CPU {
            memory: [0; 4096],
            pc: 0, 
            stack: [0; 48],
            sp: 0,
            index_register: 0,
            delay_timer: 0,
            sound_timer: 0,
            general_registers: [0; 16],
            display: [0; DISPLAY_BUFFER],
            keys: [0; 16]
        };

        // set font
        c.set_font();
        c
    }

    fn set_font(&mut self) {
        self.memory[080..085].copy_from_slice(&[0xF0, 0x90, 0x90, 0x90, 0xF0]); // 0
        self.memory[085..090].copy_from_slice(&[0x20, 0x60, 0x20, 0x20, 0x70]); // 1
        self.memory[090..095].copy_from_slice(&[0xF0, 0x10, 0xF0, 0x80, 0xF0]); // 2
        self.memory[095..100].copy_from_slice(&[0xF0, 0x10, 0xF0, 0x10, 0xF0]); // 3
        self.memory[100..105].copy_from_slice(&[0x90, 0x90, 0xF0, 0x10, 0x10]); // 4
        self.memory[105..110].copy_from_slice(&[0xF0, 0x80, 0xF0, 0x10, 0xF0]); // 5
        self.memory[110..115].copy_from_slice(&[0xF0, 0x80, 0xF0, 0x90, 0xF0]); // 6
        self.memory[115..120].copy_from_slice(&[0xF0, 0x10, 0x20, 0x40, 0x40]); // 7
        self.memory[120..125].copy_from_slice(&[0xF0, 0x90, 0xF0, 0x90, 0xF0]); // 8
        self.memory[125..130].copy_from_slice(&[0xF0, 0x90, 0xF0, 0x10, 0xF0]); // 9
        self.memory[130..135].copy_from_slice(&[0xF0, 0x90, 0xF0, 0x90, 0x90]); // A
        self.memory[135..140].copy_from_slice(&[0xE0, 0x90, 0xE0, 0x90, 0xE0]); // B
        self.memory[140..145].copy_from_slice(&[0xF0, 0x80, 0x80, 0x80, 0xF0]); // C
        self.memory[145..150].copy_from_slice(&[0xE0, 0x90, 0x90, 0x90, 0xE0]); // D
        self.memory[150..155].copy_from_slice(&[0xF0, 0x80, 0xF0, 0x80, 0xF0]); // E
        self.memory[155..160].copy_from_slice(&[0xF0, 0x80, 0xF0, 0x80, 0x80]); // F
    }

    fn address_for_font(&self, char: u8) -> usize {
        0x50 + char as usize * 5
    }

    pub fn load(&mut self, prog: Vec<u8>) {
        self.memory[0x200..0x200+prog.len()].copy_from_slice(&prog.as_slice());
        println!("Loaded {} bytes into memory", prog.len());
        self.pc = 0x200;
        println!("PC set to 0x200");
    }

    pub fn step(&mut self) {
        let raw = self.fetch();
        let instruction = self.decode(raw);
        self.execute(instruction);
    }

    fn fetch(&mut self) -> [u8; 2] {
        let raw = [
            self.memory[self.pc],
            self.memory[self.pc + 1]
        ];
        self.pc += 2;
        raw
    }

    fn decode(&self, raw: [u8; 2]) -> Instruction {
        let opcode = (raw[0] & HIGH_MASK) >> 4;
        let register_a = raw[0] & LOW_MASK;
        let register_b = (raw[1] & HIGH_MASK) >> 4;
        let N = raw[1] & LOW_MASK;
        let NN = raw[1];
        let NNN = (((raw[0] & LOW_MASK) as u16) << 8) | raw[1] as u16;

        match opcode {
            0x00 => {
                match raw[1] {
                    0xE0 => {
                        ClearScreen
                    },
                    0xEE => {
                        Return
                    },
                    0x00 => {
                        NOP
                    },
                    _ => {
                        Data(raw[0],raw[1])
                    }
                }
            },
            0x01 => {
                Jump(NNN)
            },
            0x02 => {
                Call(NNN)
            },
            // Skips
            0x03 => {
                SkipIEQ(register_a, NN)
            },
            0x04 => {
                SkipINEQ(register_a, NN)
            },
            0x05 => {
                SkipREQ(register_a, register_b)
            },
            0x09 => {
                SkipRNEQ(register_a, register_b)
            },
            0x06 => {
                SetRI(register_a, NN)
            },
            0x07 => {
                AddRI(register_a, NN)
            },
            0x08 => {
                match N {
                    0x00 => {
                        SetRR(register_a, register_b)
                    },
                    0x01 => {
                        OrRR(register_a, register_b)
                    },
                    0x02 => {
                        AndRR(register_a, register_b)
                    },
                    0x03 => {
                        XorRR(register_a, register_b)
                    },
                    0x04 => {
                        AddRR(register_a, register_b)
                    },
                    0x05 => {
                        SubAB(register_a, register_b)
                    },
                    0x06 => {
                        ShiftRightRR(register_a, register_b)
                    },
                    0x07 => {
                        SubBA(register_a, register_b)
                    },
                    0x0E => {
                        ShiftLeftRR(register_a, register_b)
                    },
                    _ => {
                        Data(raw[0], raw[1])
                    }
                }

            }
            0x0A => {
                SetX(NNN)
            },
            0x0B => {
                JumpOffset(NNN)
            }
            0x0C => {
                Random(register_a, NN)
            },
            0x0D => {
                Draw(register_a, register_b, N)
            },
            0x0E => {
                match NN {
                    0x9E => {
                        SkipKeyEQ(register_a)
                    },
                    0xA1 => {
                        SkipKeyNEQ(register_a)
                    },
                    _ => {
                        Data(raw[0], raw[1])
                    }
                }
            },
            0x0F => {
                match NN {
                    0x07 => {
                        SetRDelay(register_a)
                    },
                    0x15 => {
                        SetDelayR(register_a)
                    },
                    0x18 => {
                        SetSoundR(register_a)
                    },
                    0x1E => {
                        AddXR(register_a)
                    },
                    0x0A => {
                        GetKey(register_a)
                    },
                    0x29 => {
                        SetXFontR(register_a)
                    },
                    0x33 => {
                        StoreDecimalR(register_a)
                    },
                    0x55 => {
                        Store(register_a)
                    },
                    0x65 => {
                        Load(register_b)
                    }
                    _ => {
                        Data(raw[0], raw[1])
                    }
                }
            }
            _ => {
                Data(raw[0], raw[1])
            }
        }
    }

    fn execute(&mut self, instruction: Instruction) {

        //println!("{:?}", instruction);
        match instruction {
            NOP => {

            },
            ClearScreen => {
                for i in 0..DISPLAY_BUFFER {
                    self.display[i] = 0;
                }
            },
            Jump(a) => {
                self.pc = a as usize;
            },
            SetRI(r, n) => {
                self.general_registers[r as usize] = n;
            },
            AddRI(r, n) => {
                let mut t = self.general_registers[r as usize] as u16 + n as u16;
                if t > 0xFF {
                    t -= 0x100;
                }
                self.general_registers[r as usize] = t as u8;
            },
            SetX(n) => {
                self.index_register = n;
            },
            Draw(a, b, n) => {
                self.draw(a, b, n);
            },
            Call(n) => {
                self.stack[self.sp] = self.pc;
                self.sp += 1;
                self.pc = n as usize;
            },
            Return => {
                self.sp -= 1;
                self.pc = self.stack[self.sp];
            },
            SkipIEQ(r, n) => {
                if self.general_registers[r as usize] == n {
                    self.pc += 2;
                }
            },
            SkipINEQ(r, n) => {
                if self.general_registers[r as usize] != n {
                    self.pc += 2;
                } 
            },
            SkipREQ(a, b) => {
                if self.general_registers[a as usize] == self.general_registers[b as usize] {
                    self.pc += 2;
                }
            },
            SkipRNEQ(a, b) => {
                if self.general_registers[a as usize] != self.general_registers[b as usize] {
                    self.pc += 2;
                }
            },
            SetRR(a, b) => {
                self.general_registers[a as usize] = self.general_registers[b as usize];
            },
            OrRR(a, b) => {
                self.general_registers[a as usize] = self.general_registers[a as usize] | self.general_registers[b as usize];
            },
            AndRR(a, b) => {
                self.general_registers[a as usize] = self.general_registers[a as usize] & self.general_registers[b as usize];
            },
            XorRR(a, b) => {
                self.general_registers[a as usize] = self.general_registers[a as usize] ^ self.general_registers[b as usize];
            },
            AddRR(a, b) => {
                let mut t = self.general_registers[a as usize] as u16 + self.general_registers[b as usize] as u16;
                if t > 0xFF {
                    self.general_registers[0xF] = 1;
                    t = t - 0x100;
                }
                self.general_registers[a as usize] = t as u8;
            },
            SubAB(a, b) => {
                let r = a;
                let mut a = self.general_registers[a as usize] as i16;
                let b = self.general_registers[b as usize] as i16;
                if a > b {
                    self.general_registers[0xF] = 1;
                } else {
                    self.general_registers[0xF] = 0;
                    a += 256;
                }
                self.general_registers[r as usize] = (a - b) as u8;
            },
            SubBA(a, b) => {
                let r = a;
                let a = self.general_registers[a as usize] as i16;
                let mut b = self.general_registers[b as usize] as i16;
                if b > a {
                    self.general_registers[0xF] = 1;
                } else {
                    self.general_registers[0xF] = 0;
                    b += 256;
                }
                self.general_registers[r as usize] = (b - a) as u8;
            },
            ShiftRightRR(a, _) => {
                let outbit = self.general_registers[a as usize] & 0x01;
                self.general_registers[0xF] = if outbit > 0 {1} else {0};
                self.general_registers[a as usize] = self.general_registers[a as usize] >> 1;
            },
            ShiftLeftRR(a, _) => {
                let outbit = self.general_registers[a as usize] & 0x80;
                self.general_registers[0xF] = if outbit > 0 {1} else {0};
                self.general_registers[a as usize] = self.general_registers[a as usize] << 1;
            },
            JumpOffset(offset) => {
                self.pc = self.general_registers[0] as usize + offset as usize;
            },
            Random(register_a, n) => {
                let k =  rand::thread_rng().gen_range(0..0xFF) & n;
                self.general_registers[register_a as usize] = k;
            },
            SkipKeyEQ(a) => {
                let v = self.general_registers[a as usize] & 0xF;
                if self.keys[v as usize] > 0 {
                    self.pc += 2;
                }
            },
            SkipKeyNEQ(a) => {
                let v = self.general_registers[a as usize] & 0xF;
                if self.keys[v as usize] == 0 {
                    self.pc += 2;
                }
            },
            SetRDelay(a) => {
                self.general_registers[a as usize] = self.delay_timer;
            },
            SetDelayR(a) => {
                self.delay_timer = self.general_registers[a as usize];
            },
            SetSoundR(a) => {
                self.sound_timer = self.general_registers[a as usize];
            },
            AddXR(a) => {
                self.index_register += self.general_registers[a as usize] as u16;
                if self.index_register > 0xFFF {
                    self.general_registers[0xF] = 1;
                }
            },
            SetXFontR(a) => {
                let hex = self.general_registers[a as usize] & 0xF;
                self.index_register = self.address_for_font(hex) as u16;
                
            },
            StoreDecimalR(a) => {
                let v = self.general_registers[a as usize];
                self.memory[self.index_register as usize] = v / 100;
                self.memory[(self.index_register + 1) as usize] = (v / 10) % 10;
                self.memory[(self.index_register + 2) as usize] = v % 10;
            }
            GetKey(a) => {
                let mut hex = 0;
                for key in self.keys {
                    if key > 0 {
                        self.general_registers[a as usize] = hex;
                        return;
                    }
                    hex += 1;
                }
                // decrement pc if we haven't encountered a key press
                self.pc -=2;
            },
            Store(a) => {
                for r in 0..a+1 {
                   self.memory[(self.index_register + r as u16) as usize] = self.general_registers[r as usize]; 
                }
            },
            Load(a) => {
                for r in 0..a+1 {
                    self.general_registers[r as usize] = self.memory[(self.index_register + r as u16) as usize];
                }
            }
            Data(a, b) => {
                panic!("Should not be executing this! Line: {} {} {}", self.pc, a, b)
            },
        }
    }


    fn draw(&mut self, a: u8, b: u8, n: u8) {
        let x = self.general_registers[a as usize] as usize % DISPLAY_WIDTH;
        let mut y = self.general_registers[b as usize] as usize % DISPLAY_HEIGHT;
        self.general_registers[0xF] = 0;
        for row in 0..n as usize {
            let mut mx = x;
            let sprite_row = self.memory[self.index_register as usize + row];
            for i in 0..8 {
                let bit = sprite_row & (1 << (7 - i));
                if bit > 0 {
                    let c = y*DISPLAY_WIDTH + mx;
                    if self.display[c] == 1 {
                        self.general_registers[0xF] = 1;
                        self.display[c] = 0;
                    } else {
                        self.display[c] = 1;
                    }
                }
                if mx == DISPLAY_WIDTH - 1 {
                    break
                }
                mx += 1;
            }
            if y == DISPLAY_HEIGHT -1 {
                break
            }
            y += 1;
        }
    }

    pub fn dump_current(&self) {
        let instr = self.decode(
            [
                self.memory[self.pc],
                self.memory[self.pc + 1]
            ]
        );
        println!("{:#08x}:\t{:?}", self.pc, instr);
    }

    pub fn dump_registers(&self) {
        for r in 0..16 {
            println!("{}: {}", r, self.general_registers[r]);
        }
        println!("i: {}", self.index_register);
    }
    
    pub fn dump_memory_instr(&self) {
        for i in 0..4096/2 {
            let instr = self.decode([
                self.memory[i*2],
                self.memory[i*2+1]
            ]);
            match instr {
                NOP => {},
                _ => {println!("{:#08x}:\t{:?}", i*2, instr)}
            } 
        }
    }
}
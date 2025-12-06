use std::convert::TryInto;

#[derive(Default)]
struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,

    la: u32,
    lb: u32,
    lc: u32,
    ld: u32,
    ptr: u32,
    pc: u32,
}

type Memory = Vec<u8>;
type Output = Vec<u8>;

struct CPU {
    registers: Registers,
    memory: Memory,
    output: Output,
}

impl CPU {
    fn new(program: Vec<u8>) -> Self {
        Self {
            registers: Default::default(),
            memory: program,
            output: Default::default(),
        }
    }

    fn execute(&mut self) {
        loop {
            match self.opcode() {
                0x01 => break,
                0x02 => self.out(),
                0x21 => self.jez(),
                0x22 => self.jnz(),
                0xc1 => self.cmp(),
                0xc2 => self.add(),
                0xc3 => self.sub(),
                0xc4 => self.xor(),
                0xe1 => self.aptr(),
                0b01_000_000..=0b01_111_111 => self.mv(),
                0b10_000_000..=0b10_111_111 => self.mv32(),
                otherwise => unreachable!("unknown opcode {:#x}", otherwise),
            }
        }
    }

    /// --[ ADD a <- b ]--------------------------------------------
    ///
    ///   8-bit addition
    ///   Opcode: 0xC2 (1 byte)
    ///
    ///   Sets `a` to the sum of `a` and `b`, modulo 256.
    fn add(&mut self) {
        self.registers.a = self.registers.a.wrapping_add(self.registers.b);
        self.registers.pc += 1;
    }

    /// --[ APTR imm8 ]---------------------------------------------
    ///
    ///   Advance ptr
    ///   Opcode: 0xE1 0x__ (2 bytes)
    ///
    ///   Sets `ptr` to the sum of `ptr` and `imm8`. Overflow
    ///   behaviour is undefined.
    fn aptr(&mut self) {
        self.registers.ptr += u32::from(self.imm8());
        self.registers.pc += 2;
    }

    /// --[ CMP ]---------------------------------------------------
    ///
    ///   Compare
    ///   Opcode: 0xC1 (1 byte)
    ///
    ///   Sets `f` to zero if `a` and `b` are equal, otherwise sets
    ///   `f` to 0x01.
    fn cmp(&mut self) {
        self.registers.f = if self.registers.a == self.registers.b {
            0
        } else {
            1
        };
        self.registers.pc += 1;
    }

    /// --[ JEZ imm32 ]---------------------------------------------
    ///
    ///   Jump if equals zero
    ///   Opcode: 0x21 0x__ 0x__ 0x__ 0x__ (5 bytes)
    ///
    ///   If `f` is equal to zero, sets `pc` to `imm32`. Otherwise
    ///   does nothing.
    fn jez(&mut self) {
        if self.registers.f == 0 {
            self.registers.pc = self.imm32();
        } else {
            self.registers.pc += 5;
        }
    }

    /// --[ JNZ imm32 ]---------------------------------------------
    ///
    ///   Jump if not zero
    ///   Opcode: 0x22 0x__ 0x__ 0x__ 0x__ (5 bytes)
    ///
    ///   If `f` is not equal to zero, sets `pc` to `imm32`.
    ///   Otherwise does nothing.
    fn jnz(&mut self) {
        if self.registers.f != 0 {
            self.registers.pc = self.imm32();
        } else {
            self.registers.pc += 5;
        }
    }

    /// --[ MV {dest} <- {src} ]------------------------------------
    ///
    ///   Move 8-bit value
    ///   Opcode: 0b01DDDSSS (1 byte)
    ///
    ///   Sets `{dest}` to the value of `{src}`.
    ///
    ///   Both `{dest}` and `{src}` are 3-bit unsigned integers that
    ///   correspond to an 8-bit register or pseudo-register. In the
    ///   opcode format above, the "DDD" bits are `{dest}`, and the
    ///   "SSS" bits are `{src}`. Below are the possible valid
    ///   values (in decimal) and their meaning.
    ///
    ///                           1 => `a`
    ///                           2 => `b`
    ///                           3 => `c`
    ///                           4 => `d`
    ///                           5 => `e`
    ///                           6 => `f`
    ///                           7 => `(ptr+c)`
    ///
    ///   A zero `{src}` indicates an MVI instruction, not MV.
    ///
    /// --[ MVI {dest} <- imm8 ]------------------------------------
    ///
    ///   Move immediate 8-bit value
    ///   Opcode: 0b01DDD000 0x__ (2 bytes)
    ///
    ///   Sets `{dest}` to the value of `imm8`.
    ///
    ///   `{dest}` is a 3-bit unsigned integer that corresponds to
    ///   an 8-bit register or pseudo-register. It is the "DDD" bits
    ///   in the opcode format above. Below are the possible valid
    ///   values (in decimal) and their meaning.
    ///
    ///                           1 => `a`
    ///                           2 => `b`
    ///                           3 => `c`
    ///                           4 => `d`
    ///                           5 => `e`
    ///                           6 => `f`
    ///                           7 => `(ptr+c)`
    fn mv(&mut self) {
        let opcode = self.opcode();
        let src = opcode & 0b111;
        let dest = (opcode >> 3) & 0b111;
        let src_val = self.load_8bit(src);
        self.store_8bit(dest, src_val);

        self.registers.pc += 1;
        if src == 0 {
            self.registers.pc += 1;
        }
    }

    /// --[ MV32 {dest} <- {src} ]----------------------------------
    ///
    ///   Move 32-bit value
    ///   Opcode: 0b10DDDSSS (1 byte)
    ///
    ///   Sets `{dest}` to the value of `{src}`.
    ///
    ///   Both `{dest}` and `{src}` are 3-bit unsigned integers that
    ///   correspond to a 32-bit register. In the opcode format
    ///   above, the "DDD" bits are `{dest}`, and the "SSS" bits are
    ///   `{src}`. Below are the possible valid values (in decimal)
    ///   and their meaning.
    ///
    ///                           1 => `la`
    ///                           2 => `lb`
    ///                           3 => `lc`
    ///                           4 => `ld`
    ///                           5 => `ptr`
    ///                           6 => `pc`
    ///
    /// --[ MVI32 {dest} <- imm32 ]---------------------------------
    ///
    ///   Move immediate 32-bit value
    ///   Opcode: 0b10DDD000 0x__ 0x__ 0x__ 0x__ (5 bytes)
    ///
    ///   Sets `{dest}` to the value of `imm32`.
    ///
    ///   `{dest}` is a 3-bit unsigned integer that corresponds to a
    ///   32-bit register. It is the "DDD" bits in the opcode format
    ///   above. Below are the possible valid values (in decimal)
    ///   and their meaning.
    ///
    ///                           1 => `la`
    ///                           2 => `lb`
    ///                           3 => `lc`
    ///                           4 => `ld`
    ///                           5 => `ptr`
    ///                           6 => `pc`
    fn mv32(&mut self) {
        let opcode = self.opcode();
        let src = opcode & 0b111;
        let dest = (opcode >> 3) & 0b111;
        // NB: `load_32bit` can access `pc` and the spec says that it is incremented _before_
        // execution, so we must increment here to ensure correctness.
        self.registers.pc += 1;
        if src == 0 {
            self.registers.pc += 4;
        }
        let src_val = self.load_32bit(src);
        self.store_32bit(dest, src_val);
    }

    /// --[ OUT a ]-------------------------------------------------
    ///
    ///   Output byte
    ///   Opcode: 0x02 (1 byte)
    ///
    ///   Appends the value of `a` to the output stream.
    fn out(&mut self) {
        self.output.push(self.registers.a);
        self.registers.pc += 1;
    }

    /// --[ SUB a <- b ]--------------------------------------------
    ///
    ///   8-bit subtraction
    ///   Opcode: 0xC3 (1 byte)
    ///
    ///   Sets `a` to the result of subtracting `b` from `a`. If
    ///   subtraction would result in a negative number, 256 is
    ///   added to ensure that the result is non-negative.
    fn sub(&mut self) {
        self.registers.a = self.registers.a.wrapping_sub(self.registers.b);
        self.registers.pc += 1;
    }

    /// --[ XOR a <- b ]--------------------------------------------
    ///
    ///   8-bit bitwise exclusive OR
    ///   Opcode: 0xC4 (1 byte)
    ///
    ///   Sets `a` to the bitwise exclusive OR of `a` and `b`.
    fn xor(&mut self) {
        self.registers.a ^= self.registers.b;
        self.registers.pc += 1;
    }

    /// Fetch the current opcode
    fn opcode(&self) -> u8 {
        self.memory[self.registers.pc as usize]
    }

    /// Fetch an imm8 value
    fn imm8(&self) -> u8 {
        self.memory[self.registers.pc as usize + 1]
    }

    fn imm32(&self) -> u32 {
        self.imm32_offset(0)
    }

    /// Fetch an imm32 value
    fn imm32_offset(&self, offset: usize) -> u32 {
        u32::from_le_bytes(
            self.memory
                [self.registers.pc as usize + 1 - offset..self.registers.pc as usize + 5 - offset]
                .try_into()
                .unwrap(),
        )
    }

    fn load_8bit(&self, src: u8) -> u8 {
        match src {
            0 => self.imm8(),
            1 => self.registers.a,
            2 => self.registers.b,
            3 => self.registers.c,
            4 => self.registers.d,
            5 => self.registers.e,
            6 => self.registers.f,
            7 => self.memory[(self.registers.ptr + u32::from(self.registers.c)) as usize],
            _ => unreachable!(),
        }
    }

    fn store_8bit(&mut self, dest: u8, val: u8) {
        let dest_ref = match dest {
            1 => &mut self.registers.a,
            2 => &mut self.registers.b,
            3 => &mut self.registers.c,
            4 => &mut self.registers.d,
            5 => &mut self.registers.e,
            6 => &mut self.registers.f,
            7 => &mut self.memory[(self.registers.ptr + u32::from(self.registers.c)) as usize],
            _ => unreachable!(),
        };
        *dest_ref = val;
    }

    fn load_32bit(&self, src: u8) -> u32 {
        match src {
            0 => self.imm32_offset(5),
            1 => self.registers.la,
            2 => self.registers.lb,
            3 => self.registers.lc,
            4 => self.registers.ld,
            5 => self.registers.ptr,
            6 => self.registers.pc,
            _ => unreachable!(),
        }
    }

    fn store_32bit(&mut self, dest: u8, val: u32) {
        let dest_ref = match dest {
            1 => &mut self.registers.la,
            2 => &mut self.registers.lb,
            3 => &mut self.registers.lc,
            4 => &mut self.registers.ld,
            5 => &mut self.registers.ptr,
            6 => &mut self.registers.pc,
            _ => unreachable!(),
        };
        *dest_ref = val;
    }
}

pub fn solve(payload: Vec<u8>) -> Vec<u8> {
    let mut cpu = CPU::new(payload);
    cpu.execute();
    cpu.output
}

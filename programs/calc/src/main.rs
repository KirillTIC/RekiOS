#![no_std]
#![no_main]

static mut OUTPUT_BUF: [u8; 64] = [0; 64];

struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self { input, pos: 0 }
    }

    fn peek(&self) -> Option<u8> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn skip_spaces(&mut self) {
        while let Some(b' ') = self.peek() {
            self.advance();
        }
    }

    fn parse_number(&mut self) -> Option<i64> {
        self.skip_spaces();
        let neg = if self.peek() == Some(b'-') {
            self.advance();
            true
        } else {
            false
        };
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }
        if self.pos == start {
            return None;
        }
        let mut val: i64 = 0;
        for &b in &self.input[start..self.pos] {
            val = val * 10 + (b - b'0') as i64;
        }
        if neg {
            val = -val;
        }
        Some(val)
    }

    fn parse_factor(&mut self) -> Option<i64> {
        self.skip_spaces();
        if self.peek() == Some(b'(') {
            self.advance();
            let val = self.parse_expr()?;
            self.skip_spaces();
            if self.peek() == Some(b')') {
                self.advance();
            }
            Some(val)
        } else {
            self.parse_number()
        }
    }

    fn parse_term(&mut self) -> Option<i64> {
        let mut val = self.parse_factor()?;
        loop {
            self.skip_spaces();
            match self.peek() {
                Some(b'*') => {
                    self.advance();
                    val *= self.parse_factor()?;
                }
                Some(b'/') => {
                    self.advance();
                    let rhs = self.parse_factor()?;
                    if rhs == 0 {
                        return None;
                    }
                    val /= rhs;
                }
                _ => break,
            }
        }
        Some(val)
    }

    fn parse_expr(&mut self) -> Option<i64> {
        let mut val = self.parse_term()?;
        loop {
            self.skip_spaces();
            match self.peek() {
                Some(b'+') => {
                    self.advance();
                    val += self.parse_term()?;
                }
                Some(b'-') => {
                    self.advance();
                    val -= self.parse_term()?;
                }
                _ => break,
            }
        }
        Some(val)
    }
}

fn itoa(mut val: i64, buf: &mut [u8]) -> usize {
    if val == 0 {
        buf[0] = b'0';
        return 1;
    }
    let neg = val < 0;
    if neg {
        val = -val;
    }
    let mut tmp = [0u8; 20];
    let mut len = 0;
    while val > 0 {
        tmp[len] = b'0' + (val % 10) as u8;
        val /= 10;
        len += 1;
    }
    let mut pos = 0;
    if neg {
        buf[pos] = b'-';
        pos += 1;
    }
    for i in (0..len).rev() {
        buf[pos] = tmp[i];
        pos += 1;
    }
    pos
}

#[unsafe(no_mangle)]
pub extern "C" fn _start(arg_ptr: u64, arg_len: u64) -> ! {
    if arg_len == 0 {
        let msg = b"Usage: calc <expression>\n";
        unsafe {
            core::arch::asm!(
                "mov rax, 0",
                "mov rdi, 1",
                "syscall",
                "mov rax, 1",
                "mov rdi, 1",
                "syscall",
                in("rsi") msg.as_ptr() as u64,
                in("rdx") msg.len() as u64,
                options(noreturn),
            );
        }
    }

    let input = unsafe { core::slice::from_raw_parts(arg_ptr as *const u8, arg_len as usize) };

    let mut parser = Parser::new(input);
    match parser.parse_expr() {
        Some(val) => {
            let buf = unsafe { &mut *(&raw mut OUTPUT_BUF) };
            let len = itoa(val, buf);
            buf[len] = b'\n';
            let ptr = buf.as_ptr() as u64;
            let total_len = (len + 1) as u64;
            unsafe {
                core::arch::asm!(
                    "mov rax, 0",
                    "mov rdi, 1",
                    "syscall",
                    "mov rax, 1",
                    "mov rdi, 0",
                    "syscall",
                    in("rsi") ptr,
                    in("rdx") total_len,
                    options(noreturn),
                );
            }
        }
        None => {
            let msg = b"Error: invalid expression\n";
            unsafe {
                core::arch::asm!(
                    "mov rax, 0",
                    "mov rdi, 1",
                    "syscall",
                    "mov rax, 1",
                    "mov rdi, 1",
                    "syscall",
                    in("rsi") msg.as_ptr() as u64,
                    in("rdx") msg.len() as u64,
                    options(noreturn),
                );
            }
        }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

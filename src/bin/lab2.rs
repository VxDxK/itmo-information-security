use std::process::ExitCode;
use rand::{random};
use std::fs;
use std::path::PathBuf;
use clap::{Parser, ValueEnum};

fn u128_to_u32_be(key: u128) -> [u32; 4] {
    [
        (key >> 96) as u32,
        (key >> 64) as u32,
        (key >> 32) as u32,
        key as u32,
    ]
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    running_mode: Mode,
    input_file: PathBuf,
    tea_key: Option<u128>,
    cbc_iv: Option<u64>,
    output_file: Option<PathBuf>,
}

#[derive(ValueEnum, Debug, Clone)]
enum Mode {
    Encrypt,
    Decrypt,
}

/// Tiny Encryption Algorithm
struct TEA {
    key: [u32; 4],
}

/// Константа выведена из золотого сечения
static DELTA: u32 = 0x9e3779b9;
impl TEA {
    pub fn new(key: u128) -> Self {
        Self { key: u128_to_u32_be(key) }
    }

    fn encrypt_block(&self, block: u64) -> u64 {
        let (mut v0, mut v1) = ((block >> 32) as u32, block as u32);
        let mut sum: u32 = 0;

        // Итерация цикла 32, а не 64 потому что в теле цикла делаем сразу две итерации
        for _ in 0..32 {
            sum = sum.wrapping_add(DELTA);
            // Нечетная итерация
            v0 = v0.wrapping_add(
                ((v1 << 4).wrapping_add(self.key[0]))
                    ^ (v1.wrapping_add(sum))
                    ^ ((v1 >> 5).wrapping_add(self.key[1])),
            );
            // Четная итерация
            v1 = v1.wrapping_add(
                ((v0 << 4).wrapping_add(self.key[2]))
                    ^ (v0.wrapping_add(sum))
                    ^ ((v0 >> 5).wrapping_add(self.key[3])),
            );
        }

        ((v0 as u64) << 32) | (v1 as u64)
    }

    fn decrypt_block(&self, block: u64) -> u64 {
        let (mut v0, mut v1) = ((block >> 32) as u32, block as u32);
        let mut sum: u32 = DELTA.wrapping_mul(32);

        for _ in 0..32 {
            v1 = v1.wrapping_sub(
                ((v0 << 4).wrapping_add(self.key[2]))
                    ^ (v0.wrapping_add(sum))
                    ^ ((v0 >> 5).wrapping_add(self.key[3])),
            );
            v0 = v0.wrapping_sub(
                ((v1 << 4).wrapping_add(self.key[0]))
                    ^ (v1.wrapping_add(sum))
                    ^ ((v1 >> 5).wrapping_add(self.key[1])),
            );
            sum = sum.wrapping_sub(DELTA);
        }

        ((v0 as u64) << 32) | (v1 as u64)
    }
}

/// Cipher Block Chaining
struct CBC {
    iv: u64,
    tea: TEA,
}

impl CBC {
    fn new(iv: u64, tea: TEA) -> Self {
        Self { iv, tea }
    }

    fn encrypt_block(&mut self, block: u64) -> u64 {
        let processed = block ^ self.iv;
        let processed = self.tea.encrypt_block(processed);
        self.iv = processed;
        processed
    }

    fn decrypt_block(&mut self, block: u64) -> u64 {
        let decrypted = self.tea.decrypt_block(block);
        let processed = decrypted ^ self.iv;
        self.iv = block;
        processed
    }

    fn process_slice(&mut self, mode: &Mode, data: &[u8]) -> Vec<u8> {
        let mut res = Vec::with_capacity(data.len());
        for chunk in data.chunks(8) {
            let mut block = [0u8; 8];
            block[..chunk.len()].copy_from_slice(chunk);
            let block = u64::from_le_bytes(block);
            let block_value = match mode {
                Mode::Encrypt => self.encrypt_block(block),
                Mode::Decrypt => self.decrypt_block(block),
            };
            res.extend_from_slice(&block_value.to_ne_bytes());
        }
        res
    }
}

fn main() -> ExitCode {
    let mut args = Args::parse();
    let content = match fs::read(&args.input_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read input file {e}");
            return ExitCode::from(1);
        }
    };
    if let None = args.tea_key {
        args.tea_key = Some(random());
        println!("Key for TEA wasn't specified, generating random: '{}'", args.tea_key.unwrap());
    }

    if let None = args.cbc_iv {
        args.cbc_iv = Some(random());
        println!("Initialization vector for CBC wasn't specified, generating random: '{}'", args.cbc_iv.unwrap());
    }

    let mut cbc = CBC::new(args.cbc_iv.unwrap(), TEA::new(args.tea_key.unwrap()));
    let result = cbc.process_slice(&args.running_mode, &content.as_slice());
    if let Some(output_file) = args.output_file {
        fs::write(output_file, &result).unwrap();
    } else {
        println!("{:?}", result);
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use rand::random;
    use crate::{Mode, CBC, TEA};

    #[test]
    fn encrypt_and_decrypt() {
        let text = b"Hello my friends";
        let key: u128 = random();
        let iv: u64 = random();
        let mut cbc0 = CBC::new(iv, TEA::new(key));
        let mut cbc1 = CBC::new(iv, TEA::new(key));
        let encrypted = cbc0.process_slice(&Mode::Encrypt, text.as_slice());
        let decrypted = cbc1.process_slice(&Mode::Decrypt, encrypted.as_slice());
        assert_eq!(decrypted, text);
    }
}
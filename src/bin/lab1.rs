use clap::{Parser, ValueEnum};
use std::borrow::Borrow;
use std::collections::{BTreeMap, HashSet};
use std::fmt::Write;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::process::ExitCode;

fn check_keyword<T>(s: &T) -> bool
where
    T: Borrow<str>,
{
    let str = s.borrow();
    let mut chars_set = HashSet::new();
    for char in str.chars() {
        if !char.is_alphabetic() || !chars_set.insert(char) {
            return false;
        }
    }
    true
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    running_mode: RunningMode,
    input_file: PathBuf,
    keyword: String,
    output_file: Option<PathBuf>,
}

#[derive(ValueEnum, Debug, Clone)]
enum RunningMode {
    Encrypt,
    Decrypt,
}

#[derive(Debug)]
struct Cipher {
    keyword: Vec<char>,
    position: usize,
    words: BTreeMap<char, Vec<char>>,
}

impl Cipher {
    fn new(keyword: String) -> Self {
        let mut words = BTreeMap::new();
        for char in keyword.chars() {
            words.insert(char, Vec::new());
        }
        Self {
            keyword: keyword.chars().collect(),
            position: 0,
            words,
        }
    }

    fn from_content(content: String, keyword: String) -> Self {
        let keyword: Vec<char> = keyword.chars().collect();
        let mut words = BTreeMap::new();
        let content: Vec<char> = content.chars().collect();
        for char in keyword.iter() {
            words.insert(char.to_owned(), Vec::new());
        }
        let mut in_use = 0;
        let usize_signum = |value: usize| if value == 0 { 0usize } else { 1usize };
        for (i, (_, v)) in words.iter_mut().enumerate() {
            let to_use = (content.len() - in_use) / (keyword.len() - i)
                + usize_signum((content.len() - in_use) % (keyword.len() - i));
            v.extend_from_slice(&content[in_use..in_use + to_use]);
            in_use += to_use;
        }

        Self {
            keyword,
            words,
            position: content.len(),
        }
    }

    fn add_char(&mut self, c: char) {
        let current_char = self.keyword[self.position % self.keyword.len()];
        self.words
            .get_mut(&current_char)
            .expect("Must init map with char in constructor")
            .push(c);
        self.position += 1;
    }

    fn decrypt(self) -> String {
        let iterator = DecryptIterator::new(self);
        let vec: Vec<char> = iterator.collect();
        vec.iter().collect()
    }

    fn encrypt(self) -> String {
        let mut ans = Vec::new();
        for (_, v) in self.words.into_iter() {
            ans.extend_from_slice(v.as_slice());
        }
        ans.iter().collect()
    }
}

impl Write for Cipher {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        for c in s.chars() {
            self.add_char(c);
        }
        Ok(())
    }
}

struct DecryptIterator {
    encrypt: Cipher,
    position: usize,
}

impl DecryptIterator {
    pub fn new(encrypt: Cipher) -> Self {
        Self {
            encrypt,
            position: 0,
        }
    }
}

impl Iterator for DecryptIterator {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let current_char = self.encrypt.keyword[self.position % self.encrypt.keyword.len()];
        let ans = self.encrypt.words[&current_char]
            .get(self.position / self.encrypt.keyword.len())
            .map(|x| x.to_owned());
        self.position += 1;
        ans
    }
}

fn write_result<T: Borrow<str>>(content: &T, filename: Option<PathBuf>) -> Option<()> {
    use std::io::Write;
    let mut file = File::create(filename?).ok()?;
    file.write_all(content.borrow().as_bytes()).ok()?;
    Some(())
}

fn main() -> ExitCode {
    let args = Args::parse();
    if !check_keyword(&args.keyword) {
        eprintln!(
            "Keyword {} contain duplicated or unexpected symbols",
            args.keyword
        );
        return ExitCode::from(1);
    }
    let content = match fs::read_to_string(&args.input_file) {
        Ok(content) => content.strip_suffix("\n").unwrap_or(&content).to_string(),
        Err(e) => {
            eprintln!("Failed to read input file {e}");
            return ExitCode::from(1);
        }
    };

    let new_content = match args.running_mode {
        RunningMode::Encrypt => {
            let mut encrypt = Cipher::new(args.keyword);
            let _ = encrypt.write_str(&content);
            encrypt.encrypt()
        }
        RunningMode::Decrypt => {
            let decrypt = Cipher::from_content(content, args.keyword);
            decrypt.decrypt()
        }
    };
    if write_result(&new_content, args.output_file).is_none() {
        println!("Result:\n{new_content}");
    }
    ExitCode::from(0)
}

#[cfg(test)]
mod tests {
    use crate::Cipher;
    use std::fmt::Write;

    #[test]
    fn complex_test() {
        let initial_text: String = String::from("перестановочный шифр");
        let keyword: String = String::from("шифр");
        let mut encrypt = Cipher::new(keyword.clone());
        let _ = encrypt.write_str(&initial_text);
        let encrypted = encrypt.encrypt();
        assert_eq!(encrypted, String::from("етвыиенч рраойфпсонш"));
        let decrypt = Cipher::from_content(encrypted, keyword);
        let decrypted = decrypt.decrypt();
        assert_eq!(decrypted, initial_text);
    }
}

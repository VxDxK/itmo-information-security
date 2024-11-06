use clap::{Parser, ValueEnum};
use std::borrow::Borrow;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::process::ExitCode;

/// Проверка строки на удовлетворение условиям ключа - содержит неповторящиеся символы алфавита
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

/// Объект шифра, может использоваться как для шифрования текста с помощью ключа так и для дешифрования уже имеющегося
#[derive(Debug)]
struct Cipher {
    keyword: Vec<char>,
    position: usize,
    words: BTreeMap<char, Vec<char>>,
}

impl Cipher {
    /// Конструктор пустого шифра из ключа
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

    /// Конструктор шифра из зашифрованных данных и ключа с помощью которого проводилось шифрование
    fn from_content(content: String, keyword: String) -> Self {
        let keyword: Vec<char> = keyword.chars().collect();
        let mut words = BTreeMap::new();
        let content: Vec<char> = content.chars().collect();
        for char in keyword.iter() {
            words.insert(char.to_owned(), Vec::new());
        }
        let usize_signum = |value: usize| if value == 0 { 0usize } else { 1usize };

        let mut in_use = 0;
        let mut char_to_use = HashMap::new();
        for (i, c) in keyword.iter().enumerate() {
            let to_use = (content.len() - in_use) / (keyword.len() - i)
                + usize_signum((content.len() - in_use) % (keyword.len() - i));
            println!("char: {c} to_use: {to_use}");
            char_to_use.insert(*c, to_use);
            in_use += to_use;
        }
        let mut in_use = 0;
        for (c, v) in words.iter_mut() {
            let to_use = char_to_use[c];
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
        let iterator = CipherIterator::new(self);
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

/// Структура итератора над шифром
struct CipherIterator {
    cipher: Cipher,
    position: usize,
}

impl CipherIterator {
    pub fn new(cipher: Cipher) -> Self {
        Self {
            cipher,
            position: 0,
        }
    }
}

impl Iterator for CipherIterator {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let current_char = self.cipher.keyword[self.position % self.cipher.keyword.len()];
        let ans = self.cipher.words[&current_char]
            .get(self.position / self.cipher.keyword.len())
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
    // Чтение файла
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
    ExitCode::SUCCESS
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

    #[test]
    fn complex_test2() {
        let initial_text: String = String::from("перест");
        println!("{}", initial_text);
        let keyword: String = String::from("шифр");
        let mut encrypt = Cipher::new(keyword.clone());
        let _ = encrypt.write_str(&initial_text);
        println!("{:#?}", encrypt);
        let encrypted = encrypt.encrypt();
        println!("{}", encrypted);
        let decrypt = Cipher::from_content(encrypted, keyword);
        println!("{:#?}", decrypt);
        let decrypted = decrypt.decrypt();
        println!("{}", decrypted);
        assert_eq!(decrypted, initial_text);
    }

    #[test]
    fn complex_test3() {
        let initial_text: String = String::from("Трус умирает каждый день, а воин ожидает свою гибель и живет каждый день, если человек ушел из жизни раньше времени, то он обретает вечное существование, но правила таковы, нельзя убивать самого себя и спровоцировать свою гибель, и поэтому в нас заложены самосохранение, интуиция и инстинкт.");
        println!("{}", initial_text);
        let keyword: String = String::from("шифр");
        let mut encrypt = Cipher::new(keyword.clone());
        let _ = encrypt.write_str(&initial_text);
        println!("{:#?}", encrypt);
        let encrypted = encrypt.encrypt();
        println!("{}", encrypted);
        let decrypt = Cipher::from_content(encrypted, keyword);
        println!("{:#?}", decrypt);
        let decrypted = decrypt.decrypt();
        println!("{}", decrypted);
        assert_eq!(decrypted, initial_text);
    }
}

use console::Term;

struct Fcsr {
    state: Vec<u8>,
    taps: Vec<usize>,
    carry: u8,
    length: usize,
}

impl Fcsr {
    fn new(initial_state: Vec<u8>, taps: Vec<usize>, length: usize) -> Self {
        assert_eq!(initial_state.len(), length);
        Fcsr {
            state: initial_state,
            taps,
            carry: 0,
            length,
        }
    }

    fn next_bit(&mut self) -> u8 {
        let mut feedback = self.carry;

        for &tap in &self.taps {
            feedback += self.state[tap];
        }

        let new_bit = feedback % 2;
        self.carry = feedback / 2;

        self.state.rotate_left(1);
        self.state[self.length - 1] = new_bit;
        new_bit
    }
}

struct Cipher {
    fcsr1: Fcsr,
    fcsr2: Fcsr,
    fcsr3: Fcsr,
}

impl Cipher {
    pub fn new() -> Self {
        let fcsr1 = Fcsr::new(vec![1; 96], vec![96 - 1, 95 - 1, 45 - 1, 2 - 1], 96);
        let fcsr2 = Fcsr::new(vec![1; 96], vec![96 - 1, 88 - 1, 79 - 1, 2 - 1], 96);
        let fcsr3 = Fcsr::new(vec![1; 96], vec![96 - 1, 69 - 1, 17 - 1, 2 - 1], 96);
        Self {
            fcsr1,
            fcsr2,
            fcsr3,
        }
    }

    fn generate_gamma(&mut self) -> u32 {
        (self.fcsr1.next_bit() ^ self.fcsr2.next_bit() ^ self.fcsr3.next_bit()) as u32
    }

    pub fn process_char(&mut self, ch: char) -> char {
        let gamma = self.generate_gamma();
        std::char::from_u32((ch as u32) ^ gamma).expect("invalid char")
    }

    #[allow(dead_code)]
    pub fn process_str(&mut self, text: &str) -> String {
        let mut result = Vec::new();
        for ch in text.chars() {
            result.push(self.process_char(ch));
        }
        result.iter().collect()
    }
}

fn main() {
    let mut cipher = Cipher::new();
    let term = Term::stdout();
    term.clear_screen().unwrap();
    let mut input = String::new();
    let mut output = String::new();
    loop {
        let ch = term.read_char().unwrap();
        term.clear_screen().unwrap();
        input.push(ch);
        let enc = cipher.process_char(ch);
        output.push(enc);
        println!("input: \"{}\"\noutput: \"{}\"", input, output);
    }
}

#[cfg(test)]
mod tests {
    use crate::Cipher;

    #[test]
    fn encrypt_and_decrypt() {
        let text = "Привет, Rust пока ***@@@ жизнь! :(";
        let mut cipher = Cipher::new();
        let enc = cipher.process_str(text);
        let mut cipher = Cipher::new();
        let dec = cipher.process_str(enc.as_str());
        assert_eq!(text, dec);
    }

    #[test]
    fn encrypt_and_decrypt2() {
        let text = "ven-ig3-2tr345g 4 134 4y  f  kf13ык 32 345 р5кнт кл ё2о 353 ";
        let mut cipher = Cipher::new();
        let enc = cipher.process_str(text);
        let mut cipher = Cipher::new();
        let dec = cipher.process_str(enc.as_str());
        assert_eq!(text, dec);
    }
}

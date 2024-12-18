use encoding_rs::WINDOWS_1251;
use num_bigint::BigInt;
use std::process::ExitCode;

fn decrypt(n: &BigInt, e: &BigInt, c: &[BigInt]) -> Option<String> {
    let mut ans = String::new();
    for now in c {
        let mut yi = now.modpow(e, n);
        let mut result = BigInt::ZERO;
        while yi != *now {
            result = yi.clone();
            yi = yi.modpow(e, n);
        }
        let (_, bytes) = result.to_bytes_be();
        match WINDOWS_1251.decode(&bytes) {
            (str, _, false) => ans.push_str(&str),
            (_, _, true) => return None,
        }
    }
    Some(ans)
}

fn main() -> ExitCode {
    let numbers = [
        BigInt::from(54879925681459u64),
        BigInt::from(72167008182929u64),
        BigInt::from(17828219756166u64),
        BigInt::from(17814399744948u64),
        BigInt::from(37136636080011u64),
        BigInt::from(77223434260215u64),
        BigInt::from(4272415279426u64),
        BigInt::from(73759271926435u64),
        BigInt::from(74021335775875u64),
        BigInt::from(16903113250201u64),
        BigInt::from(77520052156956u64),
        BigInt::from(41247980943013u64),
    ];
    if let Some(result) = decrypt(
        &BigInt::from(84032429242009u64),
        &BigInt::from(2581907),
        &numbers,
    ) {
        println!("Result: '{result}'",);
    } else {
        print!("[ERROR]");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use crate::decrypt;
    use num_bigint::BigInt;

    #[test]
    fn complex_test() {
        let numbers = [
            BigInt::from(54879925681459u64),
            BigInt::from(72167008182929u64),
            BigInt::from(17828219756166u64),
            BigInt::from(17814399744948u64),
            BigInt::from(37136636080011u64),
            BigInt::from(77223434260215u64),
            BigInt::from(4272415279426u64),
            BigInt::from(73759271926435u64),
            BigInt::from(74021335775875u64),
            BigInt::from(16903113250201u64),
            BigInt::from(77520052156956u64),
            BigInt::from(41247980943013u64),
        ];
        assert_eq!(
            decrypt(
                &BigInt::from(84032429242009u64),
                &BigInt::from(2581907),
                &numbers,
            ),
            Some("параллельными мостами, а всемаршрутные пакеты -_".to_string())
        );
    }
}

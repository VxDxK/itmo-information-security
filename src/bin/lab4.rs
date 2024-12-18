use encoding_rs::WINDOWS_1251;
use num_bigint::{BigInt, ToBigInt};
use num_integer::Integer;
use num_traits::One;
use std::process::ExitCode;

fn fermat_factorization(n: &BigInt) -> (BigInt, BigInt) {
    let mut a = n.sqrt() + 1.to_bigint().unwrap();
    loop {
        let w = &a * &a - n;
        let b = w.sqrt();
        if &b * &b == w {
            let p = &a + &b;
            let q = &a - &b;
            return (p, q);
        }
        a += 1.to_bigint().unwrap();
    }
}

fn mod_inverse(e: &BigInt, phi: &BigInt) -> Option<BigInt> {
    let egc = e.extended_gcd(phi);
    let (g, x) = (egc.gcd, egc.x);
    if g != BigInt::one() {
        None
    } else {
        Some((x % phi + phi) % phi)
    }
}

fn decrypt(n: &BigInt, e: &BigInt, c: &[BigInt]) -> Option<String> {
    let (p, q) = fermat_factorization(n);
    let phi = (&p - 1.to_bigint().unwrap()) * (&q - 1.to_bigint().unwrap());
    let d = mod_inverse(e, &phi)?;
    let mut message = String::new();
    for c_block in c {
        let m = c_block.modpow(&d, n);
        let bytes = m.to_bytes_be().1;
        match WINDOWS_1251.decode(&bytes) {
            (str, _, false) => message.push_str(&str),
            (_, _, true) => return None,
        }
    }
    Some(message)
}

fn main() -> ExitCode {
    let c = [
        BigInt::from(32279109612093u64),
        BigInt::from(17838629182964u64),
        BigInt::from(4165776716262u64),
        BigInt::from(13093284635895u64),
        BigInt::from(20048651313008u64),
        BigInt::from(54626454832531u64),
        BigInt::from(12801053743903u64),
        BigInt::from(54675332003643u64),
        BigInt::from(4544911979279u64),
        BigInt::from(31928373564570u64),
        BigInt::from(798945495513u64),
        BigInt::from(19569174668782u64),
    ];
    if let Some(result) = decrypt(&BigInt::from(59046883376179u64), &BigInt::from(4044583), &c) {
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

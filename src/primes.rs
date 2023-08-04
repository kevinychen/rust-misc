use std::ops::Range;

use bnum::{prelude::As, types::I256};
use rand::{rngs::StdRng, Rng, SeedableRng};

fn pow_mod(a: I256, e: I256, m: I256) -> I256 {
    if e.is_zero() {
        I256::ONE
    } else {
        let mut res = pow_mod(a, e.shr(1), m);
        res = res * res % m;
        if e.bit(0) {
            res = res * a % m
        }
        res
    }
}

/// Returns a list of all primes less than n, in ascending order
fn get_primes(n: u32) -> Vec<u32> {
    let n = n as usize;
    let mut sieve = vec![true; n];
    let mut i = 3;
    while i * i < n {
        if sieve[i] {
            for j in (i * i..n).step_by(2 * i) {
                sieve[j] = false;
            }
        }
        i += 2
    }
    let mut primes: Vec<u32> = vec![2];
    for i in (3..n).step_by(2) {
        if sieve[i] {
            primes.push(i as u32)
        }
    }
    primes
}

/// Returns whether n is prime
fn miller_rabin(n: I256, rng: &mut StdRng) -> bool {
    if n <= I256::from(3) {
        return n >= I256::from(2);
    }

    let k = 20;
    let mut s = 0;
    let mut d = n - I256::ONE;
    while !d.bit(0) {
        s += 1;
        d = d.shr(1);
    }

    for _ in 0..k {
        let a = rng.gen_range(I256::TWO..n - I256::ONE);
        let mut x = pow_mod(a, d, n);
        for _ in 0..s {
            let y = x * x % n;
            if y.is_one() && !x.is_one() && x != n - I256::ONE {
                return false;
            }
            x = y;
        }
        if !x.is_one() {
            return false;
        }
    }
    true
}

pub fn find_prime(range: Range<I256>, rng: &mut StdRng) -> I256 {
    let window_len = range.start.bits() * 2;
    let primes = get_primes(window_len);

    loop {
        let low = rng.gen_range(range.clone());
        let mut sieve = vec![true; window_len as usize];
        for p in &primes {
            let start = p - 1 - (low % I256::from(*p)).as_::<u32>();
            for i in (start..window_len).step_by(*p as usize) {
                sieve[i as usize] = false;
            }
            for i in 0..window_len {
                if sieve[i as usize] {
                    let n = low + I256::from(i + 1);
                    if miller_rabin(n, rng) {
                        return n;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_miller_rabin() {
        const L: u32 = 10000;
        let mut rng = StdRng::seed_from_u64(0);

        let primes = get_primes(L);
        let mut is_prime = vec![false; L as usize];
        for p in primes {
            is_prime[p as usize] = true;
        }
        for i in 2..L {
            assert!(miller_rabin(I256::from(i), &mut rng) == is_prime[i as usize]);
        }
    }

    #[test]
    #[ignore]
    fn test_find_prime() {
        let start = I256::from(1000000000000000000_u64);
        let mut rng = StdRng::seed_from_u64(0);

        for _ in 0..100 {
            find_prime(start..start.shl(1), &mut rng);
        }
    }
}

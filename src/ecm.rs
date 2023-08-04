use std::fmt::Debug;

use super::primes;
use bnum::types::I256;
use rand::{rngs::StdRng, Rng, SeedableRng};

/// Finds a factor of n using Lenstra elliptic-curve factorization.
/// n must be an odd number with at least two distinct prime factors.
/// https://en.wikipedia.org/wiki/Lenstra_elliptic-curve_factorization
pub fn find_factor(n: I256) -> I256 {
    // https://www.rieselprime.de/ziki/Elliptic_curve_method#Choosing_the_best_parameters_for_ECM
    let limit = 1u32 << (n.bits() / 5);
    println!("limit: {}", limit);
    let mut rng = StdRng::seed_from_u64(0);
    let primes = primes::get_primes(limit);

    loop {
        let curve = EllipticCurve {
            a: rng.gen_range(I256::ONE..n),
        };
        let mut p = Point {
            x: rng.gen_range(I256::ONE..n),
            y: rng.gen_range(I256::ONE..n),
        };
        println!("curve: {} {:?}", curve.a, p);

        for prime in &primes {
            let mut i = *prime;
            while prime * i < limit {
                i *= prime;
            }
            match curve.multiply(p, i, n) {
                Ok(q) => p = q,
                Err(g) => return g,
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Point {
    x: I256,
    y: I256,
}

/// Represents y² = x³+ax+b (mod m)
struct EllipticCurve {
    a: I256,
    // b: I256,
}

impl EllipticCurve {
    fn add(&self, p: Point, q: Point, m: I256) -> Result<Point, I256> {
        let lambda = if p.x == q.x {
            (I256::THREE * p.x * p.x + self.a) % m * mod_inverse(I256::TWO * p.y, m)?
        } else {
            (q.y - p.y) * mod_inverse(q.x - p.x, m)?
        } % m;
        let x = (lambda * lambda - p.x - q.x) % m;
        Ok(Point {
            x,
            y: (lambda * (p.x - x) - p.y) % m,
        })
    }

    fn multiply(&self, p: Point, k: u32, m: I256) -> Result<Point, I256> {
        if k == 1 {
            Ok(p)
        } else {
            let mut q = self.multiply(p, k / 2, m)?;
            q = self.add(q, q, m)?;
            if k % 2 == 1 {
                q = self.add(q, p, m)?;
            }
            Ok(q)
        }
    }
}

fn gcd(a: I256, b: I256) -> I256 {
    if b.is_zero() {
        a
    } else {
        gcd(b, a % b)
    }
}

/// Returns (x,y) such that ax + by = GCD(a,b)
fn find_linear_combination(a: I256, b: I256) -> Point {
    if b.is_zero() {
        Point {
            x: a.signum(),
            y: I256::ZERO,
        }
    } else {
        let Point { x, y } = find_linear_combination(b, a % b);
        Point {
            x: y,
            y: x - a / b * y,
        }
    }
}

/// Returns either Ok(a^⁻¹ (mod n)), or Err(GCD(a,n)) if a doesn't have an inverse
fn mod_inverse(a: I256, n: I256) -> Result<I256, I256> {
    let Point { x, y } = find_linear_combination(a, n);
    let g = a * x + n * y;
    if g.is_one() {
        Ok(x)
    } else {
        Err(g)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_find_factor() {
        let start = I256::from(10u64.pow(11));
        let mut rng = StdRng::seed_from_u64(0);

        for i in 0..5 {
            println!("test {}", i + 1);
            let a = primes::find_prime(start..start.shl(1), &mut rng);
            let b = primes::find_prime(start..start.shl(1), &mut rng);
            let factor = find_factor(a * b);
            assert!(factor == a || factor == b);
        }
    }
}

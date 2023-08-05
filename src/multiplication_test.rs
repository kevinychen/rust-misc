const R: u64 = 1u64 << 32;

struct Point {
    x: i64,
    y: i64,
}

fn find_linear_combination(a: i64, b: i64) -> Point {
    if b == 0 {
        Point {
            x: if a >= 0 { 1 } else { -1 },
            y: 0,
        }
    } else {
        let Point { x, y } = find_linear_combination(b, a % b);
        Point {
            x: y,
            y: x - a / b * y,
        }
    }
}

fn mod_inverse(a: u64, n: u64) -> u64 {
    let inverse = find_linear_combination(a as i64, n as i64).x;
    if inverse < 0 {
        (inverse + n as i64) as u64
    } else {
        inverse as u64
    }
}

fn redc(t: u64, n: u64, np: u64) -> u64 {
    let tt = (t + t % R * np % R * n) / R;
    if tt < n {
        tt
    } else {
        tt - n
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};
    use std::time::Instant;

    use crate::multiplication_test::mod_inverse;

    /*
    This takes ~9.5s with normal multiplication, and ~8.1s with Montgomery multiplication, a 15% speedup.
     */
    const COUNT: usize = 1000000000;
    const P: u64 = 1000000007;

    #[test]
    fn test_montgomery() {
        let mut rng = StdRng::seed_from_u64(0);

        let mut nums = vec![];
        let mut num = 2;
        for _ in 0..COUNT {
            nums.push(num);
            num = num * num % P + 1;
        }

        let start = Instant::now();

        let np = mod_inverse(R - P, R);
        let r2 = R % P * R % P;
        let mut prod = R;
        for num in nums {
            let mnum = redc(num * r2, P, np);
            prod = redc(prod * mnum, P, np);
            // prod *= num;
            // prod %= P;
        }
        prod = redc(prod, P, np);

        println!("Product: {} ({:?})", prod, start.elapsed());
        assert_eq!(prod, 450032353);
    }
}

#[cfg(test)]
mod tests {
    use bnum::types::I8192;
    use crypto_bigint::{Random, Uint, Wrapping, U8192};
    use ibig::ibig;
    use rand::{rngs::StdRng, Rng, SeedableRng};
    use rug::rand::RandState;
    use rug::{Assign, Integer};
    use std::time::{Duration, Instant};

    /*
    Time elapsed: 50.339727ms
    test multiplication_test::tests::test_rug ... ok
    Time elapsed: 107.228613ms
    test multiplication_test::tests::test_ibig ... ok
    Time elapsed: 166.851162ms
    test multiplication_test::tests::test_crypto_bigint ... ok
    Time elapsed: 103.439212ms
    test multiplication_test::tests::test_bnum ... ok
     */
    const COUNT: usize = 10000;
    const NUM_WORDS: usize = 128;

    #[test]
    fn test_ibig() {
        let mut rng = StdRng::seed_from_u64(0);

        let mut total_duration = Duration::ZERO;

        let mut sum_products = ibig!(1);
        for _ in 0..COUNT {
            let a = rng.gen_range(ibig!(1)..ibig!(2).pow(64 * NUM_WORDS));
            let b = rng.gen_range(ibig!(1)..ibig!(2).pow(64 * NUM_WORDS));
            let start = Instant::now();
            sum_products += a * b;
            total_duration += start.elapsed();
        }

        println!("Time elapsed: {:?}", total_duration);
    }

    #[test]
    fn test_bnum() {
        let mut rng = StdRng::seed_from_u64(0);

        let mut total_duration = Duration::ZERO;

        let mut sum_products = I8192::ONE;
        for _ in 0..COUNT {
            let a = rng.gen_range(I8192::ONE..I8192::from(2).pow(NUM_WORDS as u32));
            let b = rng.gen_range(I8192::ONE..I8192::from(2).pow(NUM_WORDS as u32));
            let start = Instant::now();
            sum_products += a * b;
            total_duration += start.elapsed();
        }

        println!("Time elapsed: {:?}", total_duration);
    }

    #[test]
    fn test_crypto_bigint() {
        let mut rng = StdRng::seed_from_u64(0);

        let mut total_duration = Duration::ZERO;

        let mut sum_products: Wrapping<Uint<NUM_WORDS>> = Wrapping(Uint::ONE);
        for _ in 0..COUNT {
            let a = Wrapping(U8192::random(&mut rng));
            let b = Wrapping(U8192::random(&mut rng));
            let start = Instant::now();
            sum_products += a * b;
            total_duration += start.elapsed();
        }

        println!("Time elapsed: {:?}", total_duration);
    }

    #[test]
    fn test_rug() {
        let mut rng = RandState::new();

        let mut total_duration = Duration::ZERO;

        let mut sum_products = Integer::new();
        sum_products.assign(Integer::ONE);
        for _ in 0..COUNT {
            let a = Integer::from(Integer::random_bits(64 * NUM_WORDS as u32, &mut rng));
            let b = Integer::from(Integer::random_bits(64 * NUM_WORDS as u32, &mut rng));
            let start = Instant::now();
            sum_products += a * b;
            total_duration += start.elapsed();
        }

        println!("Time elapsed: {:?}", total_duration);
    }
}

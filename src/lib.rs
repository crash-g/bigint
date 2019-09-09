#![feature(extern_crate_item_prelude)]
#![feature(test)]

extern crate test;

pub mod optimized_memory {
    ///! Optimized implementation of BigInt using representation in base u32.
    ///! Atomic operations use casts to u64, leveraging the fact that overflow is
    ///! impossible.

    #[derive(Debug)]
    pub struct BigInt {
        data: Vec<u32>,
    }

    impl BigInt {
        const BASE: u64 = std::u32::MAX as u64 + 1;
        const PARSE_STEP: usize = 8;

        pub fn zero() -> BigInt {
            BigInt { data: Vec::new() }
        }

        /// Convert a decimal string to BigInt.
        pub fn from_string(s: &str) -> BigInt {
            let mut chunks = split_string(s, BigInt::PARSE_STEP);
            let mut result = BigInt::zero();
            loop {
                let mut carry = 0;
                for i in 0..chunks.len() {
                    let temp: u64 = if carry > 0 {
                        let original_chunk_size = if i == chunks.len() - 1 {
                            s.len() % BigInt::PARSE_STEP
                        } else {
                            BigInt::PARSE_STEP
                        };
                        BigInt::apply_carry(chunks[i], carry, original_chunk_size)
                    } else {
                        chunks[i]
                    };
                    let quotient = temp / BigInt::BASE;
                    let remainder = temp % BigInt::BASE;
                    chunks[i] = quotient;
                    carry = remainder;
                }
                result.data.push(carry as u32);

                if BigInt::all_zero(&chunks) {
                    break;
                }
            }
            result
        }

        /// Helper function for `from_string`.
        fn apply_carry(u: u64, carry: u64, original_size: usize) -> u64 {
            let u_string = u.to_string();
            if u_string.len() < original_size {
                (carry.to_string() + &"0".repeat(original_size - u_string.len()) + &u_string)
                    .parse()
                    .unwrap()
            } else {
                (carry.to_string() + &u.to_string()).parse().unwrap()
            }
        }

        /// Helper function for `from_string`.
        fn all_zero(v: &[u64]) -> bool {
            for u in v {
                if *u > 0 {
                    return false;
                }
            }

            return true;
        }

        fn get(&self, i: usize) -> u32 {
            if i < self.data.len() {
                self.data[i]
            } else {
                0
            }
        }
    }

    impl PartialEq for BigInt {
        fn eq(&self, other: &Self) -> bool {
            let largest = std::cmp::max(self.data.len(), other.data.len());
            for i in 0..largest {
                if self.get(i) != other.get(i) {
                    return false;
                }
            }
            true
        }
    }

    impl Eq for BigInt {}

    pub fn sum(b1: &BigInt, b2: &BigInt) -> BigInt {
        let mut result = BigInt::zero();
        let largest = std::cmp::max(b1.data.len(), b2.data.len());
        let mut carry = 0;
        for i in 0..largest {
            let digit_sum = b1.get(i) as u64 + b2.get(i) as u64 + carry;
            if digit_sum >= BigInt::BASE {
                result.data.push((digit_sum - BigInt::BASE) as u32);
                carry = 1;
            } else {
                result.data.push(digit_sum as u32);
                carry = 0;
            }
        }

        if carry == 1 {
            result.data.push(1);
        }

        result
    }

    pub fn product(b1: &BigInt, b2: &BigInt) -> BigInt {
        let mut result = BigInt::zero();

        for (i, d) in b2.data.iter().enumerate() {
            if *d > 0 {
                let mut temp = BigInt { data: vec![0; i] };
                temp.data.extend(atomic_product(&b1, *d).data);
                result = sum(&result, &temp);
            }
        }

        result
    }

    fn atomic_product(b1: &BigInt, d: u32) -> BigInt {
        let mut result = BigInt::zero();
        let mut carry = 0;
        for d1 in &b1.data {
            let digit_product = (*d1 as u64 * d as u64) + carry;
            result.data.push((digit_product % BigInt::BASE) as u32);
            carry = digit_product / BigInt::BASE;
        }

        if carry > 0 {
            result.data.push(carry as u32);
        }

        result
    }

    fn split_string(s: &str, step: usize) -> Vec<u64> {
        let mut result = Vec::new();
        let mut i = 0;
        while i < s.len() {
            let right = std::cmp::min(i + step, s.len());
            result.push(s[i..right].parse().unwrap());
            i = i + step;
        }
        result
    }

    #[cfg(test)]
    mod tests {

        use super::*;
        use test::Bencher;

        #[bench]
        fn bench_sum_short(b: &mut Bencher) {
            let b1 = BigInt::from_string("34324");
            let b2 = BigInt::from_string("11");
            b.iter(|| sum(&b1, &b2))
        }

        #[bench]
        fn bench_sum_long(b: &mut Bencher) {
            let b1 = BigInt::from_string("9999999999999999999999999999999999999999999999999");
            let b2 = BigInt::from_string("111111111111111111111111111111111123432342342111");
            b.iter(|| sum(&b1, &b2))
        }

        #[bench]
        fn bench_product_short(b: &mut Bencher) {
            let b1 = BigInt::from_string("34324");
            let b2 = BigInt::from_string("11");
            b.iter(|| product(&b1, &b2))
        }

        #[bench]
        fn bench_product_long(b: &mut Bencher) {
            let b1 = BigInt::from_string("9999999999999999999999999999999999999999999999999");
            let b2 = BigInt::from_string("111111111111111111111111111111111123432342342111");
            b.iter(|| product(&b1, &b2))
        }

        #[test]
        fn test_eq() {
            assert!(BigInt::from_string("") == BigInt::from_string(""));
            assert!(BigInt::from_string("") == BigInt{data: vec![0, 0]});
            assert!(BigInt::from_string("342") == BigInt{data: vec![342]});
            assert!(BigInt::from_string("342") == BigInt{data: vec![342, 0, 0]});
            assert!(BigInt{data: vec![342, 0, 0, 0]} == BigInt{data: vec![342, 0]});
            assert!(BigInt{data: vec![0, 342, 0, 0]} != BigInt{data: vec![342, 0, 0]});
        }

        #[test]
        fn test_from_string() {
            assert_eq!(BigInt { data: vec![4] }, BigInt::from_string("4"));
            assert_eq!(BigInt::zero(), BigInt::from_string(""));
            assert_eq!(
                BigInt {
                    data: vec![4294967295]
                },
                BigInt::from_string("4294967295")
            );
            assert_eq!(
                BigInt { data: vec![0, 1] },
                BigInt::from_string("4294967296")
            );
            assert_eq!(
                BigInt {
                    data: vec![3435973836, 214748364]
                },
                BigInt::from_string("922337203685477580")
            );
            assert_eq!(
                BigInt {
                    data: vec![4294963245, 4294967295, 499]
                },
                BigInt::from_string("9223372036854775803949")
            );
            assert_eq!(
                BigInt {
                    data: vec![3461744650, 2330743505, 1228788904, 542101086]
                },
                BigInt::from_string("42949672963434342343243324343232890890")
            );
        }

        #[test]
        fn test_sum() {
            assert_eq!(
                BigInt {
                    data: vec![0, 3, 1]
                },
                sum(
                    &BigInt {
                        data: vec![(BigInt::BASE - 1) as u32, 1]
                    },
                    &BigInt {
                        data: vec![1, 1, 1]
                    }
                )
            );
            assert_eq!(
                BigInt::from_string("683598743919434280434619734254544588"),
                sum(
                    &BigInt::from_string("683598349590386730945834985730495834"),
                    &BigInt::from_string("394329047549488784748524048754")
                )
            );
            assert_eq!(
                BigInt::from_string("10111111111111111111111111111111111123432342342110"),
                sum(
                    &BigInt::from_string("9999999999999999999999999999999999999999999999999"),
                    &BigInt::from_string("111111111111111111111111111111111123432342342111")
                )
            );
        }

        #[test]
        fn test_product() {
            assert_eq!(
                BigInt {
                    data: vec![4294931842, 177267, 35464, 2]
                },
                product(
                    &BigInt {
                        data: vec![35454, 2]
                    },
                    &BigInt {
                        data: vec![(BigInt::BASE - 1) as u32, 4, 1]
                    }
                )
            );
            assert_eq!(BigInt::from_string("1111111111111111111111111111111111234323423421109888888888888888888888888888888888876567657657889"),
                       product(&BigInt::from_string("9999999999999999999999999999999999999999999999999"),
                               &BigInt::from_string("111111111111111111111111111111111123432342342111")));
        }
    }

}

pub mod easy {
    ///! Short, non-optimized implementation of BigInt.

    #[derive(Debug, PartialEq, Eq)]
    pub struct BigInt {
        data: Vec<u8>,
    }

    impl BigInt {
        pub fn zero() -> BigInt {
            BigInt { data: Vec::new() }
        }

        pub fn from_binary_string(s: &str) -> BigInt {
            let mut data = Vec::new();
            for c in s.chars() {
                data.push(c as u8 - 48);
            }
            BigInt{data}
        }

        fn times_two(&mut self) {
            self.data.insert(0, 0);
        }

        fn get(&self, i: usize) -> u8 {
            if i < self.data.len() {
                self.data[i]
            } else {
                0
            }
        }
    }

    pub fn sum(b1: &BigInt, b2: &BigInt) -> BigInt {
        let mut result = BigInt::zero();
        let largest = std::cmp::max(b1.data.len(), b2.data.len());
        let mut carry = 0;
        for i in 0..largest {
            match (b1.get(i), b2.get(i)) {
                (0, 0) => result.data.push(carry),
                (1, 0) | (0, 1) => result.data.push((1 + carry) % 2),
                (1, 1) => {
                    result.data.push(carry);
                    carry = 1;
                }
                _ => panic!("Something is wrong"),
            }
        }

        if carry == 1 {
            result.data.push(1);
        }

        result
    }

    pub fn product(b1: &BigInt, b2: &BigInt) -> BigInt {
        let mut result = BigInt::zero();
        let mut temp = BigInt {
            data: b1.data.clone(),
        };
        for digit in &b2.data {
            if *digit == 1 {
                result = sum(&result, &temp);
            }
            temp.times_two();
        }
        result
    }

    #[cfg(test)]
    mod tests {

        use super::*;
        use test::Bencher;

        #[bench]
        fn bench_sum_short(b: &mut Bencher) {
            let b1 = BigInt::from_binary_string("1000011000010100");
            let b2 = BigInt::from_binary_string("1011");
            b.iter(|| sum(&b1, &b2))
        }

        #[bench]
        fn bench_product_short(b: &mut Bencher) {
            let b1 = BigInt::from_binary_string("1000011000010100");
            let b2 = BigInt::from_binary_string("1011");
            b.iter(|| product(&b1, &b2))
        }

        #[test]
        fn test_from_binary_string() {
            assert_eq!(BigInt{data: vec![1, 0, 1, 1]}, BigInt::from_binary_string("1011"));
        }

        #[test]
        fn test_sum() {
            assert_eq!(
                BigInt {
                    data: vec![1, 0, 1, 0, 1]
                },
                sum(
                    &BigInt {
                        data: vec![1, 1, 1]
                    },
                    &BigInt {
                        data: vec![0, 1, 1, 1]
                    }
                )
            );
        }

        #[test]
        fn test_product() {
            assert_eq!(
                BigInt {
                    data: vec![0, 1, 0, 1]
                },
                product(
                    &BigInt { data: vec![0, 1] },
                    &BigInt {
                        data: vec![1, 0, 1]
                    }
                )
            );
        }

    }

}

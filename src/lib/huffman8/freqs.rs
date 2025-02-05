use crate::constants::FESTError;

pub struct Freqs {
    pub freqs: Vec<usize>,
    pub num_leafs: usize,
}

impl Freqs {
    pub fn create_freqs(data: &[u8]) -> Result<Self, FESTError> {
        println!("CREATE_FREQS");

        let mut freqs = vec![0usize; 0xFF + 1];

        for &byte in data {
            // NOTE: "nbits" isn't used beyond the condition for the loop, so it can be omitted
            // NOTE: Also dropped "num_bits", maybe it had a use at some point, but the code doesn't really need it
            let ch = unsafe { freqs.get_unchecked_mut(byte as usize) };

            match ch.checked_add(1) {
                Some(_) => *ch += 1,
                None => return Err(FESTError::FreqOverflow(byte)),
            }
            // *freqs.get_mut(byte as usize)? += 1;
        }

        let mut num_leafs = freqs.iter().filter(|&freq| *freq != 0usize).count();

        if num_leafs < 2 {
            if num_leafs == 1 {
                for byte in &mut *freqs {
                    if *byte != 0usize {
                        *byte = 1;
                        break;
                    }
                }
            }

            // NOTE: There's no need to break the for loop right after the first 0 since, on the next loop, we'll pass by it anyway, so we just treat "num_leafs" as a counter, and once that counter reaches the end after modifiying "freqs", only then we break the loop
            for byte in &mut *freqs {
                if *byte == 0 {
                    *byte = 2;

                    num_leafs += 1;

                    if num_leafs >= 2 {
                        break;
                    }
                }
            }
        }

        Ok(Freqs { freqs, num_leafs })
    }
}

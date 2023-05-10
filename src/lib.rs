use ff_ce::{Field, PrimeField};
use mimc_sponge_rs::{Fr, MimcSponge};
const DEFAULT_ZERO: &str =
    "21663839004416932945382355908790599225266501822907911457504978515578255421292";

#[derive(Debug)]
pub struct MerkleTree {
    pub levels: usize,
    pub capacity: u128,
    pub zero_element: String,
    pub zeros: Vec<Fr>,
    pub layers: Vec<Vec<Fr>>, //todo: should change to hash table
    pub hash_fn: fn(Fr, Fr) -> Fr,
}

impl MerkleTree {
    fn default_hash(left: Fr, right: Fr) -> Fr {
        let arr = vec![left, right];
        let ms = MimcSponge::default();
        let k = Fr::zero();
        let res = ms.multi_hash(&arr, k, 1);
        return res[0];
    }

    fn new(
        levels: usize,
        zero_element: Option<String>,
        hash_fn: Option<fn(Fr, Fr) -> Fr>,
        elements: Option<Vec<Fr>>,
    ) -> MerkleTree {
        let capacity = 2u128.pow(u32::try_from(levels).unwrap());
        let zero_element = match zero_element {
            Some(s) => s, //todo: check sanity
            None => DEFAULT_ZERO.into(),
        };
        let hash_fn = match hash_fn {
            Some(f) => f,
            None => MerkleTree::default_hash,
        };
        let mut zeros = Vec::<Fr>::new();
        let mut layers = Vec::<Vec<Fr>>::new();
        zeros.push(Fr::from_str(&zero_element).unwrap());
        layers.push(Vec::<Fr>::new()); //to initiate first layer
        for i in 1..=levels {
            zeros.push(hash_fn(zeros[i - 1], zeros[i - 1]));
            layers.push(Vec::<Fr>::new()); //to initiate each layer
        }
        match elements {
            Some(v) => layers[0] = v,
            None => (),
        }

        let mut mk = MerkleTree {
            levels: levels,
            capacity: capacity,
            zero_element: zero_element,
            zeros: zeros,
            layers: layers,
            hash_fn: hash_fn,
        };

        mk.rebuild();

        mk
    }

    fn rebuild(&mut self) {
        for level in 1..=self.levels {
            println!("level:{}", level);
            let lim = (self.layers[level - 1].len() + 1) / 2; //ceil(layers[level-1].len()/2)
            println!("lim:{}", lim);
            for i in 0..lim {
                let left = self.layers[level - 1][i * 2];
                let right = if i * 2 + 1 < self.layers[level - 1].len() {
                    self.layers[level - 1][i * 2 + 1]
                } else {
                    self.zeros[level - 1]
                };
                println!("ind:{}", i);
                self.layers[level].push((self.hash_fn)(left, right));
            }
        }
    }

    pub fn root(&self) -> Fr {
        if self.layers[self.levels].len() == 0 {
            self.zeros[self.levels]
        } else {
            self.layers[self.levels][0]
        }
    }

    // pub fn path(&self, index: u128) {
    //     // if index >= self.capacity {
    //     //     //error
    //     //     return
    //     // }
    //     let mut pathElements = Vec::<Fr>::new();
    //     let mut pathIndices = Vec::<u128>::new();
    //     for level in 0..self.levels {
    //         pathIndices.push(index % 2);
    //         let element = if index ^ 1 < self.layers[level].len().try_into().unwrap() {
    //             self.layers[level][index ^ 1]
    //         } else {
    //             self.zeros[level]
    //         };

    //         pathElements.push
    //     }
    // }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    pub fn test_new() {
        // let elements = vec![
        // //     Fr::from_str("1").unwrap(),
        // //     Fr::from_str("7").unwrap(),
        // //     Fr::from_str("3").unwrap(),
        // //     Fr::from_str("4").unwrap(),
        //  ];
        let a = MerkleTree::new(6, None, None, None);
        println!("{:?}", a.root());
    }
}

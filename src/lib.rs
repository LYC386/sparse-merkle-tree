use std::thread::panicking;

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
    pub layers: Vec<Vec<Fr>>,
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
            Some(v) => {
                if v.len() > capacity.try_into().unwrap() {
                    panic!("Tree is full");
                };
                layers[0] = v
            }
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
            let lim = (self.layers[level - 1].len() + 1) / 2; //ceil(layers[level-1].len()/2)
            for i in 0..lim {
                let left = self.layers[level - 1][i * 2];
                let right = if i * 2 + 1 < self.layers[level - 1].len() {
                    self.layers[level - 1][i * 2 + 1]
                } else {
                    self.zeros[level - 1]
                };
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

    pub fn path(&self, mut index: u128) -> (Vec<Fr>, Vec<u128>) {
        if index >= self.layers[0].len().try_into().unwrap() {
            panic!("Index out of bound");
        }
        let mut pathElements = Vec::<Fr>::new();
        let mut pathIndices = Vec::<u128>::new();
        for level in 0..self.levels {
            pathIndices.push(index % 2);
            let element = if index ^ 1 < self.layers[level].len().try_into().unwrap() {
                self.layers[level][usize::try_from(index ^ 1).unwrap()]
            } else {
                self.zeros[level]
            };

            pathElements.push(element);

            index >>= 1;
        }
        (pathElements, pathIndices)
    }

    pub fn update(&mut self, mut index: usize, element: Fr) {
        if index >= self.layers[0].len() || index >= self.capacity.try_into().unwrap() {
            panic!("Index out of bound");
        }
        self.layers[0][index] = element;
        for level in 1..=self.levels {
            index >>= 1;
            let left = self.layers[level - 1][index * 2];
            let right = if index * 2 + 1 < self.layers[level - 1].len() {
                self.layers[level - 1][index * 2 + 1]
            } else {
                self.zeros[level - 1]
            };
            if index == self.layers[level].len() {
                self.layers[level].push((self.hash_fn)(left, right));
            } else {
                self.layers[level][index] = (self.hash_fn)(left, right);
            }
        }
    }

    pub fn insert(&mut self, element: Fr) {
        if u128::try_from(self.layers[0].len()).unwrap() >= self.capacity {
            panic!("Tree is full");
        }
        self.layers[0].push(Fr::zero());
        self.update(self.layers[0].len() - 1, element)
    }

    pub fn bulkInsert(&mut self, elements: Vec<Fr>) {
        if u128::try_from(self.layers[0].len() + elements.len()).unwrap() >= self.capacity {
            panic!("Tree is full");
        }

        for i in 0..elements.len() - 1 {
            self.layers[0].push(elements[i]);
            let mut level = 0;
            let mut index = self.layers[0].len() - 1;
            while index % 2 == 1 {
                level += 1;
                index >>= 1;
                let left = self.layers[level - 1][index * 2];
                let right = self.layers[level - 1][index * 2 + 1];
                if index == self.layers[level].len() {
                    self.layers[level].push((self.hash_fn)(left, right));
                } else {
                    self.layers[level][index] = (self.hash_fn)(left, right);
                }
            }
        }
        self.insert(elements[elements.len() - 1]);
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    pub fn test_new() {
        let elements = vec![
            Fr::from_str("100").unwrap(),
            Fr::from_str("7").unwrap(),
            Fr::from_str("3").unwrap(),
            Fr::from_str("4").unwrap(),
            Fr::from_str("5").unwrap(),
            Fr::from_str("6").unwrap(),
            Fr::from_str("7").unwrap(),
            Fr::from_str("8").unwrap(),
            Fr::from_str("9").unwrap(),
            Fr::from_str("10").unwrap(),
            Fr::from_str("11").unwrap(),
            Fr::from_str("12").unwrap(),
            Fr::from_str("13").unwrap(),
            Fr::from_str("14").unwrap(),
            Fr::from_str("15").unwrap(),
            Fr::from_str("16").unwrap(),
            Fr::from_str("17").unwrap(),
        ];
        let mut a = MerkleTree::new(11, None, None, Some(elements));
        println!("{:?}", a.root());
        println!("{:?}", a.path(4));
        a.update(16, Fr::from_str("44").unwrap());
        println!("{:?}", a.root());
        println!("{:?}", a.path(16));
        a.insert(Fr::from_str("100").unwrap());
        println!("{:?}", a.root());
        println!("{:?}", a.path(16));
        a.insert(Fr::from_str("100").unwrap());
        println!("{:?}", a.root());
        println!("{:?}", a.path(16));

        a.bulkInsert(vec![
            Fr::from_str("18").unwrap(),
            Fr::from_str("19").unwrap(),
            Fr::from_str("20").unwrap(),
            Fr::from_str("21").unwrap(),
        ]);
        println!("{:?}", a.path(20));
        println!("{:?}", a.root());
    }
}

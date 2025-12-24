extern crate num;
use num::PrimInt;

use std::ops::Add;

pub struct CritBit<K, V>(Option<CritBitNode<K, V>>)
where
    K: PrimInt;

enum CritBitNode<K, V>
where
    K: PrimInt,
{
    Leaf(K, V),
    Internal(InternalCritBitNode<K, V>),
}

struct InternalCritBitNode<K, V>
where
    K: PrimInt,
{
    left: Option<Box<CritBitNode<K, V>>>,
    right: Option<Box<CritBitNode<K, V>>>,
    crit: u32,
}

#[inline(always)]
fn bit_at<T: PrimInt>(value: &T, pos: &u32) -> bool {
    value.rotate_left(*pos).leading_zeros() == 0
}

impl<K, V> Default for CritBit<K, V>
where
    K: PrimInt,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> CritBit<K, V>
where
    K: PrimInt,
{
    pub fn new() -> CritBit<K, V> {
        CritBit(None)
    }

    pub fn clear(&mut self) {
        self.0 = None;
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    pub fn len(&self) -> usize {
        self.0.iter().map(CritBitNode::len).fold(0, Add::add)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        match &self.0 {
            Some(node) => node.get(key),
            &None => None,
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        match self.0 {
            Some(ref mut node) => node.get_mut(key),
            None => None,
        }
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match &mut self.0 {
            &mut Some(ref mut node) => node.insert(key, value),
            x => {
                x.replace(CritBitNode::Leaf(key, value));
                None
            }
        }
    }
}

impl<K: PrimInt, V> CritBitNode<K, V> {
    fn len(&self) -> usize {
        match *self {
            CritBitNode::Leaf(..) => 1,
            CritBitNode::Internal(InternalCritBitNode {
                ref left,
                ref right,
                ..
            }) => left
                .iter()
                .chain(right.iter())
                .map(|x| x.len())
                .fold(0, Add::add),
        }
    }

    fn get(&self, key: &K) -> Option<&V> {
        match *self {
            CritBitNode::Leaf(ref k, ref v) if *k == *key => Some(v),
            CritBitNode::Internal(InternalCritBitNode {
                left: Some(ref left),
                right: _,
                ref crit,
            }) if !bit_at(key, crit) => left.get(key),
            CritBitNode::Internal(InternalCritBitNode {
                left: _,
                right: Some(ref right),
                ref crit,
            }) if bit_at(key, crit) => right.get(key),
            _ => None,
        }
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        match *self {
            CritBitNode::Leaf(ref k, ref mut v) if *k == *key => Some(v),
            CritBitNode::Internal(InternalCritBitNode {
                left: Some(ref mut kid),
                right: _,
                ref crit,
            }) if !bit_at(key, crit) => kid.get_mut(key),
            CritBitNode::Internal(InternalCritBitNode {
                left: _,
                right: Some(ref mut kid),
                ref crit,
            }) if bit_at(key, crit) => kid.get_mut(key),
            _ => None,
        }
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        match *self {
            CritBitNode::Leaf(ref k, ref mut v) if *k == key => Some(std::mem::replace(v, value)),
            CritBitNode::Leaf(..) => {
                if let CritBitNode::Leaf(k, v) = std::mem::replace(
                    self,
                    CritBitNode::Internal(InternalCritBitNode {
                        left: None,
                        right: None,
                        crit: 0,
                    }),
                ) {
                    let crit = (k ^ key).leading_zeros();
                    let _ = std::mem::replace(
                        self,
                        CritBitNode::Internal({
                            let (left, right) = if k < key {
                                (
                                    Some(Box::new(CritBitNode::Leaf(k, v))),
                                    Some(Box::new(CritBitNode::Leaf(key, value))),
                                )
                            } else {
                                (
                                    Some(Box::new(CritBitNode::Leaf(key, value))),
                                    Some(Box::new(CritBitNode::Leaf(k, v))),
                                )
                            };
                            InternalCritBitNode { left, right, crit }
                        }),
                    );
                } else {
                    unreachable!("We just checked that this was a leaf...")
                }
                None
            }
            CritBitNode::Internal(InternalCritBitNode {
                left: Some(ref mut kid),
                right: _,
                ref crit,
            }) if !bit_at(&key, crit) => kid.insert(key, value),
            CritBitNode::Internal(InternalCritBitNode {
                left: _,
                right: Some(ref mut kid),
                ref crit,
            }) if bit_at(&key, crit) => kid.insert(key, value),
            _ => unreachable!(
                "Internal nodes should always have both branches filled, what happened?"
            ),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{CritBit, bit_at};

    #[test]
    fn verify_bit_at() {
        assert!(!bit_at(&1u8, &0u32));
        assert!(bit_at(&128u8, &0u32));
        assert!(bit_at(&1u8, &7u32));
        assert!(!bit_at(&128u8, &7u32));
    }

    #[test]
    fn empty_len() {
        let t: CritBit<u8, ()> = CritBit::new();
        assert_eq!(t.len(), 0);
    }

    #[test]
    fn empty_contains_key() {
        let t: CritBit<u8, ()> = CritBit::new();
        assert!(!t.contains_key(&0u8));
        assert!(!t.contains_key(&128u8));
        assert!(!t.contains_key(&255u8));
    }

    #[test]
    fn empty_get() {
        let t: CritBit<u8, ()> = CritBit::new();
        assert_eq!(t.get(&0u8), None);
        assert_eq!(t.get(&128u8), None);
        assert_eq!(t.get(&255u8), None);
    }

    #[test]
    fn empty_get_mut() {
        let mut t: CritBit<u8, ()> = CritBit::new();
        assert!(t.get_mut(&0u8).is_none());
        assert!(t.get_mut(&128u8).is_none());
        assert!(t.get_mut(&255u8).is_none());
    }

    #[test]
    fn insert_len() {
        let mut t: CritBit<u8, ()> = CritBit::new();
        assert_eq!(t.len(), 0);

        t.insert(0u8, ());
        assert_eq!(t.len(), 1)
    }

    #[test]
    fn insert_contains_key() {
        let mut t: CritBit<u8, ()> = CritBit::new();
        assert!(!t.contains_key(&0u8));

        t.insert(0u8, ());
        assert!(t.contains_key(&0u8));
    }

    #[test]
    fn insert_get() {
        let mut t: CritBit<u8, u8> = CritBit::new();
        assert_eq!(t.get(&0u8), None);

        t.insert(0u8, 1u8);
        assert_eq!(t.get(&0u8), Some(&1u8));
    }

    #[test]
    fn insert_get_mut() {
        let mut t: CritBit<u8, u8> = CritBit::new();
        assert_eq!(t.get(&0u8), None);

        t.insert(0u8, 1u8);
        assert_eq!(t.get(&0u8), Some(&1u8));

        *t.get_mut(&0u8).unwrap() = 2u8;
        assert_eq!(t.get(&0u8), Some(&2u8));
    }

    #[test]
    fn insert_insert() {
        let mut t: CritBit<u8, u8> = CritBit::new();
        assert_eq!(t.get(&0u8), None);

        t.insert(0u8, 1u8);
        assert_eq!(t.get(&0u8), Some(&1u8));

        assert_eq!(t.insert(0u8, 2u8), Some(1u8));
        assert_eq!(t.get(&0u8), Some(&2u8));
    }
}

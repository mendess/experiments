#![feature(return_type_notation)]

use std::{
    iter::zip,
    mem::{size_of_val, MaybeUninit},
    num::NonZeroUsize,
    ops::{Index, IndexMut},
};

#[derive(Debug)]
pub struct RingBuffer<T> {
    data: Box<[MaybeUninit<T>]>,
    write: usize,
    is_full: bool,
}

impl<T> RingBuffer<T> {
    pub fn new(size: NonZeroUsize) -> Self {
        Self {
            data: (0..size.get()).map(|_| MaybeUninit::uninit()).collect(),
            write: 0,
            is_full: false,
        }
    }

    /// # Panics
    /// if the iterator has size 0
    pub fn from_iterator<I: Iterator<Item = T>>(i: I) -> Self {
        Self {
            data: i.map(MaybeUninit::new).collect(),
            write: 0,
            is_full: true,
        }
    }

    pub fn push(&mut self, t: T) {
        if self.is_full {
            // SAFETY: if it's full, then every value has been initialized.
            *unsafe { self.data[self.write].assume_init_mut() } = t;
            self.write += 1;
            if self.write == self.data.len() {
                self.write = 0;
            }
        } else {
            self.data[self.write].write(t);
            self.write += 1;
            if self.write == self.data.len() {
                self.is_full = true;
                self.write = 0;
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let (first, second) = if self.is_full {
            let (second, first) = self.data.split_at(self.write);
            (first, second)
        } else {
            (&self.data[..self.write], &[][..])
        };

        first.iter().chain(second).map(|elem| {
            // SAFETY:
            // if we're full then all values are initialized
            // if we're not full then all values before `self.write` are initialized.
            unsafe { elem.assume_init_ref() }
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        let (first, second) = if self.is_full {
            let (second, first) = self.data.split_at_mut(self.write);
            (first, second)
        } else {
            (&mut self.data[..self.write], &mut [][..])
        };

        first.iter_mut().chain(second).map(|elem| {
            // SAFETY:
            // if we're full then all values are initialized
            // if we're not full then all values before `self.write` are initialized.
            unsafe { elem.assume_init_mut() }
        })
    }

    pub fn len(&self) -> usize {
        if self.is_full {
            self.data.len()
        } else {
            self.write
        }
    }

    pub fn capacity(&self) -> NonZeroUsize {
        // SAFETY
        // Self::new guarantees that this length is always non zero
        unsafe { NonZeroUsize::new_unchecked(self.data.len()) }
    }

    pub fn is_full(&self) -> bool {
        self.is_full
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn first(&self) -> Option<&T> {
        let e = if self.is_full {
            &self.data[self.write]
        } else if self.write == 0 {
            return None;
        } else {
            &self.data[0]
        };
        // SAFETY:
        // If it's full all elements are initialized
        // If it's not full all elements behind `self.write` are initialized
        Some(unsafe { e.assume_init_ref() })
    }

    pub fn last(&self) -> Option<&T> {
        let e = if self.is_full {
            &self.data[self.write.checked_sub(1).unwrap_or(self.data.len() - 1)]
        } else {
            &self.data[self.write.checked_sub(1)?]
        };
        // SAFETY:
        // If it's full all elements are initialized
        // If it's not full all elements behind `self.write` are initialized
        Some(unsafe { e.assume_init_ref() })
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        self.iter().nth(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.iter_mut().nth(idx)
    }

    pub fn clear(&mut self) {
        let to_delete = if self.is_full {
            self.data.iter_mut()
        } else {
            self.data[0..self.write].iter_mut()
        };
        to_delete.for_each(|e| {
            // SAFETY:
            // If it's full all elements are initialized
            unsafe { e.assume_init_drop() }
        })
    }
}

impl<T> IntoIterator for RingBuffer<T> {
    type Item = T;
    type IntoIter = RingBuffer<T>::iter(..);
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: PartialEq> PartialEq for RingBuffer<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && zip(self.iter(), other.iter()).all(|(a, b)| a == b)
    }
}

impl<T: Eq> Eq for RingBuffer<T> {}

impl<T> Index<usize> for RingBuffer<T> {
    type Output = T;
    fn index(&self, idx: usize) -> &Self::Output {
        self.get(idx).unwrap()
    }
}

impl<T> IndexMut<usize> for RingBuffer<T> {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        self.get_mut(idx).unwrap()
    }
}

impl<T: Clone> Clone for RingBuffer<T> {
    fn clone(&self) -> Self {
        Self::from_iterator(self.iter().cloned())
    }

    fn clone_from(&mut self, source: &Self) {
        if self.capacity().get() >= source.len() {
            let (initialized, uninit) = if self.is_full {
                (&mut self.data[..], &mut [][..])
            } else {
                self.data.split_at_mut(self.write)
            };
            let common_len = usize::min(initialized.len(), source.len());
            let mut source = source.iter();
            for (e, s) in zip(
                initialized[..common_len].iter_mut(),
                source.by_ref().take(common_len),
            ) {
                *unsafe { e.assume_init_mut() } = s.clone();
            }
            for e in initialized {
                unsafe { e.assume_init_drop() };
            }
            for (s, u) in zip(source, uninit) {
                u.write(s.clone());
            }
        } else {
            *self = source.clone()
        }
    }
}

impl<T> Drop for RingBuffer<T> {
    fn drop(&mut self) {
        self.clear();
    }
}

fn main() {
    let mut ring = RingBuffer::new(3.try_into().unwrap());
    println!("ring: {:?}", ring.iter().collect::<Vec<_>>());
    for c in ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'] {
        ring.push(c);
        println!("ring: {:?}", ring.iter().collect::<Vec<_>>());
    }

    println!("sizeof iterator: {}", size_of_val(&ring.iter()));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn iterator_test() {
        let mut ring = RingBuffer::new(3.try_into().unwrap());
        println!("ring: {:?}", ring.iter().collect::<Vec<_>>());
        let values = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
        let expect = [
            vec!['a'],
            vec!['a', 'b'],
            vec!['a', 'b', 'c'],
            vec!['b', 'c', 'd'],
            vec!['c', 'd', 'e'],
            vec!['d', 'e', 'f'],
            vec!['e', 'f', 'g'],
            vec!['f', 'g', 'h'],
        ];
        for (c, expect) in zip(values, expect) {
            ring.push(c);
            assert_eq!(ring.iter().copied().collect::<Vec<_>>(), expect,);
        }
    }
}

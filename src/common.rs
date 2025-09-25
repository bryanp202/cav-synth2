use std::{mem::MaybeUninit, ops::{Deref, DerefMut}};


/// Does not work well with type T that require drop behavior
pub struct ComponentVec<T, const MAX: usize> {
    components: [MaybeUninit<T>; MAX],
    count: usize,
}

impl <T, const MAX: usize> ComponentVec<T, MAX> {
    pub fn new() -> Self {
        Self {
            components: [const { MaybeUninit::uninit() }; MAX],
            count: 0,
        }
    }

    pub fn push(&mut self, new_component: T) -> Result<(), ()> {
        if self.count == MAX {
            return Err(());
        }
        self.components[self.count].write(new_component);
        self.count += 1;
        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> Result<T, ()> {
        if index >= self.count {
            return Err(());
        }
        let removed = unsafe { self.components[index].assume_init_read() };
        if index + 1 != self.count {
            self.count -= 1;
            self.components.swap(index, self.count);
        }
        Ok(removed)
    }

    pub fn iter(&self) -> ComponentVecIter<T, MAX> {
        ComponentVecIter::new(self)
    }
}

impl <T, const MAX: usize> Deref for ComponentVec<T, MAX> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { &self.components[0..self.count].assume_init_ref() }
    }
}

impl <T, const MAX: usize> DerefMut for ComponentVec<T, MAX> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { (&mut self.components[0..self.count]).assume_init_mut() }
    }
}

pub struct ComponentVecIter<'a, T, const MAX: usize> {
    current: usize,
    count: usize,
    source: &'a ComponentVec<T, MAX>,
}

impl <'a, T, const MAX: usize> ComponentVecIter<'a, T, MAX> {
    pub fn new(source: &'a ComponentVec<T, MAX>) -> Self {
        Self {
            current: 0,
            count: source.count,
            source,
        }
    }
}

impl <'a, T, const MAX: usize> Iterator for ComponentVecIter<'a, T, MAX> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current != self.count {
            let item = &self.source[self.current];
            self.current += 1;
            Some(item)
        } else {
            None
        }
    }
}


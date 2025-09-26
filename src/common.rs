use std::{mem::MaybeUninit, ops::{Deref, DerefMut}};

use sdl3::render::{FPoint, FRect};


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

    #[allow(dead_code)]
    pub fn remove(&mut self, index: usize) -> T {
        if index >= self.count {
            panic!("Out of bounds remove");
        }
        let removed = unsafe { self.components[index].assume_init_read() };
        self.count -= 1;
        if index != self.count {
            self.components.swap(index, self.count);
        }
        removed
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        (self as &[T]).iter()
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

pub fn point_in_frect(rect: &FRect, x: f32, y: f32) -> bool {
    x >= rect.x && x <= rect.x + rect.w && y >= rect.y && y <= rect.y + rect.h
}

pub fn frect_center(rect: &FRect) -> FPoint {
    FPoint::new(rect.x + rect.w / 2.0, rect.y + rect.h / 2.0)
}
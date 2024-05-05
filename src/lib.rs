#![feature(vec_into_raw_parts)]

use std::fmt::Display;
use std::ops::Range;

const GAP_SIZE: usize = 2;

pub struct GapBuffer<T: Display> {
    buffer: Vec<T>,
    gap: Range<usize>,
}

impl<T: Display> GapBuffer<T> {
    pub fn new(data: Vec<T>) -> Self {
        let mut buffer: Vec<T> = Vec::with_capacity(data.len() + GAP_SIZE);
        let gap = 0..GAP_SIZE;

        // APPROACH #1:
        // We have to pop all the elements, wrap them in ManuallyDrop
        // and write them from the back of the new Vec<ManaullyDrop<T>>.
        //
        // let i = buffer.capacity() - 1;
        // for e in data {
        //     // ManuallyDrop<T> is guaranteed to have the same layout and bit validity as T.
        //     let e = ManuallyDrop::new(e);
        //     unsafe { std::ptr::write(buffer.as_mut_ptr().add(i), &e) };
        //     i -= 1;
        // }

        // APPROACH #2:
        // Use into_raw_parts and copy to the new vector non-verlapping
        // and then reconstruct a Vec with same capacity but 0 len.
        let (data_ptr, data_len, data_cap) = data.into_raw_parts();

        unsafe {
            std::ptr::copy_nonoverlapping(data_ptr, buffer.as_mut_ptr().add(gap.end), data_len);
            // Construct the vector with length 0 so that it only drops
            // its own memory and not that of its elements.
            let _ = Vec::from_raw_parts(data_ptr, 0, data_cap);
        };

        Self { buffer, gap }
    }

    pub fn get_ref_pos(&self, idx: usize) -> &T {
        let idx = self.translate_idx(idx);
        unsafe { &*self.buffer.as_ptr().add(idx) }
    }

    pub fn set_position(&mut self, idx: usize) {
        if idx > self.data_len() {
            panic!("index out of bounds");
        }

        let src: *const T;
        let dst: *mut T;
        let count;

        if idx < self.gap.start {
            unsafe {
                src = self.buffer.as_ptr().add(idx);
                dst = self.buffer.as_mut_ptr().add(idx + self.gap.len());
            }
            count = self.gap.start - idx;
        } else {
            unsafe {
                src = self.buffer.as_ptr().add(self.gap.end);
                dst = self.buffer.as_mut_ptr().add(self.gap.start);
            }
            count = self.translate_idx(idx) - self.gap.end;
        }

        self.gap = idx..idx + self.gap.len();
        unsafe { std::ptr::copy(src, dst, count) };
    }

    /// Removes the element at gap.end. Given the following buffer
    /// where pos = 4.
    ///
    /// +-------------------------+
    /// | 0 | 1 | 2 | 3 | Gap | 4 |
    /// +-------------------------+
    ///                         |
    ///                       remove
    ///
    /// Issuing a remove will result in:
    ///
    /// +---------------------+
    /// | 0 | 1 | 2 | 3 | Gap |
    /// +---------------------+
    pub fn remove(&mut self) {
        if self.gap.end == self.buffer.capacity() {
            return;
        }

        unsafe {
            std::ptr::read(self.buffer.as_ptr().add(self.gap.end));
        };
        self.gap.end += 1;
    }

    pub fn insert_iter<I: IntoIterator<Item = T>>(&mut self, elems: I) {
        for e in elems.into_iter() {
            self.insert(e);
        }
    }

    pub fn insert(&mut self, elem: T) {
        if self.gap.len() == 0 {
            self.enlarge();
        }

        unsafe { std::ptr::write(self.buffer.as_mut_ptr().add(self.gap.start), elem) };
        self.gap.start += 1;
    }

    fn enlarge(&mut self) {
        let mut new_buffer = Vec::with_capacity(self.buffer.capacity() * 2);

        unsafe {
            // Copy all the elements in the old bufffer which reside before gap.start.
            std::ptr::copy_nonoverlapping(
                self.buffer.as_ptr(),
                new_buffer.as_mut_ptr(),
                self.gap.start,
            );

            // Copy all elements from gap.start to the end in the old buffer
            // to the region after the gap in the new buffer.
            std::ptr::copy_nonoverlapping(
                self.buffer.as_ptr().add(self.gap.start),
                new_buffer
                    .as_mut_ptr()
                    .add(self.gap.start + self.buffer.capacity()),
                self.buffer.capacity() - self.gap.start,
            );
        }

        self.gap.end = self.gap.start + self.buffer.capacity();
        // Old buffer is dropped but since its len is 0 none of the elements
        // will actually get dropped.
        self.buffer = new_buffer;
    }

    fn translate_idx(&self, idx: usize) -> usize {
        if idx < self.gap.start {
            idx
        } else {
            idx + self.gap.len()
        }
    }

    pub fn len(&self) -> usize {
        self.data_len()
    }

    fn data_len(&self) -> usize {
        self.buffer.capacity() - self.gap.len()
    }
}

impl<T: Display> Drop for GapBuffer<T> {
    fn drop(&mut self) {
        for i in 0..self.gap.start {
            unsafe { println!("dropping {}", &*self.buffer.as_ptr().add(i)) };
            unsafe { std::ptr::drop_in_place(self.buffer.as_mut_ptr().add(i)) };
        }
        for i in self.gap.end..self.buffer.capacity() {
            unsafe { println!("dropping {}", &*self.buffer.as_ptr().add(i)) };
            unsafe { std::ptr::drop_in_place(self.buffer.as_mut_ptr().add(i)) };
        }
    }
}

impl<T: Display> Display for GapBuffer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ ")?;
        for i in 0..self.gap.start {
            let elem: &T = unsafe { &*self.buffer.as_ptr().add(i) };
            write!(f, "{}, ", elem)?;
        }
        write!(f, "Gap[{}, {}), ", self.gap.start, self.gap.end)?;
        for i in self.gap.end..self.buffer.capacity() {
            let elem: &T = unsafe { &*self.buffer.as_ptr().add(i) };
            write!(f, "{}, ", elem)?;
        }
        write!(f, "]")
    }
}

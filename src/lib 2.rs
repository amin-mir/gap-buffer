use std::{fmt::Display, mem::ManuallyDrop, ops::Range};

const GAP_SIZE: usize = 16;

pub struct GapBuffer<T: Display> {
    buffer: Vec<T>,
    gap: Range<usize>,
}

impl<T: Display> GapBuffer<T> {
    // Constructs a new GapBuffer by copying bytes into a new buffer.
    pub fn new(data: Vec<T>) -> Self {
        let mut buffer: Vec<T> = Vec::with_capacity(data.len() + 16);
        let gap = 0..GAP_SIZE;

        // APPROACH #1:
        // We have to pop all the elements, wrap them in ManuallyDrop
        // and write them from the back of the new Vec.
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
            // Construct the vector with length 0 so that it only drops
            // its own memory and not that of its elements.
            let _ = Vec::from_raw_parts(data_ptr, 0, data_cap)
        }

        unsafe {  };

        unsafe { std::ptr::copy(data.as_ptr(), buffer.as_mut_ptr().add(gap.end), data.len()) };

        Self { buffer, gap }
    }

    pub fn get_pos(&self, idx: usize) -> &T {
        let idx = self.translate_idx(idx);
        unsafe { &*self.buffer.as_ptr().add(idx) }
    }

    pub fn get(&mut self) -> T {
        let idx = self.gap.end;
        self.gap.end += 1;
        unsafe { std::ptr::read(self.buffer.as_ptr().add(idx)) }
    }

    pub fn set_pos(&mut self, idx: usize) {
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

    fn translate_idx(&self, idx: usize) -> usize {
        if idx < self.gap.start {
            idx
        } else {
            idx + self.gap.len()
        }
    }

    // fn data_len(&self) -> usize {
    //     self.data.len() - self.gap.len()
    // }
}

// impl<T> Drop for GapBuffer<T> {
//     fn drop(&mut self) {
//         for i in 0..self.gap.start {
//             unsafe { std::ptr::drop_in_place(self.data.as_mut_ptr().add(i)) };
//         }
//         for i in self.gap.end..self.data.len() {
//             unsafe { std::ptr::drop_in_place(self.data.as_mut_ptr().add(i)) };
//         }
//     }
// }

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
        write!(f, "[")?;
        for i in 0..self.gap.start {
            let elem: &T = unsafe { &*self.buffer.as_ptr().add(i) };
            write!(f, "{}, ", elem)?;
        }
        write!(f, "-- Gap[{}, {}) -- ", self.gap.start, self.gap.end)?;
        // println!(
        //     "gap end: {}, data len: {}",
        //     self.gap.end,
        //     self.data.capacity()
        // );
        for i in self.gap.end..self.buffer.capacity() {
            let elem: &T = unsafe { &*self.buffer.as_ptr().add(i) };
            write!(f, "{}, ", elem)?;
        }
        Ok(())
    }
}

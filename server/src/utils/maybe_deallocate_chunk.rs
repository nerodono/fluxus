use std::{
    mem::{
        self,
        MaybeUninit,
    },
    slice,
};

pub struct MaybeDeallocateChunk<'a> {
    leaked: &'a mut [MaybeUninit<u8>],
    deallocate: bool,
}

impl<'a> MaybeDeallocateChunk<'a> {
    pub fn as_ptr(&self) -> *const MaybeUninit<u8> {
        self.leaked.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut MaybeUninit<u8> {
        self.leaked.as_mut_ptr()
    }

    pub fn data(&mut self) -> &mut [MaybeUninit<u8>] {
        self.leaked
    }

    pub unsafe fn data_initialized(&self, of_size: usize) -> &[u8] {
        debug_assert!(self.len() >= of_size);
        slice::from_raw_parts(self.leaked.as_ptr().cast(), of_size)
    }

    pub unsafe fn full_data_initialized(&self) -> &[u8] {
        slice::from_raw_parts(self.leaked.as_ptr().cast(), self.len())
    }

    pub fn len(&self) -> usize {
        self.leaked.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a> MaybeDeallocateChunk<'a> {
    pub fn new_sufficiency_clamped(
        slice: &'a mut [MaybeUninit<u8>],
        sufficient: usize,
    ) -> Self {
        if slice.len() >= sufficient {
            Self::new(unsafe {
                slice::from_raw_parts_mut(slice.as_mut_ptr(), sufficient)
            })
        } else {
            Self::new_deallocated(Vec::with_capacity(sufficient), sufficient)
        }
    }

    /// Simple pattern: allocates only if slice has
    /// insufficient space
    pub fn new_sufficiency(
        slice: &'a mut [MaybeUninit<u8>],
        sufficient: usize,
    ) -> Self {
        if slice.len() >= sufficient {
            Self::new(slice)
        } else {
            Self::new_deallocated(Vec::with_capacity(sufficient), sufficient)
        }
    }

    pub fn new(slice: &'a mut [MaybeUninit<u8>]) -> Self {
        Self {
            leaked: slice,
            deallocate: false,
        }
    }

    pub fn new_deallocated(mut vec: Vec<u8>, exact_capacity: usize) -> Self {
        assert!(vec.capacity() >= exact_capacity);

        unsafe { vec.set_len(0) };
        let ptr = vec.as_mut_ptr();
        let cap = exact_capacity;

        mem::forget(vec);

        Self {
            leaked: unsafe { slice::from_raw_parts_mut(ptr.cast(), cap) },
            deallocate: true,
        }
    }
}

impl<'a> Drop for MaybeDeallocateChunk<'a> {
    fn drop(&mut self) {
        if self.deallocate {
            _ = unsafe {
                Vec::from_raw_parts(
                    self.leaked.as_mut_ptr(),
                    0,
                    self.leaked.len(),
                )
            }
        }
    }
}

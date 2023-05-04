use std::{
    mem::{
        self,
        MaybeUninit,
    },
    slice,
};

pub struct MaybeHeapChunk<'a> {
    inner: &'a mut [MaybeUninit<u8>],
    heap: bool,
}

impl<'a> MaybeHeapChunk<'a> {
    /// # Safety
    ///
    /// Unsafe due to ability of reading an uninitialized
    /// data
    pub const unsafe fn data_initialized(&self, len: usize) -> &[u8] {
        debug_assert!(len <= self.inner.len());
        slice::from_raw_parts(self.inner.as_ptr().cast(), len)
    }

    /// # Safety
    ///
    /// Same as [`MaybeHeapChunk::data_initialized`]
    pub unsafe fn data_initialized_mut(&mut self, len: usize) -> &mut [u8] {
        debug_assert!(len <= self.inner.len());
        slice::from_raw_parts_mut(self.inner.as_mut_ptr().cast(), len)
    }

    pub const fn data(&self) -> &[MaybeUninit<u8>] {
        &*self.inner
    }

    pub fn data_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self.inner
    }

    pub fn stack_uninit(inner: &'a mut [MaybeUninit<u8>]) -> Self {
        Self { inner, heap: false }
    }

    pub fn heap(mut buf: Vec<u8>) -> Self {
        let capacity = buf.capacity();
        let ptr = buf.as_mut_ptr();

        mem::forget(buf);

        Self {
            inner: unsafe {
                slice::from_raw_parts_mut::<'a, _>(
                    ptr.cast::<MaybeUninit<u8>>(),
                    capacity,
                )
            },
            heap: true,
        }
    }
}

impl<'a> Drop for MaybeHeapChunk<'a> {
    fn drop(&mut self) {
        if self.heap {
            // Deallocate it
            let _vec = unsafe {
                Vec::from_raw_parts(
                    self.inner.as_mut_ptr(),
                    0,
                    self.inner.len(),
                )
            };
        }
    }
}

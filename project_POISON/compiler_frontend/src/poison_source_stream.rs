pub struct SourceStream<'a> {
    _marker: std::marker::PhantomData<&'a [u8]>,
    pub initial_ptr: *const u8,
    pub index: usize,
    pub length: usize,
}
impl<'a> SourceStream<'a> {
    #[inline(always)]
    pub fn new(source: &'a [u8]) -> Self {
        let start = source.as_ptr();
        Self {
            _marker: std::marker::PhantomData,
            initial_ptr: start,
            index: 0,
            length: source.len(),
        }
    }
    #[inline(always)]
    pub unsafe fn get_slice(&self, start: usize, end: usize) -> &'a [u8] {
        debug_assert!(start <= end && end <= self.length);
        std::slice::from_raw_parts(self.initial_ptr.add(start), end - start)
    }
    #[inline(always)]
    pub fn peek(&self) -> Option<u8>{
        if self.index >= self.length {
            None
        } else {
            unsafe { Some(*self.initial_ptr.add(self.index)) }
        }
    }

    #[inline(always)]
    pub fn peek_at(&self, offset: usize) -> Option<u8> {
        let target = self.index + offset;
        if target < self.length {
            unsafe { Some(*self.initial_ptr.add(target)) }
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn index(&self) -> usize {
        self.index
    }

    #[inline(always)]
    pub fn advance(&mut self) -> Option<u8>{
        if self.index >= self.length {
            None
        } else {
            unsafe {
                let byte = *self.initial_ptr.add(self.index);
                self.index += 1;
                Some(byte)
            }
        }
    }
    #[inline(always)]
    pub fn skip_whitespace(&mut self) {
        let base = self.initial_ptr;
        let len = self.length;
        let mut idx = self.index;

        while idx < len {
            unsafe {
                let b = *base.add(idx);
                if b < 33 && ((1u64 << b) & 0x100002600) != 0 {
                    idx += 1;
                } else {
                    break;
                }
            }
        }
        self.index = idx;
    }
}
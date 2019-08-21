use super::*;
use util::constants::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use util::*;
use vm::*;

const BITS_IN_MARK_TABLE: usize = BYTES_IN_REGION / BYTES_IN_WORD;
const MARK_TABLE_SIZE: usize = BITS_IN_MARK_TABLE / BITS_IN_BYTE;

pub struct MarkTable {
    data: Box<[AtomicUsize; MARK_TABLE_SIZE]>,
}

impl MarkTable {
    pub fn new() -> Self {
        Self {
            data: unsafe { ::std::mem::transmute(box [0usize; MARK_TABLE_SIZE]) },
        }
    }

    fn get_entry_for_address(&self, addr: Address) -> (usize, usize) {
        debug_assert!(!addr.is_zero());
        let diff = addr - Region::align(addr);
        let index = diff >> LOG_BITS_IN_WORD;
        let offset = diff & (BITS_IN_WORD - 1);
        (index, offset)
    }

    fn get_entry(&self, obj: ObjectReference) -> (usize, usize) {
        debug_assert!(!obj.is_null());
        let addr = VMObjectModel::object_start_ref(obj);
        self.get_entry_for_address(addr)
    }

    pub fn mark(&self, obj: ObjectReference, atomic: bool) -> bool {
        let (index, offset) = self.get_entry(obj);
        let entry = &self.data[index];
        let mask = 1 << offset;
        if atomic {
            let old_value = entry.fetch_or(mask, Ordering::SeqCst);
            (old_value & mask) == 0
        } else {
            let value = entry.load(Ordering::SeqCst);
            if (value & mask) != 0 {
                return true
            }
            entry.store(value | mask, Ordering::SeqCst);
            true
        }
    }

    fn test(&self, a: Address) -> bool {
        let (index, offset) = self.get_entry_for_address(a);
        let entry = &self.data[index];
        let mask = 1 << offset;
        let value = entry.load(Ordering::SeqCst);
        (value & mask) != 0
    }

    pub fn is_marked(&self, o: ObjectReference) -> bool {
        self.test(VMObjectModel::object_start_ref(o))
    }
    
    #[inline(always)]
    pub fn iterate<F: Fn(ObjectReference)>(&self, start: Address, end: Address, f: F) {
        let mut cursor = start;
        while cursor < end {
            if self.test(cursor) {
                let object = unsafe { VMObjectModel::get_object_from_start_address(cursor) };
                debug_assert!(VMObjectModel::object_start_ref(object) == cursor);
                f(object);
            }
            cursor = cursor + BYTES_IN_ADDRESS;
        }
    }
}

impl ::std::fmt::Debug for MarkTable {
    fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        writeln!(formatter, "<marktable>")
    }
}
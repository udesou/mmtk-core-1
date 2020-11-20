use crate::util::ObjectReference;
use crate::vm::ObjectModel;
use crate::vm::VMBinding;
use std::sync::atomic::AtomicU8;

/// Return the GC byte of an object.
///
/// MMTk requires *exactly one byte* for each object as per-object metadata.
///
/// If the client VM provides the one byte in its object headers
/// (see [trait ObjectModel](crate::vm::ObjectModel)),
/// MMTk uses that byte as the per-object metadata.
/// Otherwise, MMTk provides the metadata on its side.
///
pub fn get_gc_byte<VM: VMBinding>(object: ObjectReference) -> &'static AtomicU8 {
    if VM::VMObjectModel::HAS_GC_BYTE {
        unsafe {
            let res = &*(object.to_address() + VM::VMObjectModel::GC_BYTE_OFFSET).to_ptr::<AtomicU8>();
            res
        }
    } else {
        todo!("\"HAS_GC_BYTE == false\" is not supported yet")
    }
}

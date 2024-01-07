use std::sync::atomic::AtomicU32;

/// Counter that uses atomic u32 internally. Used for
/// generation of unique identifiers.
struct StaticCounter(AtomicU32);
impl StaticCounter {
    pub const fn create() -> Self {
        Self(AtomicU32::new(0))
    }
    pub fn next(&self) -> u32 {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}

macro_rules! impl_basic_id {
    ($name:ident) => {
        impl $name {
            const COUNTER: StaticCounter = StaticCounter::create();

            pub fn new(id: u32) -> Self {
                Self(id)
            }

            pub fn new_unique() -> Self {
                Self(Self::COUNTER.next())
            }
        }
    };
}

/// ID of package, this is unique for every crate during compilation,
/// but may change between compilations, so It shouldn't be used
/// between them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PkgID(pub u32);
impl_basic_id!(PkgID);

/// ID of source file, this is generated as unique for every
/// source file in the current compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceFileID(pub u32);
impl_basic_id!(SourceFileID);


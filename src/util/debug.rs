use std::fmt::{Debug, Formatter};

pub struct LiteralDebug(pub &'static str);

impl Debug for LiteralDebug {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

pub struct PointerDebug {
    value: *const (),
}

impl PointerDebug {
    pub fn new<T>(value: *const T) -> Self {
        Self {
            value: value as *const (),
        }
    }
}

impl Debug for PointerDebug {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:p}", self.value))
    }
}

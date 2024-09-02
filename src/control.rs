use raylib::{RaylibHandle, RaylibThread};

pub enum ControlResult {
    Passthrough,
    Block,
}

pub struct ControlContext<'a, 'b> {
    pub thread: &'a RaylibThread,
    pub rl: &'b mut RaylibHandle,

    pub is_dirty: bool,
}

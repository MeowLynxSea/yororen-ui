use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClipboardOp {
    Copy,
}

#[derive(Clone, Debug)]
pub struct FileClipboard {
    pub op: ClipboardOp,
    pub src: PathBuf,
}


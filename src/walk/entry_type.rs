#[derive(Debug, Eq, PartialEq, Clone)]
pub enum EntryType {
    Dir,
    File,
    StdIn,
    Symlink,
    Socket,
    BlockDevice,
    CharDevice,
    FIFO,
    Unknown
}

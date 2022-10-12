use std::ffi::OsStr;
use std::fs::Permissions;
use std::os::unix::prelude::{FileTypeExt, MetadataExt};
use std::path::Path;
use std::time::SystemTime;

use ignore::DirEntry;

use crate::errors::GenericError;
use crate::walk::entry_type::EntryType;
use crate::walk::traits::DirEntryWrapperExt;

pub mod entry_type;
pub mod traits;

impl DirEntryWrapperExt for DirEntry {
    fn get_entry_type(&self) -> EntryType {
        let path = self.path();
        if path.is_dir() {
            EntryType::Dir
        } else if path.is_file() {
            EntryType::File
        } else if path.is_symlink() {
            EntryType::Symlink
        } else if self.is_stdin() {
            EntryType::StdIn
        } else {
            match self.file_type() {
                None => EntryType::Unknown,
                Some(ft) if ft.is_socket() => EntryType::Socket,
                Some(ft) if ft.is_block_device() => EntryType::BlockDevice,
                Some(ft) if ft.is_char_device() => EntryType::CharDevice,
                Some(ft) if ft.is_fifo() => EntryType::FIFO,
                Some(_) => EntryType::Unknown,
            }
        }
    }

    fn get_name(&self) -> &OsStr {
        self.file_name()
    }

    fn get_path(&self) -> &Path {
        self.path()
    }

    fn get_size(&self) -> usize {
        self.path().metadata().map(|metadata| metadata.len() as usize).unwrap_or(0)
    }

    fn get_depth(&self) -> usize {
        self.depth()
    }

    fn get_mtime(&self) -> Result<SystemTime, GenericError> {
        Ok(self.path().metadata()?.modified()?)
    }

    fn get_atime(&self) -> Result<SystemTime, GenericError> {
        Ok(self.path().metadata()?.accessed()?)
    }

    fn get_btime(&self) -> Result<SystemTime, GenericError> {
        Ok(self.path().metadata()?.created()?)
    }

    fn get_user_id(&self) -> Result<u32, GenericError> {
        Ok(self.path().metadata()?.uid())
    }

    fn get_group_id(&self) -> Result<u32, GenericError> {
        Ok(self.path().metadata()?.gid())
    }

    fn get_permissions(&self) -> Result<Permissions, GenericError> {
        Ok(self.path().metadata()?.permissions())
    }
}

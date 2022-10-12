#![allow(dead_code)]

use std::ffi::OsStr;
use std::fs::Permissions;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::GenericError;
use crate::walk::entry_type::EntryType;
use crate::walk::traits::DirEntryWrapperExt;

#[derive(Default)]
pub(crate) struct DirEntryMock {
   pub(crate) entry_type: Option<EntryType>,
   pub(crate) file: Option<PathBuf>,
   pub(crate) size: Option<usize>,
   pub(crate) depth: Option<usize>,

   pub(crate) user_id: Option<u32>,
   pub(crate) group_id: Option<u32>,

   pub(crate) mtime: Option<SystemTime>,
   pub(crate) atime: Option<SystemTime>,
   pub(crate) btime: Option<SystemTime>,

   pub(crate) permissions: Option<Permissions>
}

impl DirEntryMock {
    pub(crate) fn set_entry_type(mut self, entry_type: EntryType) -> Self {
        self.entry_type = entry_type.into();
        self
    }
    pub(crate) fn set_file(mut self, file: PathBuf) -> Self {
        self.file = file.into();
        self
    }
    pub(crate) fn set_size(mut self, size: usize) -> Self {
        self.size = size.into();
        self
    }
    pub(crate) fn set_depth(mut self, depth: usize) -> Self {
        self.depth = depth.into();
        self
    }
    pub(crate) fn set_user_id(mut self, user_id: u32) -> Self {
        self.user_id = user_id.into();
        self
    }
    pub(crate) fn set_group_id(mut self, group_id: u32) -> Self {
        self.group_id = group_id.into();
        self
    }
    pub(crate) fn set_mtime(mut self, mtime: SystemTime) -> Self {
        self.mtime = mtime.into();
        self
    }
    pub(crate) fn set_atime(mut self, atime: SystemTime) -> Self {
        self.atime = atime.into();
        self
    }
    pub(crate) fn set_btime(mut self, btime: SystemTime) -> Self {
        self.btime = btime.into();
        self
    }
    pub(crate) fn set_permissions(mut self, permissions: Permissions) -> Self {
        self.permissions = permissions.into();
        self
    }
}

impl DirEntryWrapperExt for DirEntryMock {
    fn get_entry_type(&self) -> EntryType {
        self.entry_type.as_ref().unwrap().clone()
    }

    fn get_name(&self) -> &OsStr {
        self.file.as_ref().unwrap().file_name().unwrap()
    }

    fn get_path(&self) -> &Path {
        self.file.as_ref().unwrap().as_path()
    }

    fn get_size(&self) -> usize {
        self.size.unwrap_or(0)
    }

    fn get_depth(&self) -> usize {
        self.depth.unwrap_or(0)
    }

    fn get_mtime(&self) -> Result<SystemTime, GenericError> {
        if let Some(time) = self.mtime {
            Ok(time)
        } else {
            Err(GenericError::UnknownCommand("sample".to_string()))
        }
    }

    fn get_atime(&self) -> Result<SystemTime, GenericError> {
        if let Some(time) = self.atime {
            Ok(time)
        } else {
            Err(GenericError::UnknownCommand("sample".to_string()))
        }
    }

    fn get_btime(&self) -> Result<SystemTime, GenericError> {
        if let Some(time) = self.btime {
            Ok(time)
        } else {
            Err(GenericError::UnknownCommand("sample".to_string()))
        }
    }

    fn get_user_id(&self) -> Result<u32, GenericError> {
        if let Some(user_id) = self.user_id {
            Ok(user_id)
        } else {
            Err(GenericError::UnknownCommand("sample".to_string()))
        }
    }

    fn get_group_id(&self) -> Result<u32, GenericError> {
        if let Some(group_id) = self.group_id {
            Ok(group_id)
        } else {
            Err(GenericError::UnknownCommand("sample".to_string()))
        }
    }

    fn get_permissions(&self) -> Result<Permissions, GenericError> {
        if let Some(ref permissions) = self.permissions {
            Ok(permissions.clone())
        } else {
            Err(GenericError::UnknownCommand("sample".to_string()))
        }
    }


}

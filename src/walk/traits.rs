use std::ffi::OsStr;
use std::fs::Permissions;
use std::path::Path;
use std::time::SystemTime;

use crate::errors::GenericError;
use crate::walk::entry_type::EntryType;

pub trait DirEntryWrapperExt {
    fn get_entry_type(&self) -> EntryType;
    fn get_name(&self) -> &OsStr;
    fn get_path(&self) -> &Path;
    fn get_size(&self) -> usize;
    fn get_depth(&self) -> usize;

    fn get_mtime(&self) -> Result<SystemTime, GenericError>;
    fn get_atime(&self) -> Result<SystemTime, GenericError>;
    fn get_btime(&self) -> Result<SystemTime, GenericError>;

    fn get_user_id(&self) -> Result<u32, GenericError>;
    fn get_group_id(&self) -> Result<u32, GenericError>;
    fn get_permissions(&self) -> Result<Permissions, GenericError>;
}

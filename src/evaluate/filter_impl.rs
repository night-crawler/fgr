use std::fs::OpenOptions;
use std::io::{BufRead, BufReader};
use std::os::unix::prelude::PermissionsExt;

use crate::errors::GenericError;
use crate::evaluate::traits::DurationOffsetExt;
use crate::evaluate::NOW;
use crate::parse::comparison::Comparison;
use crate::parse::file_type::FileType;
use crate::parse::filter::Filter;
use crate::walk::entry_type::EntryType;
use crate::walk::traits::DirEntryWrapperExt;
use crate::Evaluate;

impl<E: DirEntryWrapperExt> Evaluate<E> for Filter {
    fn evaluate(&self, entry: &E) -> Result<bool, GenericError> {
        match self {
            Self::Size { value, comparison } => {
                if entry.get_entry_type() != EntryType::File {
                    return Err(GenericError::NotAFile(entry.get_path().to_path_buf()));
                }
                Ok(comparison.evaluate(entry.get_size(), *value))
            }
            Self::Depth { value, comparison } => {
                Ok(comparison.evaluate(entry.get_depth(), *value))
            }
            Self::Type { value, comparison } => {
                if entry.get_entry_type() != EntryType::File {
                    return Ok(false);
                }

                let file_type: FileType =
                    if let Some(file_type) = infer::get_from_path(entry.get_path())? {
                        file_type.matcher_type()
                    } else {
                        return Ok(false);
                    }
                    .into();

                let mut result = &file_type == value;
                if comparison != &Comparison::Eq {
                    result = !result;
                }

                Ok(result)
            }
            Self::AccessTime { value, comparison } => {
                let file_atime = entry.get_atime()?;
                let user_time = value.add_to(*NOW);

                Ok(comparison.evaluate(file_atime, user_time))
            }
            Self::ModificationTime { value, comparison } => {
                let file_mtime = entry.get_mtime()?;
                let user_time = value.add_to(*NOW);

                Ok(comparison.evaluate(file_mtime, user_time))
            }
            Self::Name { value, comparison } => {
                let is_match = value.is_match(entry.get_name().to_string_lossy());

                Ok(comparison.evaluate(is_match, true))
            }
            Self::Extension { value, comparison } => {
                let name = entry.get_name().to_string_lossy();
                let name_str = name.as_ref();
                let extension = if let Some((_, extension)) = name_str.split_once('.') {
                    extension
                } else {
                    return Ok(false);
                };

                Ok(comparison.evaluate(value.is_match(extension), true))
            }
            Self::Contains { value, comparison } => {
                if entry.get_entry_type() != EntryType::File {
                    return Ok(false);
                }
                let file = OpenOptions::new().read(true).open(entry.get_path())?;

                let reader = BufReader::new(file);
                let result = reader.lines().flatten().any(|line| value.is_match(&line));

                Ok(comparison.evaluate(result, true))
            }
            Self::User { value, comparison } => {
                Ok(comparison.evaluate(entry.get_user_id()?, *value))
            }
            Self::Group { value, comparison } => {
                Ok(comparison.evaluate(entry.get_group_id()?, *value))
            }
            Self::Permissions { value, comparison } => {
                let msb = 32 - value.mode().leading_zeros();
                let mask = (1 << msb) - 1;

                let file_permissions = entry.get_permissions()?;
                Ok(comparison
                    .evaluate(file_permissions.mode() & mask, value.mode() & mask))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::ops::Add;
    use std::os::linux::fs::MetadataExt;
    use std::path::PathBuf;

    use chrono::Duration;

    use crate::parse::comparison::Comparison;
    use crate::parse::file_type::FileType;
    use crate::parse::filter::Filter;
    use crate::test_utils::DirEntryMock;
    use crate::walk::entry_type::EntryType;
    use crate::Evaluate;

    #[test]
    fn test_name() {
        let glob = globset::Glob::new("sample").unwrap();
        let filter = Filter::Name { comparison: Comparison::Eq, value: glob.into() };

        let mut entry = DirEntryMock::default().set_file("sample".into());

        let result = filter.evaluate(&entry);
        assert!(result.is_ok());
        assert!(result.unwrap());

        entry.file = PathBuf::from("a").into();
        let result = filter.evaluate(&entry);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_size() {
        let filter = Filter::Size { value: 100, comparison: Comparison::Lte };
        let mut entry =
            DirEntryMock::default().set_size(110).set_entry_type(EntryType::File);

        let result = filter.evaluate(&entry);
        assert!(result.is_ok());
        assert!(!result.unwrap());

        entry = entry.set_size(100);
        let result = filter.evaluate(&entry);
        assert!(result.is_ok());
        assert!(result.unwrap());

        entry = entry.set_entry_type(EntryType::Unknown).set_file("sample".into());
        let result = filter.evaluate(&entry);
        assert!(result.is_err());
    }

    #[test]
    fn test_depth() {
        let filter = Filter::Depth { value: 100, comparison: Comparison::Neq };
        let mut entry = DirEntryMock::default().set_depth(101);

        assert!(filter.evaluate(&entry).is_ok());
        assert!(filter.evaluate(&entry).unwrap());

        entry = entry.set_depth(100);
        assert!(filter.evaluate(&entry).is_ok());
        assert!(!filter.evaluate(&entry).unwrap());
    }

    #[test]
    fn test_type() {
        let filter = Filter::Type { value: FileType::Text, comparison: Comparison::Eq };
        let mut entry = DirEntryMock::default()
            .set_file("sample".into())
            .set_entry_type(EntryType::File);

        let result = filter.evaluate(&entry);
        assert!(result.is_err());

        let mut file = tempfile::NamedTempFile::new().unwrap();
        write!(file, "<html>").unwrap();
        file.flush().unwrap();
        entry = entry.set_file(file.path().into());

        let result = filter.evaluate(&entry);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_time() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let file_atime = file.path().metadata().unwrap().accessed().unwrap();
        let mut entry = DirEntryMock::default()
            .set_atime(file_atime)
            .set_file(file.path().into())
            .set_mtime(file_atime);

        let filters = [
            Filter::AccessTime { value: Duration::zero(), comparison: Comparison::Lte },
            Filter::ModificationTime {
                value: Duration::zero(),
                comparison: Comparison::Lte,
            },
        ];

        for filter in &filters {
            let result = filter.evaluate(&entry);
            assert!(result.is_ok(), "{:?}; {:?}", filter, &result);
            assert!(result.unwrap(), "{:?}", filter)
        }

        entry = entry
            .set_mtime(file_atime.add(std::time::Duration::from_secs(86400)))
            .set_atime(file_atime.add(std::time::Duration::from_secs(86400)));

        for filter in &filters {
            let result = filter.evaluate(&entry);
            assert!(result.is_ok(), "{:?}; {:?}", filter, &result);
            assert!(!result.unwrap(), "{:?}", filter)
        }
    }

    #[test]
    fn test_extension() {
        let filter = Filter::Extension { value: globset::Glob::new("txt").unwrap().into(), comparison: Comparison::Eq };
        let mut entry = DirEntryMock::default().set_file("long_sample_long.txt".into());

        let result = filter.evaluate(&entry);
        assert!(result.is_ok());
        assert!(result.unwrap());

        entry = entry.set_file("sample".into());
        let result = filter.evaluate(&entry);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_contains() {
        let filter = Filter::Contains { value: globset::Glob::new("*amp*").unwrap().into(), comparison: Comparison::Eq };
        let mut file = tempfile::NamedTempFile::new().unwrap();

        let entry = DirEntryMock::default().set_file(file.path().to_path_buf()).set_entry_type(EntryType::File);

        let result = filter.evaluate(&entry);
        assert!(result.is_ok());
        assert!(!result.unwrap());

        write!(file, "sample").unwrap();
        file.flush().unwrap();
        let result = filter.evaluate(&entry);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_user() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let uid = file.as_file().metadata().unwrap().st_uid();

        let filter = Filter::User { value: uid, comparison: Comparison::Eq };
        let entry = DirEntryMock::default().set_user_id(uid);

        let result = filter.evaluate(&entry);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_group() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let gid = file.as_file().metadata().unwrap().st_gid();

        let filter = Filter::Group { value: gid + 1000, comparison: Comparison::Lte };
        let entry = DirEntryMock::default().set_group_id(gid);

        let result = filter.evaluate(&entry);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_permissions() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let permissions = file.as_file().metadata().unwrap().permissions();

        let filter = Filter::Permissions { value: permissions.clone(), comparison: Comparison::Lte };
        let entry = DirEntryMock::default().set_permissions(permissions);

        let result = filter.evaluate(&entry);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}

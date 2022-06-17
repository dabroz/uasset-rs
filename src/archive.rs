use crate::{enums::ObjectVersionUE5, Error, ObjectVersion, Result};
use binread::BinReaderExt;
use num_traits::FromPrimitive;
use std::io::{Read, Seek};

/// Magic sequence identifying an unreal asset (can also be used to determine endianness)
const PACKAGE_FILE_MAGIC: u32 = 0x9E2A83C1;

#[derive(Debug)]
pub struct Archive<R> {
    pub reader: R,
    /// The serialization version used when saving this asset (C++ name: `FileVersionUE4`)
    pub file_version: ObjectVersion,
    /// The serialization version used when saving this asset (C++ name: `FileVersionUE5`)
    pub file_version_ue5: Option<ObjectVersionUE5>,
    /// The licensee serialization version used when saving this asset (C++ name: `FileVersionLicenseeUE4`)
    pub file_licensee_version: i32,
    pub legacy_version: i32,
}

impl<R> Archive<R>
where
    R: Seek + Read,
{
    pub fn new(mut reader: R) -> Result<Self> {
        let magic: u32 = reader.read_le()?;
        if magic != PACKAGE_FILE_MAGIC {
            return Err(Error::InvalidFile);
        }

        // See `void operator<<(FStructuredArchive::FSlot Slot, FPackageFileSummary& Sum)` in Engine/Source/Runtime/CoreUObject/Private/UObject/PackageFileSummary.cpp
        let legacy_version: i32 = reader.read_le()?;
        if !(-8..=-6).contains(&legacy_version) {
            return Err(Error::UnsupportedVersion(legacy_version));
        }

        let _legacy_ue3_version: i32 = reader.read_le()?;

        let file_version = reader.read_le()?;

        let file_version_ue5 = if legacy_version <= -8 {
            reader.read_le()?
        } else {
            0
        };

        let file_licensee_version: i32 = reader.read_le()?;
        if file_version == 0 && file_licensee_version == 0 && file_version_ue5 == 0 {
            return Err(Error::UnversionedAsset);
        }

        if file_version == 0 {
            return Err(Error::UnsupportedUE4Version(file_version));
        }
        let file_version = ObjectVersion::from_i32(file_version)
            .ok_or(Error::UnsupportedUE4Version(file_version))?;

        let file_version_ue5 = if file_version_ue5 != 0 {
            Some(
                ObjectVersionUE5::from_i32(file_version_ue5)
                    .ok_or(Error::UnsupportedUE5Version(file_version_ue5))?,
            )
        } else {
            None
        };

        Ok(Archive {
            reader,
            file_version,
            file_version_ue5,
            file_licensee_version,
            legacy_version,
        })
    }

    pub fn reader(&mut self) -> &mut R {
        &mut self.reader
    }

    pub fn serialized_with(&self, version: ObjectVersion) -> bool {
        self.file_version >= version
    }

    pub fn serialized_without(&self, version: ObjectVersion) -> bool {
        !self.serialized_with(version)
    }
}

impl<R> Read for Archive<R>
where
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}

impl<R> Seek for Archive<R>
where
    R: Seek,
{
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.reader.seek(pos)
    }
}

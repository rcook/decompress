use crate::RelPath;
use std::path::Path;

pub struct MapArgs<'a> {
    rel_path: &'a RelPath,
    path: &'a Path,
    output_dir: &'a Path,
}

impl<'a> MapArgs<'a> {
    pub fn rel_path(&self) -> &RelPath {
        self.rel_path
    }

    pub fn path(&self) -> &Path {
        self.path
    }

    pub fn output_dir(&self) -> &Path {
        self.output_dir
    }

    pub(crate) fn new(rel_path: &'a RelPath, path: &'a Path, output_dir: &'a Path) -> Self {
        assert!(path.is_absolute());
        assert!(output_dir.is_absolute());
        Self {
            rel_path,
            path,
            output_dir,
        }
    }
}

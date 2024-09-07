use crate::DecompressError;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::{Component, Path, PathBuf, MAIN_SEPARATOR};

type Result<T> = std::result::Result<T, DecompressError>;

const PART_SEPARATOR: char = '/';
const PART_SEPARATOR_STR: &str = "/";

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RelPathKind {
    File,
    Directory,
}

#[derive(Debug)]
pub struct RelPath {
    kind: RelPathKind,
    parts: Vec<String>,
    value: String,
}

impl RelPath {
    #[allow(unused)]
    pub fn new_guess_kind<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let kind = Self::guess_kind(path)?;
        Self::new(kind, path)
    }

    #[allow(unused)]
    pub fn new_directory<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::new(RelPathKind::Directory, path.as_ref())
    }

    #[allow(unused)]
    pub fn new_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::new(RelPathKind::File, path.as_ref())
    }

    #[allow(unused)]
    pub fn as_str(&self) -> &str {
        &self.value
    }

    #[allow(unused)]
    pub fn kind(&self) -> RelPathKind {
        self.kind
    }

    #[allow(unused)]
    pub fn is_directory(&self) -> bool {
        self.kind == RelPathKind::Directory
    }

    #[allow(unused)]
    pub fn is_file(&self) -> bool {
        self.kind == RelPathKind::File
    }

    #[allow(unused)]
    pub fn parts(&self) -> &Vec<String> {
        &self.parts
    }

    #[allow(unused)]
    pub fn join_onto<P: Into<PathBuf>>(&self, path: P) -> PathBuf {
        let mut p: PathBuf = path.into();
        p.extend(&self.parts);
        p
    }

    fn new(kind: RelPathKind, path: &Path) -> Result<Self> {
        let parts = Self::get_parts(path)?;
        let value = parts.join(PART_SEPARATOR_STR);
        Ok(Self { kind, parts, value })
    }

    fn guess_kind(path: &Path) -> Result<RelPathKind> {
        let s = path
            .to_str()
            .ok_or_else(|| DecompressError::PathNotUtf8(path.to_path_buf()))?;
        Ok(
            if s.ends_with(MAIN_SEPARATOR) || s.ends_with(PART_SEPARATOR) {
                RelPathKind::Directory
            } else {
                RelPathKind::File
            },
        )
    }

    fn get_parts(path: &Path) -> Result<Vec<String>> {
        if !path.is_relative() {
            return Err(DecompressError::PathNotRelative(path.to_path_buf()));
        }

        path.components()
            .map(|c| match c {
                Component::Normal(c) => c
                    .to_str()
                    .ok_or_else(|| DecompressError::PathNotUtf8(path.to_path_buf()))
                    .map(String::from),
                _ => unreachable!(),
            })
            .collect::<Result<Vec<_>>>()
    }
}

impl Display for RelPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::{RelPath, RelPathKind, Result};
    use rstest::rstest;
    use std::path::Path;

    #[rstest]
    #[case("aaa\\bbb", RelPathKind::File)]
    #[case("aaa\\bbb\\", RelPathKind::Directory)]
    #[case("aaa/bbb", RelPathKind::File)]
    #[case("aaa/bbb/", RelPathKind::Directory)]
    fn new_guess_kind(#[case] input: &str, #[case] expected: RelPathKind) -> Result<()> {
        assert_eq!(expected, RelPath::new_guess_kind(input)?.kind());
        assert_eq!(expected, RelPath::new_guess_kind(Path::new(input))?.kind());
        Ok(())
    }

    #[rstest]
    #[case("aaa\\bbb", "C:\\aaa\\bbb")]
    #[case("aaa\\bbb\\", "C:\\aaa\\bbb")]
    #[case("aaa/bbb", "C:\\aaa\\bbb")]
    #[case("aaa/bbb/", "C:\\aaa\\bbb")]
    fn new_directory(#[case] input: &str, #[case] expected: &str) -> Result<()> {
        let expected = Path::new(expected);
        let result = RelPath::new_directory(input)?;
        assert_eq!(RelPathKind::Directory, result.kind());
        assert!(result.is_directory());
        assert!(!result.is_file());
        assert_eq!(expected, result.join_onto("C:\\"));
        Ok(())
    }

    #[rstest]
    #[case("aaa\\bbb", "C:\\aaa\\bbb")]
    #[case("aaa\\bbb\\", "C:\\aaa\\bbb")]
    #[case("aaa/bbb", "C:\\aaa\\bbb")]
    #[case("aaa/bbb/", "C:\\aaa\\bbb")]
    fn new_file(#[case] input: &str, #[case] expected: &str) -> Result<()> {
        let expected = Path::new(expected);
        let result = RelPath::new_file(input)?;
        assert_eq!(RelPathKind::File, result.kind());
        assert!(!result.is_directory());
        assert!(result.is_file());
        assert_eq!(expected, result.join_onto("C:\\"));
        Ok(())
    }

    #[rstest]
    #[case("aaa\\bbb", "C:\\root", "C:\\root\\aaa\\bbb")]
    #[case("aaa\\bbb/", "C:\\root", "C:\\root\\aaa\\bbb")]
    #[case("aaa/bbb", "C:/root", "C:\\root\\aaa\\bbb")]
    #[case("aaa/bbb/", "C:/root", "C:\\root\\aaa\\bbb")]
    fn join_onto(#[case] input: &str, #[case] root: &str, #[case] expected: &str) -> Result<()> {
        let input = RelPath::new_file(input)?;
        let root = Path::new(root);
        let expected = Path::new(expected);
        assert_eq!(expected, input.join_onto(root));
        Ok(())
    }
}

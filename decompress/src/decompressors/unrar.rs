use crate::{
    DecompressError, Decompression, Decompressor, ExtractOpts, FilterArgs, Listing, MapArgs,
    RelPath,
};
use lazy_static::lazy_static;
use path_absolutize::Absolutize;
use regex::Regex;
use std::path::Path;
use unrar::FileHeader;

macro_rules! check {
    ($e : expr) => {
        $e.map_err(|e| crate::DecompressError::Error(e.to_string()))?
    };
}

lazy_static! {
    static ref RE: Regex = Regex::new(r"(?i)\.rar$").unwrap();
}

#[derive(Default)]
pub struct Unrar {
    re: Option<Regex>,
}
impl Unrar {
    #[must_use]
    pub fn new(re: Option<Regex>) -> Self {
        Self { re }
    }
    #[must_use]
    pub fn build(re: Option<Regex>) -> Box<Self> {
        Box::new(Self::new(re))
    }
}

impl Decompressor for Unrar {
    fn test_mimetype(&self, archive: &str) -> bool {
        archive == "application/vnd.rar"
    }

    fn test(&self, archive: &Path) -> bool {
        archive
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .map_or(false, |f| self.re.as_ref().unwrap_or(&*RE).is_match(f))
    }

    fn list(&self, archive: &Path) -> Result<Listing, DecompressError> {
        fn enclosed_name(h: FileHeader) -> String {
            let temp = h.filename.to_string_lossy();

            #[cfg(windows)]
            let mut s = temp.replace("\\", "/");

            #[cfg(unix)]
            let mut s = temp.into_owned();

            if h.is_directory() && !s.ends_with("/") {
                s.push('/')
            }

            s
        }

        let rar = check!(unrar::Archive::new(archive).open_for_listing());
        let entries = rar
            .into_iter()
            .map(|header| Ok(enclosed_name(check!(header))))
            .collect::<Result<Vec<_>, DecompressError>>()?;
        Ok(Listing { id: "rar", entries })
    }

    fn decompress(
        &self,
        archive: &Path,
        to: &Path,
        opts: &ExtractOpts,
    ) -> Result<Decompression, DecompressError> {
        use std::fs;

        let output_dir = to.absolutize()?;

        if opts.strip != 0 {
            todo!("Stripping path components not supported")
        }

        if !to.exists() {
            fs::create_dir_all(to)?;
        }

        let mut rar = check!(unrar::Archive::new(archive).open_for_processing());
        let mut files = Vec::new();
        while let Some(header) = check!(rar.read_header()) {
            let entry = header.entry();
            if entry.is_directory() || entry.is_file() {
                let output_path = to.join(&entry.filename);
                if output_path != to {
                    let rel_path = if entry.is_directory() {
                        RelPath::new_directory(&entry.filename)?
                    } else {
                        RelPath::new_file(&entry.filename)?
                    };
                    let full_output_path = output_dir.join(&entry.filename);
                    let filter_args = FilterArgs::new(&rel_path, &full_output_path, &output_dir);
                    if (opts.filter)(&filter_args) {
                        let map_args = MapArgs::new(&rel_path, &full_output_path, &output_dir);
                        let output_path = (opts.map)(&map_args);
                        files.push(output_path.to_string_lossy().into_owned());
                        rar = check!(header.extract_to(output_path));
                        continue;
                    }
                }
            }
            rar = check!(header.skip());
        }

        Ok(Decompression { id: "rar", files })
    }
}

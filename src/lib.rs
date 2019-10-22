use regex::Regex;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};

pub type Result<T> = std::result::Result<T, TocError>;

#[derive(Debug)]
pub enum TocError {
    RegexCompilationError(String),
    ReadLineError(String),
}

impl std::fmt::Display for TocError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TocError::RegexCompilationError(s) => write!(f, "{}", s),
            TocError::ReadLineError(s) => write!(f, "{}", s),
        }
    }
}

impl std::convert::From<regex::Error> for TocError {
    fn from(e: regex::Error) -> Self {
        Self::RegexCompilationError(e.to_string())
    }
}

impl std::convert::From<std::io::Error> for TocError {
    fn from(e: std::io::Error) -> Self {
        Self::ReadLineError(e.to_string())
    }
}

pub struct Toc {
    pub tags: HashMap<String, String>,
    pub files: Vec<String>,
}

impl Toc {
    /// Create a Toc object from the reader. Duplicate tag keys are silently
    /// overwritten.
    ///
    /// # Example
    ///
    /// ```
    /// use tocer::Toc;
    /// let reader = std::io::Cursor::new("## Interface: 1");
    /// let toc = Toc::load(reader).unwrap();
    /// dbg!(&toc.tags["Interface"]);
    /// ```
    pub fn load(reader: impl std::io::Read) -> Result<Toc> {
        let mut buf = BufReader::new(reader);
        let mut line = String::new();
        let mut tags = HashMap::new();
        let mut files = Vec::new();

        let tag_re = Regex::new(r"^##\s*(\S+)\s*:\s*(.*)")?;
        let file_re = Regex::new(r"^([^#\s].*)")?;

        // The unwraps can't fail without breaking the regex pattern, which
        // would also break the tests.
        while buf.read_line(&mut line)? != 0 {
            if let Some(caps) = tag_re.captures(line.as_ref()) {
                let tag = caps.get(1).unwrap();
                let value = caps.get(2).unwrap();
                tags.insert(tag.as_str().into(), value.as_str().into());
            } else if let Some(caps) = file_re.captures(line.as_ref()) {
                let file = caps.get(1).unwrap();
                files.push(file.as_str().into());
            }

            line.clear();
        }

        Ok(Toc { tags, files })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_reader() {
        let reader = std::io::Cursor::new("");
        let toc = Toc::load(reader).unwrap();
        assert!(toc.tags.is_empty());
        assert!(toc.files.is_empty());
    }

    #[test]
    fn test_files() {
        let s = concat!(
            "a.lua\n",
            "b.lua\n",
            "# comment\n",
            "c.lua  \n",
            "## bad comment\n",
            "dir\\d.xml\n",
            " f"
        );
        let reader = std::io::Cursor::new(s);
        let toc = Toc::load(reader).unwrap();
        assert!(toc.tags.is_empty());
        assert_eq!(toc.files.len(), 4);
        assert_eq!(toc.files[0], "a.lua");
        assert_eq!(toc.files[1], "b.lua");
        assert_eq!(toc.files[2], "c.lua  ");
        assert_eq!(toc.files[3], "dir\\d.xml");
    }

    #[test]
    fn test_tags() {
        let s = concat!(
            "##Interface:11302\n",
            "##Title: |cff20ff20Bagnon|r\n",
            "## Author: Tuller & Jaliborc (João Cardoso)\n",
            "## d : 8.2.16\n",
            "# comment\n",
            "## e : Bagnon_Sets \n",
            "## bad comment\n",
            "##   \t OptionalDeps : BagBrother, WoWUnit\n"
        );
        let reader = std::io::Cursor::new(s);
        let toc = Toc::load(reader).unwrap();
        assert_eq!(toc.tags.len(), 6);
        assert_eq!(toc.tags["Interface"], "11302");
        assert_eq!(toc.tags["Title"], "|cff20ff20Bagnon|r");
        assert_eq!(toc.tags["Author"], "Tuller & Jaliborc (João Cardoso)");
        assert_eq!(toc.tags["d"], "8.2.16");
        assert_eq!(toc.tags["e"], "Bagnon_Sets ");
        assert_eq!(toc.tags["OptionalDeps"], "BagBrother, WoWUnit");
        assert!(toc.files.is_empty());
    }
}

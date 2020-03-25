use std::collections::HashMap;
use std::io::{BufRead, BufReader};

pub struct Toc {
    pub tags: HashMap<String, String>,
    pub files: Vec<String>,
}

fn key_value_pair_begin(input: &str) -> Result<&str, &str> {
    let mut chars = input.chars();
    if chars.next() == Some('#') && chars.next() == Some('#') {
        Ok(&input[2..])
    } else {
        Err(input)
    }
}

fn key(input: &str) -> Result<(&str, &str), &str> {
    for (i, ch) in input.chars().enumerate() {
        if ch == ':' {
            let key = input[..i].trim();
            return Ok((key, &input[i..]));
        }
    }

    Err(input)
}

fn value(input: &str) -> Result<&str, &str> {
    if input.chars().next() != Some(':') {
        Err(input)
    } else {
        Ok(&input[1..].trim())
    }
}

fn key_value_pair(input: &str) -> Result<(&str, &str), &str> {
    let (k, input) = key(key_value_pair_begin(input)?)?;
    let v = value(input)?;
    Ok((k, v))
}

fn file_path(input: &str) -> Result<&str, &str> {
    if input.chars().next() == Some('#') {
        Err(input)
    } else {
        Ok(input.trim())
    }
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
    /// let toc = Toc::from_reader(reader).unwrap();
    /// dbg!(&toc.tags["Interface"]);
    /// ```
    pub fn from_reader(reader: impl std::io::Read) -> std::io::Result<Toc> {
        let mut buf = BufReader::new(reader);
        let mut line = String::new();
        let mut tags = HashMap::new();
        let mut files = Vec::new();

        while buf.read_line(&mut line)? != 0 {
            if let Ok((k, v)) = key_value_pair(&line) {
                tags.insert(k.to_string(), v.to_string());
            } else if let Ok(path) = file_path(&line) {
                println!("Adding {}", path);
                files.push(path.to_string());
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
    fn test_key_begin() {
        assert_eq!(key_value_pair_begin(""), Err(""));
        assert_eq!(key_value_pair_begin("#"), Err("#"));
        assert_eq!(key_value_pair_begin("##"), Ok(""));
    }

    #[test]
    fn test_key() {
        assert_eq!(key(""), Err(""));
        assert_eq!(key("a"), Err("a"));
        assert_eq!(key(" a "), Err(" a "));
        assert_eq!(key("a:"), Ok(("a", ":")));
        assert_eq!(key(" a:"), Ok(("a", ":")));
        assert_eq!(key("a:"), Ok(("a", ":")));
        assert_eq!(key("a :"), Ok(("a", ":")));
        assert_eq!(key(" a :"), Ok(("a", ":")));
    }

    #[test]
    fn test_value() {
        assert_eq!(value(""), Err(""));
        assert_eq!(value(" "), Err(" "));
        assert_eq!(value(": "), Ok(""));
        assert_eq!(value(":A"), Ok("A"));
        assert_eq!(value(": A"), Ok("A"));
        assert_eq!(value(": A "), Ok("A"));
    }

    #[test]
    fn test_empty_reader() {
        let reader = std::io::Cursor::new("");
        let toc = Toc::from_reader(reader).unwrap();
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
        let toc = Toc::from_reader(reader).unwrap();
        assert!(toc.tags.is_empty());
        assert_eq!(toc.files.len(), 5);
        assert_eq!(toc.files[0], "a.lua");
        assert_eq!(toc.files[1], "b.lua");
        assert_eq!(toc.files[2], "c.lua");
        assert_eq!(toc.files[3], "dir\\d.xml");
        assert_eq!(toc.files[4], "f");
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
        let toc = Toc::from_reader(reader).unwrap();
        assert_eq!(toc.tags.len(), 6);
        assert_eq!(toc.tags["Interface"], "11302");
        assert_eq!(toc.tags["Title"], "|cff20ff20Bagnon|r");
        assert_eq!(toc.tags["Author"], "Tuller & Jaliborc (João Cardoso)");
        assert_eq!(toc.tags["d"], "8.2.16");
        assert_eq!(toc.tags["e"], "Bagnon_Sets");
        assert_eq!(toc.tags["OptionalDeps"], "BagBrother, WoWUnit");
        assert!(toc.files.is_empty());
    }
}

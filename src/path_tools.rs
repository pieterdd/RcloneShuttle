use std::fmt::Display;

use regex::Regex;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct RclonePath {
    inner: String,
}

impl Display for RclonePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl RclonePath {
    pub fn from(raw: &str) -> Self {
        Self {
            inner: String::from(raw),
        }
    }

    pub fn resolve_to_parent(&self) -> Self {
        if self.inner.ends_with(':') {
            return RclonePath::from(&self.inner);
        }
        let windows_path_format = Regex::new(r"^[A-Za-z]:\\").unwrap();
        if windows_path_format.is_match(&self.inner) {
            // E.g. C:\
            if self.inner.len() == 3 {
                return self.clone();
            }
            let mut parts: Vec<&str> = self.inner[3..]
                .split(|c| c == '\\')
                .filter(|s| s != &"")
                .collect();
            parts.remove(parts.len() - 1);
            RclonePath::from(&(self.inner[0..3].to_owned() + &parts.join(r"\")))
        } else if self.inner == "/" {
            // Local Unix root
            self.clone()
        } else if self.inner.starts_with("/") {
            // Local Unix path
            let mut parts: Vec<&str> = self
                .inner
                .split(|c| c == '/')
                .filter(|s| s != &"")
                .collect();
            parts.remove(parts.len() - 1);
            RclonePath::from(&("/".to_owned() + &parts.join("/")))
        } else {
            let mut parts: Vec<&str> = self
                .inner
                .split(|c| c == ':' || c == '/')
                .filter(|s| s != &"")
                .collect();
            parts.remove(parts.len() - 1);
            RclonePath::from(&(parts[0].to_owned() + ":" + &parts[1..].join("/")))
        }
    }

    pub fn path_has_parent(&self) -> bool {
        if self.inner.contains('/') {
            return true;
        }
        !self.inner.ends_with(':')
    }

    pub fn filename(&self) -> String {
        if let Some((_, filename)) = self.inner.rsplit_once('\\') {
            filename.to_string()
        } else if let Some((_, filename)) = self.inner.rsplit_once('/') {
            filename.to_string()
        } else {
            self.inner.rsplit_once(':').unwrap().1.to_string()
        }
    }

    pub fn join(&self, tail: &str) -> RclonePath {
        let infix = if self.inner.ends_with('/') || self.inner.ends_with(':') {
            ""
        } else {
            "/"
        };
        RclonePath::from(&(format!("{}{}{}", self.inner, infix, tail)))
    }

    pub fn remote(&self) -> Option<String> {
        self.inner
            .find(':')
            .map(|index| String::from(&self.inner[0..index + 1]))
    }
}

#[cfg(test)]
mod tests {
    use crate::path_tools::RclonePath;
    use test_case::test_case;

    #[test]
    fn path_has_parent_with_parent() {
        let path_from_level_2 = RclonePath::from("foobar:foo");
        assert!(path_from_level_2.path_has_parent());
        let path_from_level_4 = RclonePath::from("foobar:foo/sjdflkdsfsd/sdfksdfsd");
        assert!(path_from_level_4.path_has_parent());
    }

    #[test]
    fn path_has_no_parent() {
        let path_from_root = RclonePath::from("foobar:");
        assert!(!path_from_root.path_has_parent());
    }

    #[test]
    fn resolve_to_parent_folder_already_parent() {
        let path_from_root = RclonePath::from("bla:");
        assert_eq!(path_from_root.resolve_to_parent(), RclonePath::from("bla:"));
    }

    #[test]
    fn resolve_to_parent_folder_not_parent_level_2_without_slash() {
        let path = RclonePath::from("bla:foo");
        assert_eq!(path.resolve_to_parent(), RclonePath::from("bla:"));
    }

    #[test]
    fn resolve_to_parent_folder_not_parent_level_2_with_slash() {
        let path = RclonePath::from("bla:foo/");
        assert_eq!(path.resolve_to_parent(), RclonePath::from("bla:"));
    }

    #[test]
    fn resolve_to_parent_folder_not_parent_level_3_without_slash() {
        let path = RclonePath::from("bla:foo/bar");
        assert_eq!(path.resolve_to_parent(), RclonePath::from("bla:foo"));
    }

    #[test_case("/", "/"; "top level")]
    #[test_case("/home", "/"; "level 2")]
    #[test_case("/home/foo/bar", "/home/foo"; "deeper than level 2")]
    fn resolve_to_parent_folder_local_unix(raw_input: &str, raw_output: &str) {
        let path = RclonePath::from(raw_input);
        assert_eq!(path.resolve_to_parent(), RclonePath::from(raw_output));
    }

    #[test_case(r"C:\", r"C:\"; "top level")]
    #[test_case(r"C:\Users", r"C:\"; "level 2 without slash")]
    #[test_case(r"C:\Users\", r"C:\"; "level 2 with slash")]
    #[test_case(r"C:\Users\foo\Desktop\bar.exe", r"C:\Users\foo\Desktop"; "deeper than level 2")]
    fn resolve_to_parent_folder_local_windows(raw_input: &str, raw_output: &str) {
        let path = RclonePath::from(raw_input);
        assert_eq!(path.resolve_to_parent(), RclonePath::from(raw_output));
    }

    #[test]
    fn get_filename_from_path_local_unix() {
        let path = RclonePath::from("/home/foo/bar");
        assert_eq!(path.filename(), "bar");
    }

    #[test]
    fn get_filename_from_path_local_windows() {
        let path = RclonePath::from(r"C:\Users\foo\Desktop\bar.exe");
        assert_eq!(path.filename(), "bar.exe");
    }

    #[test]
    fn get_filename_from_path_remote() {
        let path = RclonePath::from("foo:bar/bla.zip");
        assert_eq!(path.filename(), "bla.zip");
    }
    #[test]
    fn join_on_root() {
        let path = RclonePath::from("foo:");
        assert_eq!(path.join("bar"), RclonePath::from("foo:bar"));
    }

    #[test]
    fn join_without_trailing_slash() {
        let path_from_level_2 = RclonePath::from("foo:path");
        assert_eq!(
            path_from_level_2.join("bar"),
            RclonePath::from("foo:path/bar")
        );
        let path_from_level_3 = RclonePath::from("foo:path/subfolder");
        assert_eq!(
            path_from_level_3.join("bar"),
            RclonePath::from("foo:path/subfolder/bar")
        );
    }

    #[test]
    fn join_with_trailing_slash() {
        let path_from_level_2 = RclonePath::from("foo:path/");
        assert_eq!(
            path_from_level_2.join("bar"),
            RclonePath::from("foo:path/bar")
        );
        let path_from_level_3 = RclonePath::from("foo:path/subfolder/");
        assert_eq!(
            path_from_level_3.join("bar"),
            RclonePath::from("foo:path/subfolder/bar")
        );
    }

    #[test]
    fn bad_remote() {
        let path = RclonePath::from("foo");
        assert_eq!(path.remote(), None);
    }

    #[test]
    fn remote_from_root() {
        let path = RclonePath::from("foo:");
        assert_eq!(path.remote(), Some(String::from("foo:")));
    }

    #[test]
    fn remote_from_level_3() {
        let path = RclonePath::from("foo:bla/bar");
        assert_eq!(path.remote(), Some(String::from("foo:")));
    }
}

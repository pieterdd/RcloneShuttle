use std::fmt::Display;

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
        let mut parts: Vec<&str> = self
            .inner
            .split(|c| c == ':' || c == '/')
            .filter(|s| s != &"")
            .collect();
        parts.remove(parts.len() - 1);

        RclonePath::from(&(parts[0].to_owned() + ":" + &parts[1..].join("/")))
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

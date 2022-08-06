pub fn canonicalize_path(path: String) -> String {
    if path.is_empty() {
        path
    } else {
        let path = path.replace('\\', "/");
        let is_absolute = if let Some('/') = path.chars().nth(0) {
            true
        } else {
            false
        };

        let mut components = Vec::new();

        for component in path.split("/") {
            if let None = component.chars().find(|chr| *chr != '.') {
                // component contains only dots ('.')
                let number_of_dots = component.len();
                for i in 1..number_of_dots {
                    if !components.is_empty() {
                        components.pop();
                    }
                }
            } else {
                // component contains at least one not dot ('.') character
                components.push(component);
            }
        }

        let mut ret = String::new();

        let mut first = true;
        for component in components {
            if !first {
                ret.push_str("/");
            } else {
                first = false;
                if is_absolute {
                    ret.push_str("/");
                }
            }
            ret.push_str(&component);
        }

        ret
    }
}

pub fn parent_path(path: String) -> String {
    std::path::Path::new(&canonicalize_path(path))
        .ancestors()
        .skip(1)
        .next()
        .map_or("".to_string(), |path| path.to_str().unwrap().to_string())
}

#[derive(Clone)]
pub struct AssetsReader {}

impl AssetsReader {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_reader(&self, path: &str) -> Option<impl std::io::Read> {
        std::fs::File::open(path).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_canonicalization() {
        assert_eq!(canonicalize_path("foo/bar".to_string()), "foo/bar");
        assert_eq!(canonicalize_path("./foo/bar".to_string()), "foo/bar");
        assert_eq!(canonicalize_path("/foo/bar".to_string()), "/foo/bar");
        assert_eq!(canonicalize_path("/foo/bar/".to_string()), "/foo/bar");
        assert_eq!(canonicalize_path("foo//bar/..\\bar".to_string()), "foo/bar");
        assert_eq!(canonicalize_path("../foo/bar".to_string()), "foo/bar");
        assert_eq!(canonicalize_path("../../foo/bar".to_string()), "foo/bar");
        assert_eq!(canonicalize_path("foo/bar/../../".to_string()), "");
        assert_eq!(canonicalize_path("foo/bar/../../foo".to_string()), "foo");
        assert_eq!(canonicalize_path("c:\\foo\\bar".to_string()), "c:/foo/bar");
        assert_eq!(canonicalize_path("foo/bar/..././foo".to_string()), "foo");
        assert_eq!(canonicalize_path("bar/./.../foo".to_string()), "foo");
        assert_eq!(
            canonicalize_path("foo/bar/foo/bar/.../foo".to_string()),
            "foo/bar/foo"
        );
    }

    #[test]
    fn path_parent() {
        assert_eq!(parent_path("textures/albedo.png".to_string()), "textures");
        assert_eq!(parent_path("albedo.png".to_string()), "");
        assert_eq!(parent_path("".to_string()), "");
    }
}

use rtvui_core::paths::AbsolutePath;

#[derive(Debug, Default)]
pub struct NavHistory {
    back: Vec<AbsolutePath>,
    forward: Vec<AbsolutePath>,
}

impl NavHistory {
    pub fn push_visit(&mut self, from: AbsolutePath) {
        if self.back.last() == Some(&from) {
            return;
        }
        self.back.push(from);
        self.forward.clear();
    }

    pub fn can_back(&self) -> bool {
        !self.back.is_empty()
    }

    pub fn can_forward(&self) -> bool {
        !self.forward.is_empty()
    }

    pub fn go_back(&mut self, current: AbsolutePath) -> Option<AbsolutePath> {
        let prev = self.back.pop()?;
        self.forward.push(current);
        Some(prev)
    }

    pub fn go_forward(&mut self, current: AbsolutePath) -> Option<AbsolutePath> {
        let next = self.forward.pop()?;
        self.back.push(current);
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn path(s: &str) -> AbsolutePath {
        AbsolutePath(PathBuf::from(s))
    }

    #[test]
    fn back_forward_roundtrip() {
        let mut h = NavHistory::default();
        let a = path("/a");
        let b = path("/b");
        h.push_visit(a.clone());
        let back = h.go_back(b.clone()).unwrap();
        assert_eq!(back, a);
        let fwd = h.go_forward(back).unwrap();
        assert_eq!(fwd, b);
    }
}

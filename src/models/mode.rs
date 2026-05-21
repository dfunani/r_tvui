#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Help,
    Input(InputKind),
    ConfirmDelete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    Filter,
    GoToPath,
    Rename,
}

impl AppMode {
    pub fn is_input(&self) -> bool {
        matches!(self, Self::Input(_))
    }

    pub fn input_kind(&self) -> Option<InputKind> {
        match self {
            Self::Input(kind) => Some(*kind),
            _ => None,
        }
    }
}

/// Preview pane presentation modes (reserved for filter/edit overlays).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewWindowStates {
    View,
    Edit,
    Exit,
}

#[derive(Default, Eq, PartialEq)]
pub enum ButtonState {
    #[default]
    Default,
    InProgress,
    Failed,
}

impl ButtonState {
    pub fn is_in_progress(&self) -> bool {
        *self == Self::InProgress
    }
}

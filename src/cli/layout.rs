// スタブ実装
pub struct DisplayLayout {
    // 仮のフィールド
    pub lines: Vec<String>,
}

impl DisplayLayout {
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }
}

impl Default for DisplayLayout {
    fn default() -> Self {
        Self::new()
    }
}

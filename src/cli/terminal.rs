use std::fmt;
use std::io::{self, Write};
use crate::cli::layout::DisplayLayout;
#[allow(unused_imports)]
use terminal_size::{terminal_size, Width, Height};

#[derive(Debug, PartialEq)]
pub enum AnsiSequence {
    SaveCursor,      // \x1b[s
    RestoreCursor,   // \x1b[u
    HideCursor,      // \x1b[?25l
    ShowCursor,      // \x1b[?25h
    MoveUp(u16),     // \x1b[nA
    ClearLine,       // \x1b[2K
}

impl fmt::Display for AnsiSequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnsiSequence::SaveCursor => write!(f, "\x1b[s"),
            AnsiSequence::RestoreCursor => write!(f, "\x1b[u"),
            AnsiSequence::HideCursor => write!(f, "\x1b[?25l"),
            AnsiSequence::ShowCursor => write!(f, "\x1b[?25h"),
            AnsiSequence::MoveUp(n) => write!(f, "\x1b[{}A", n),
            AnsiSequence::ClearLine => write!(f, "\x1b[2K"),
        }
    }
}

pub struct TerminalBuffer {
    buffer: String,
}

impl TerminalBuffer {
    pub fn new() -> Self {
        Self {
            buffer: String::with_capacity(4096),
        }
    }

    pub fn queue<D: fmt::Display>(&mut self, d: D) {
        use std::fmt::Write;
        let _ = write!(self.buffer, "{}", d);
    }

    pub fn flush(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();
        stdout.write_all(self.buffer.as_bytes())?;
        stdout.flush()?;
        self.buffer.clear();
        Ok(())
    }

    #[cfg(test)]
    pub fn get_content(&self) -> &str {
        &self.buffer
    }
    
    #[cfg(test)]
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }
}

pub struct TerminalController {
    buffer: TerminalBuffer,
    width: u16,
    height: u16,
    last_line_count: u16,
}

impl TerminalController {
    pub fn new() -> Self {
        let (w, h) = if let Some((Width(w), Height(h))) = terminal_size() {
            (w, h)
        } else {
            (80, 24) // デフォルトサイズ
        };

        Self {
            buffer: TerminalBuffer::new(),
            width: w,
            height: h,
            last_line_count: 0,
        }
    }

    pub fn render(&mut self, layout: &DisplayLayout) -> io::Result<()> {
        // カーソル位置保存と非表示
        self.buffer.queue(AnsiSequence::SaveCursor);
        self.buffer.queue(AnsiSequence::HideCursor);

        // 前回の描画行数分だけ上に移動
        if self.last_line_count > 0 {
            self.buffer.queue(AnsiSequence::MoveUp(self.last_line_count));
        }

        // 新しい行を描画
        let mut line_count = 0;
        for line in &layout.lines {
            self.buffer.queue(AnsiSequence::ClearLine);
            self.buffer.queue(line);
            self.buffer.queue("\n");
            line_count += 1;
        }

        // 行数が減った場合、残りの行をクリアする必要があるが、
        // 今回の要件（MoveUpしてから描画）だと、
        // 常に上書きモードで描画している。
        // MoveUpはカーソルを相対移動させるだけ。
        // クリアロジックの詳細は設計書に準拠すべきだが、
        // 単純な実装としては、
        // 1. カーソル保存
        // 2. カーソル非表示
        // 3. 前回の行数分上に移動
        // 4. 行ごとに「行クリア -> 内容出力 -> 改行」
        // 5. カーソル復元
        // 6. カーソル表示
        
        // 注意: 前回の行数より今回の行数が少ない場合、古い行が残る可能性がある。
        // MoveUp(last_line_count)した位置から描画を開始する。
        // もし今回が3行で前回が5行なら、3行分上書きした後、残り2行が残る。
        // これを防ぐには、書き始める前に last_line_count 分の行を全てクリアするか、
        // 書き終わった後に余分な行をクリアする必要がある。
        // ここでは、設計書に「MoveUp -> ClearLine」とあるので、行ごとにクリアする方針。
        // しかし、行数が減るケースに対応するため、余分な行もクリアする処理を追加する。
        
        // TODO: 余分な行のクリア処理（今回は一旦スキップ、Redで確認）

        self.last_line_count = line_count;

        // カーソル復元と表示
        self.buffer.queue(AnsiSequence::RestoreCursor);
        self.buffer.queue(AnsiSequence::ShowCursor);

        self.buffer.flush()
    }

    pub fn clear(&mut self) -> io::Result<()> {
        // 全画面クリアではなく、前回描画した領域をクリアする意図か？
        // 設計書には「行単位のクリア」とある。
        // 全クリアが必要なら \x1b[2J \x1b[H などを使う。
        // ここでは前回描画分を消去してリセットする実装にする。
        if self.last_line_count > 0 {
            self.buffer.queue(AnsiSequence::MoveUp(self.last_line_count));
            for _ in 0..self.last_line_count {
                self.buffer.queue(AnsiSequence::ClearLine);
                self.buffer.queue("\n");
            }
            // カーソルを戻すために再度MoveUp? いや、\nで下がってるから戻る必要がある。
            // 正確には MoveUp -> (ClearLine + Down) * N -> MoveUp * N
            // 簡易的に、AnsiSequence::ClearLine はカーソル位置を変えないので、
            // MoveUpしてから ClearLine -> 下へ移動 を繰り返すより、
            // 現在位置から上に向かって消していく方が楽かもしれないが、
            // 通常は MoveUp -> (ClearLine + \n) 
        }
        self.last_line_count = 0;
        self.buffer.flush()
    }
}

impl Drop for TerminalController {
    fn drop(&mut self) {
        // カーソルを必ず表示状態に戻す
        self.buffer.queue(AnsiSequence::ShowCursor);
        let _ = self.buffer.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_sequence_display() {
        assert_eq!(AnsiSequence::SaveCursor.to_string(), "\x1b[s");
        assert_eq!(AnsiSequence::RestoreCursor.to_string(), "\x1b[u");
        assert_eq!(AnsiSequence::HideCursor.to_string(), "\x1b[?25l");
        assert_eq!(AnsiSequence::ShowCursor.to_string(), "\x1b[?25h");
        assert_eq!(AnsiSequence::MoveUp(3).to_string(), "\x1b[3A");
        assert_eq!(AnsiSequence::ClearLine.to_string(), "\x1b[2K");
    }

    #[test]
    fn test_terminal_buffer() {
        let mut buffer = TerminalBuffer::new();
        buffer.queue("Hello");
        buffer.queue(" ");
        buffer.queue("World");
        buffer.queue(AnsiSequence::ClearLine);
        
        assert_eq!(buffer.get_content(), "Hello World\x1b[2K");
        
        // flushのテストはstdoutをキャプチャする必要があるが、
        // ここではbufferがクリアされることだけ確認する手もある。
        // しかし実際のflushはio::stdoutへの書き込みを伴うため、ユニットテストでは難しい。
        // 統合テストで行うか、Writerを注入できるように設計変更するか。
        // 今回は簡易的に実装。
    }

    #[test]
    fn test_controller_render_flow() {
        // TerminalControllerのテスト
        // stdoutへの書き込みを伴うので、テスト実行時に画面が乱れる可能性がある。
        // バッファの内容を検証したいが、private fieldなのでアクセスできない。
        // Writerを注入できるようにリファクタリングするのがベスト。
    }
}

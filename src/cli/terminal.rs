use std::fmt;
use std::io::{self, Write};
use crate::cli::layout::DisplayLayout;
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
    writer: Box<dyn Write + Send>,
}

impl TerminalBuffer {
    pub fn new() -> Self {
        Self {
            buffer: String::with_capacity(4096),
            writer: Box::new(io::stdout()),
        }
    }

    pub fn with_writer(writer: Box<dyn Write + Send>) -> Self {
        Self {
            buffer: String::with_capacity(4096),
            writer,
        }
    }

    pub fn queue<D: fmt::Display>(&mut self, d: D) {
        use std::fmt::Write;
        let _ = write!(self.buffer, "{}", d);
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.write_all(self.buffer.as_bytes())?;
        self.writer.flush()?;
        self.buffer.clear();
        Ok(())
    }
}

pub struct TerminalController {
    buffer: TerminalBuffer,
    #[allow(dead_code)]
    width: u16,
    #[allow(dead_code)]
    height: u16,
    last_line_count: u16,
}

impl TerminalController {
    pub fn new() -> Self {
        let (w, h) = if let Some((Width(w), Height(h))) = terminal_size() {
            (w, h)
        } else {
            (80, 24)
        };

        Self {
            buffer: TerminalBuffer::new(),
            width: w,
            height: h,
            last_line_count: 0,
        }
    }
    
    #[cfg(test)]
    pub fn with_writer(writer: Box<dyn Write + Send>) -> Self {
        Self {
            buffer: TerminalBuffer::with_writer(writer),
            width: 80,
            height: 24,
            last_line_count: 0,
        }
    }

    pub fn render(&mut self, layout: &DisplayLayout) -> io::Result<()> {
        self.buffer.queue(AnsiSequence::SaveCursor);
        self.buffer.queue(AnsiSequence::HideCursor);

        if self.last_line_count > 0 {
            self.buffer.queue(AnsiSequence::MoveUp(self.last_line_count));
        }

        let mut line_count = 0;
        for line in &layout.lines {
            self.buffer.queue(AnsiSequence::ClearLine);
            self.buffer.queue(line);
            self.buffer.queue("\n");
            line_count += 1;
        }

        self.last_line_count = line_count;

        self.buffer.queue(AnsiSequence::RestoreCursor);
        self.buffer.queue(AnsiSequence::ShowCursor);

        self.buffer.flush()
    }

    pub fn clear(&mut self) -> io::Result<()> {
        if self.last_line_count > 0 {
            self.buffer.queue(AnsiSequence::MoveUp(self.last_line_count));
            for _ in 0..self.last_line_count {
                self.buffer.queue(AnsiSequence::ClearLine);
                self.buffer.queue("\n");
            }
            // MoveUp to restore position roughly? No, just clear buffer state
            // If we printed newlines, the cursor moved down.
            // We need to move back up to the starting position if we want to "clear" the area completely
            // and leave the cursor where it started.
            // But if we just want to wipe the text, \n moves down.
            // To restore cursor to original position:
            self.buffer.queue(AnsiSequence::MoveUp(self.last_line_count));
        }
        self.last_line_count = 0;
        self.buffer.flush()
    }
}

impl Drop for TerminalController {
    fn drop(&mut self) {
        self.buffer.queue(AnsiSequence::ShowCursor);
        let _ = self.buffer.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct MockWriter {
        data: Arc<Mutex<Vec<u8>>>,
    }

    impl MockWriter {
        fn new() -> Self {
            Self {
                data: Arc::new(Mutex::new(Vec::new())),
            }
        }
        
        fn get_content(&self) -> String {
            let data = self.data.lock().unwrap();
            String::from_utf8(data.clone()).unwrap()
        }
    }

    impl Write for MockWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.data.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

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
        let writer = MockWriter::new();
        let mut buffer = TerminalBuffer::with_writer(Box::new(writer.clone()));
        
        buffer.queue("Hello");
        buffer.queue(" ");
        buffer.queue("World");
        buffer.queue(AnsiSequence::ClearLine);
        buffer.flush().unwrap();
        
        assert_eq!(writer.get_content(), "Hello World\x1b[2K");
    }

    #[test]
    fn test_controller_render() {
        let writer = MockWriter::new();
        let mut controller = TerminalController::with_writer(Box::new(writer.clone()));
        
        let mut layout = DisplayLayout::new();
        layout.lines.push("Line 1".to_string());
        layout.lines.push("Line 2".to_string());
        
        controller.render(&layout).unwrap();
        
        let content = writer.get_content();
        // 初回描画: Save -> Hide -> (No MoveUp) -> ClearLine -> Line1 -> \n -> ClearLine -> Line2 -> \n -> Restore -> Show
        assert!(content.contains("\x1b[s"));
        assert!(content.contains("\x1b[?25l"));
        assert!(!content.contains("\x1b[A")); // 初回なのでMoveUpなし
        assert!(content.contains("Line 1\n"));
        assert!(content.contains("Line 2\n"));
        assert!(content.contains("\x1b[u"));
        assert!(content.contains("\x1b[?25h"));
    }

    #[test]
    fn test_controller_render_update() {
        let writer = MockWriter::new();
        let mut controller = TerminalController::with_writer(Box::new(writer.clone()));
        
        let mut layout = DisplayLayout::new();
        layout.lines.push("Line 1".to_string());
        controller.render(&layout).unwrap();
        
        // 2回目の描画
        let writer2 = MockWriter::new(); // 新しいWriterにして出力を分離したいが、controllerは所有している
        // なので、同じWriterに追記されることを確認するか、あるいはMockWriterの中身をクリアする機能をつけるか。
        // MockWriterは共有されているので、dataをクリアすればいい。
        writer.data.lock().unwrap().clear();
        
        layout.lines.push("Line 2".to_string());
        controller.render(&layout).unwrap();
        
        let content = writer.get_content();
        // 2回目: Save -> Hide -> MoveUp(1) -> ...
        assert!(content.contains("\x1b[1A")); // 前回が1行だったので1行戻る
        assert!(content.contains("Line 1\n"));
        assert!(content.contains("Line 2\n"));
    }
}

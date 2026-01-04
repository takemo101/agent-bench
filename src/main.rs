//! ポモドーロタイマーCLI
//!
//! macOS専用のポモドーロタイマーCLIツール。
//! デーモンプロセスとして動作し、Unix Domain Socket経由でCLIコマンドを受け付ける。

fn main() {
    println!("pomodoro v{}", env!("CARGO_PKG_VERSION"));
}

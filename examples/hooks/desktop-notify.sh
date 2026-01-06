#!/bin/bash
# =============================================================================
# desktop-notify.sh - macOSデスクトップ通知フック
# =============================================================================
# 
# 概要:
#   ポモドーロタイマーのイベントをmacOSの通知センターに表示するスクリプト
#   osascript（AppleScript）を使用
#
# 使用方法:
#   1. このスクリプトに実行権限を付与: chmod +x desktop-notify.sh
#   2. ポモドーロタイマーの設定でフックとして登録:
#      pomodoro config hooks.work_end "path/to/desktop-notify.sh"
#
# 環境変数（ポモドーロタイマーから自動設定）:
#   POMODORO_EVENT        - イベント名
#   POMODORO_TASK_NAME    - タスク名
#   POMODORO_CURRENT_CYCLE - 現在のサイクル数
#   POMODORO_TOTAL_CYCLES  - 総サイクル数
#
# 注意:
#   このスクリプトはmacOS専用です
#
# =============================================================================

set -euo pipefail

# イベントに応じたタイトルとメッセージ
case "${POMODORO_EVENT:-}" in
    work_start)
        title="🍅 作業開始"
        message="${POMODORO_TASK_NAME:-タスク名なし}"
        sound="Blow"
        ;;
    work_end)
        title="✅ 作業完了"
        message="休憩を取りましょう (${POMODORO_CURRENT_CYCLE:-?}/${POMODORO_TOTAL_CYCLES:-?})"
        sound="Glass"
        ;;
    break_start)
        title="☕ 休憩開始"
        message="リフレッシュしましょう"
        sound="Breeze"
        ;;
    break_end)
        title="💪 休憩終了"
        message="次のポモドーロを始めましょう！"
        sound="Blow"
        ;;
    session_complete)
        title="🎉 セッション完了"
        message="お疲れ様でした！"
        sound="Fanfare"
        ;;
    *)
        title="🔔 ポモドーロ"
        message="イベント: ${POMODORO_EVENT:-unknown}"
        sound="Default"
        ;;
esac

# macOS通知を表示
osascript -e "display notification \"${message}\" with title \"${title}\" sound name \"${sound}\""

echo "Desktop notification displayed: ${title} - ${message}"

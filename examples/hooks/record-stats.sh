#!/bin/bash
# =============================================================================
# record-stats.sh - 統計記録フック
# =============================================================================
# 
# 概要:
#   ポモドーロタイマーのイベントをCSVファイルに記録するスクリプト
#   作業時間やタスクの統計を後から分析できます
#
# 使用方法:
#   1. このスクリプトに実行権限を付与: chmod +x record-stats.sh
#   2. 環境変数 POMODORO_STATS_FILE で出力先を指定（オプション）
#      デフォルト: ~/pomodoro-stats.csv
#   3. ポモドーロタイマーの設定でフックとして登録:
#      pomodoro config hooks.work_end "path/to/record-stats.sh"
#      pomodoro config hooks.session_complete "path/to/record-stats.sh"
#
# 環境変数（ポモドーロタイマーから自動設定）:
#   POMODORO_EVENT          - イベント名
#   POMODORO_TASK_NAME      - タスク名
#   POMODORO_PHASE          - 現在のフェーズ (work, short_break, long_break)
#   POMODORO_WORK_DURATION  - 作業時間（秒）
#   POMODORO_BREAK_DURATION - 休憩時間（秒）
#   POMODORO_ELAPSED_SECS   - 経過時間（秒）
#   POMODORO_CURRENT_CYCLE  - 現在のサイクル数
#   POMODORO_TOTAL_CYCLES   - 総サイクル数
#   POMODORO_SESSION_ID     - セッションID
#   POMODORO_TIMESTAMP      - イベント発生時刻 (ISO 8601形式)
#
# 出力CSV形式:
#   timestamp,event,task_name,phase,cycle,total_cycles,elapsed_secs,session_id
#
# =============================================================================

set -euo pipefail

# 出力先ファイル
STATS_FILE="${POMODORO_STATS_FILE:-$HOME/pomodoro-stats.csv}"

# ファイルが存在しない場合はヘッダーを追加
if [[ ! -f "$STATS_FILE" ]]; then
    echo "timestamp,event,task_name,phase,cycle,total_cycles,elapsed_secs,session_id" > "$STATS_FILE"
fi

# タスク名のエスケープ（カンマと改行を処理）
task_name="${POMODORO_TASK_NAME:-}"
task_name="${task_name//,/;}"  # カンマをセミコロンに置換
task_name="${task_name//$'\n'/ }"  # 改行をスペースに置換

# CSVレコードを追記
echo "${POMODORO_TIMESTAMP:-$(date -u +%Y-%m-%dT%H:%M:%SZ)},\
${POMODORO_EVENT:-unknown},\
${task_name},\
${POMODORO_PHASE:-},\
${POMODORO_CURRENT_CYCLE:-0},\
${POMODORO_TOTAL_CYCLES:-0},\
${POMODORO_ELAPSED_SECS:-0},\
${POMODORO_SESSION_ID:-}" >> "$STATS_FILE"

echo "Stats recorded to: ${STATS_FILE}"

# 今日の統計サマリーを表示（work_endイベント時のみ）
if [[ "${POMODORO_EVENT:-}" == "session_complete" ]]; then
    today=$(date +%Y-%m-%d)
    completed_today=$(grep "^${today}" "$STATS_FILE" | grep -c "work_end" || echo "0")
    echo "Today's completed pomodoros: ${completed_today}"
fi

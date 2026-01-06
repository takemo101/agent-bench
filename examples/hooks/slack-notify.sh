#!/bin/bash
# =============================================================================
# slack-notify.sh - Slack通知フック
# =============================================================================
# 
# 概要:
#   ポモドーロタイマーのイベントをSlackに通知するスクリプト
#
# 使用方法:
#   1. Slack Incoming Webhook URLを取得
#      https://api.slack.com/messaging/webhooks
#   2. 環境変数 SLACK_WEBHOOK_URL を設定
#   3. このスクリプトに実行権限を付与: chmod +x slack-notify.sh
#   4. ポモドーロタイマーの設定でフックとして登録:
#      pomodoro config hooks.work_end "path/to/slack-notify.sh"
#
# 環境変数（ポモドーロタイマーから自動設定）:
#   POMODORO_EVENT        - イベント名 (work_start, work_end, break_start, break_end, etc.)
#   POMODORO_TASK_NAME    - タスク名
#   POMODORO_CURRENT_CYCLE - 現在のサイクル数
#   POMODORO_TOTAL_CYCLES  - 総サイクル数
#   POMODORO_TIMESTAMP    - イベント発生時刻 (ISO 8601形式)
#
# =============================================================================

set -euo pipefail

# Slack Webhook URL（環境変数から取得）
WEBHOOK_URL="${SLACK_WEBHOOK_URL:-}"

if [[ -z "$WEBHOOK_URL" ]]; then
    echo "Error: SLACK_WEBHOOK_URL is not set" >&2
    exit 1
fi

# イベントに応じたメッセージとアイコン
case "${POMODORO_EVENT:-}" in
    work_start)
        emoji=":tomato:"
        message="作業開始: ${POMODORO_TASK_NAME:-タスク名なし}"
        ;;
    work_end)
        emoji=":white_check_mark:"
        message="作業完了！休憩を取りましょう (${POMODORO_CURRENT_CYCLE:-?}/${POMODORO_TOTAL_CYCLES:-?})"
        ;;
    break_start)
        emoji=":coffee:"
        message="休憩開始: リフレッシュしましょう"
        ;;
    break_end)
        emoji=":muscle:"
        message="休憩終了: 次のポモドーロを始めましょう！"
        ;;
    session_complete)
        emoji=":tada:"
        message="セッション完了！お疲れ様でした！"
        ;;
    *)
        emoji=":bell:"
        message="ポモドーロイベント: ${POMODORO_EVENT:-unknown}"
        ;;
esac

# Slackに通知を送信
payload=$(cat <<EOF
{
    "text": "${emoji} ${message}",
    "username": "Pomodoro Timer",
    "icon_emoji": ":tomato:"
}
EOF
)

curl -s -X POST -H "Content-Type: application/json" \
    -d "$payload" \
    "$WEBHOOK_URL" > /dev/null

echo "Slack notification sent: ${message}"

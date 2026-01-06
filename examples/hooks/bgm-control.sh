#!/bin/bash
# =============================================================================
# bgm-control.sh - BGM制御フック
# =============================================================================
# 
# 概要:
#   ポモドーロタイマーのイベントに応じてBGMを自動制御するスクリプト
#   作業開始時に音楽を再生し、休憩開始時に一時停止します
#
# 対応アプリ:
#   - Spotify
#   - Apple Music (ミュージック)
#
# 使用方法:
#   1. このスクリプトに実行権限を付与: chmod +x bgm-control.sh
#   2. 環境変数 POMODORO_BGM_APP で使用するアプリを指定（オプション）
#      設定値: "spotify" または "music"
#      デフォルト: 両方のアプリを試行
#   3. ポモドーロタイマーの設定でフックとして登録:
#      pomodoro config hooks.work_start "path/to/bgm-control.sh"
#      pomodoro config hooks.break_start "path/to/bgm-control.sh"
#
# 環境変数（ポモドーロタイマーから自動設定）:
#   POMODORO_EVENT - イベント名
#
# 注意:
#   - このスクリプトはmacOS専用です
#   - 音楽アプリが起動している必要があります
#
# =============================================================================

set -euo pipefail

# 使用するアプリ（spotify, music, または空欄で自動検出）
BGM_APP="${POMODORO_BGM_APP:-}"

# Spotifyを制御
control_spotify() {
    local action="$1"
    if pgrep -x "Spotify" > /dev/null; then
        osascript -e "tell application \"Spotify\" to ${action}"
        return 0
    fi
    return 1
}

# Apple Musicを制御
control_music() {
    local action="$1"
    if pgrep -x "Music" > /dev/null; then
        osascript -e "tell application \"Music\" to ${action}"
        return 0
    fi
    return 1
}

# 音楽を再生
play_music() {
    case "$BGM_APP" in
        spotify)
            control_spotify "play" && echo "Spotify: Playing" && return
            ;;
        music)
            control_music "play" && echo "Music: Playing" && return
            ;;
        *)
            # 自動検出: Spotifyを優先
            control_spotify "play" && echo "Spotify: Playing" && return
            control_music "play" && echo "Music: Playing" && return
            ;;
    esac
    echo "No music app is running"
}

# 音楽を一時停止
pause_music() {
    case "$BGM_APP" in
        spotify)
            control_spotify "pause" && echo "Spotify: Paused" && return
            ;;
        music)
            control_music "pause" && echo "Music: Paused" && return
            ;;
        *)
            # 両方を一時停止
            control_spotify "pause" 2>/dev/null && echo "Spotify: Paused"
            control_music "pause" 2>/dev/null && echo "Music: Paused"
            ;;
    esac
}

# イベントに応じた制御
case "${POMODORO_EVENT:-}" in
    work_start)
        echo "Starting BGM for work session..."
        play_music
        ;;
    break_start)
        echo "Pausing BGM for break..."
        pause_music
        ;;
    break_end)
        echo "Resuming BGM for next session..."
        play_music
        ;;
    session_complete)
        echo "Session complete, pausing BGM..."
        pause_music
        ;;
    *)
        echo "No BGM action for event: ${POMODORO_EVENT:-unknown}"
        ;;
esac

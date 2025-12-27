#!/bin/bash
set -e

echo "=== uti アンインストールスクリプト ==="
echo ""

# 権限チェック
if [ "$EUID" -ne 0 ]; then
    echo "エラー: このスクリプトはroot権限で実行してください"
    echo "使用方法: sudo ./uninstall.sh"
    exit 1
fi

# サービスの停止と無効化
echo "[1/3] サービスを停止中..."
systemctl stop double-ctrl.service || true
systemctl disable double-ctrl.service || true

# systemdサービスの削除
echo "[2/3] systemd サービスを削除中..."
rm -f /etc/systemd/system/double-ctrl.service
systemctl daemon-reload

# daemonの削除
echo "[3/3] daemon を削除中..."
rm -f /usr/local/bin/double-ctrl

echo ""
echo "✓ アンインストール完了!"
echo ""

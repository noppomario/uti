#!/bin/bash
set -e

echo "=== uti インストールスクリプト ==="
echo ""

# 権限チェック
if [ "$EUID" -ne 0 ]; then
    echo "エラー: このスクリプトはroot権限で実行してください"
    echo "使用方法: sudo ./install.sh"
    exit 1
fi

# daemonのビルド
echo "[1/4] daemon をビルド中..."
cd daemon
cargo build --release
cd ..

# daemonのインストール
echo "[2/4] daemon をインストール中..."
cp daemon/target/release/double-ctrl /usr/local/bin/
chmod +x /usr/local/bin/double-ctrl

# systemdサービスのインストール
echo "[3/4] systemd サービスを設定中..."
cp daemon/systemd/double-ctrl.service /etc/systemd/system/
systemctl daemon-reload

# サービスの有効化
echo "[4/4] サービスを有効化中..."
systemctl enable double-ctrl.service
systemctl start double-ctrl.service

echo ""
echo "✓ インストール完了!"
echo ""
echo "サービスステータス:"
systemctl status double-ctrl.service --no-pager
echo ""
echo "次のステップ:"
echo "  1. cd app"
echo "  2. bun install"
echo "  3. bun run tauri:dev"
echo ""

# uti

Ctrl 2回押しで表示/非表示を切り替えるデスクトップツール

## 概要

**uti**は、Ctrlキーを2回素早く押すことでウィンドウの表示/非表示を切り替えられる、Linux向けのデスクトップユーティリティです。

### 技術スタック

- **Frontend**: React 19 + TypeScript + Tailwind CSS v4
- **Backend**: Tauri 2 (Rust)
- **Daemon**: Rust (evdev + zbus)
- **対象OS**: Linux (Fedora 43 GNOME/Wayland)

## アーキテクチャ

```
[double-ctrl daemon]  ← root/input権限で動作
  ↓ (evdevでCtrl監視)
  ↓ (300ms以内の2回押し検出)
  ↓ (D-Bus Signal送信)
[Tauri app]           ← 通常ユーザー権限
  ↓ (D-Bus受信)
  ↓ (window.hide/show)
```

## セットアップ

### 必要な環境

- Rust (latest stable)
- Bun (latest)
- Linux with evdev support
- D-Bus session bus

### 1. リポジトリのクローン

```bash
git clone https://github.com/noppomario/uti.git
cd uti
```

### 2. Daemonのビルドとインストール

```bash
cd daemon
cargo build --release

# インストール
sudo cp target/release/double-ctrl /usr/local/bin/
sudo cp systemd/double-ctrl.service /etc/systemd/system/

# サービスの有効化と起動
sudo systemctl daemon-reload
sudo systemctl enable --now double-ctrl.service

# ステータス確認
sudo systemctl status double-ctrl.service
```

### 3. Tauri Appのビルドと実行

```bash
cd ../app
bun install
bun run tauri:dev   # 開発モード
bun run tauri:build # リリースビルド
```

## 開発

### Daemon開発

```bash
cd daemon
sudo cargo run  # root権限が必要
```

### App開発

```bash
cd app
bun install
bun run tauri:dev
```

### デバッグ

#### D-Busメッセージの監視

```bash
dbus-monitor "interface='io.github.noppomario.uti.DoubleTap'"
```

#### デバイス確認

```bash
ls -l /dev/input/event*
```

#### ログ確認

```bash
journalctl -u double-ctrl.service -f
```

## プロジェクト構成

```
uti/
├── daemon/                          # Ctrl検出デーモン
│   ├── Cargo.toml
│   ├── systemd/
│   │   └── double-ctrl.service
│   └── src/
│       └── main.rs
│
└── app/                             # Tauri GUIアプリ
    ├── package.json
    ├── vite.config.ts
    ├── src/                         # React frontend
    │   ├── App.tsx
    │   ├── main.tsx
    │   └── index.css
    └── src-tauri/                   # Rust backend
        ├── Cargo.toml
        └── src/
            └── main.rs
```

## トラブルシューティング

### Daemonが起動しない

```bash
# ログを確認
sudo journalctl -u double-ctrl.service -n 50

# 手動実行でテスト
cd daemon
sudo cargo run
```

### D-Bus通信ができない

```bash
# Session Busの確認
echo $DBUS_SESSION_BUS_ADDRESS

# 手動でSignalを送信してテスト
dbus-send --session \
  --type=signal \
  /io/github/noppomario/uti/DoubleTap \
  io.github.noppomario.uti.DoubleTap.Triggered
```

## TODO

- [ ] デバイス検出の堅牢化（複数キーボード対応）
- [ ] 設定ファイル対応（間隔調整など）
- [ ] トレイアイコン対応
- [ ] ウィンドウ位置・サイズの永続化
- [ ] Windows対応

## ライセンス

MIT License

## 作者

Anonymous

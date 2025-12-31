日本語 | [English](README.md)

<div align="center">

# uti

<img src="app/src-tauri/icons/icon.png" alt="uti icon" width="128">

[![Release](https://img.shields.io/github/v/release/noppomario/uti)](https://github.com/noppomario/uti/releases)
[![CI](https://github.com/noppomario/uti/actions/workflows/ci.yml/badge.svg)](https://github.com/noppomario/uti/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/tauri-2.x-blue.svg)](https://tauri.app/)
[![Platform](https://img.shields.io/badge/platform-Linux-lightgrey.svg)](https://github.com/noppomario/uti)

> Ctrl2回押しでトグルするLinux向けクリップボードマネージャ

</div>

## なぜ uti?

**Wayland環境での課題を解決するために作りました。**

- **Waylandの制限**: X11とは異なり、Waylandではグローバルキーボードショート
  カットの設定が制限されています。Ctrl2回押しでアプリを起動するようなショート
  カットは、標準のデスクトップ設定では実現できません。

- **既存ツールの不在**: Waylandネイティブで動作し、かつCtrl2回押しで起動できる
  クリップボードマネージャは見つかりませんでした。

- **デーモンアーキテクチャ**: 専用のデーモンがキーボード入力を監視することで、
  デスクトップ環境に依存しない柔軟なショートカットを実現。将来的にはカスタマイ
  ズ可能なトリガー設定も検討中です。

## スクリーンショット

![uti screenshot](docs/assets/screenshot.png)

## 機能

- **Ctrl2回押しでトグル**: Ctrlを素早く2回押す（300ms以内）とウィンドウが表示/非表示
- **クリップボード履歴**: クリップボードの履歴を保存し、すぐにアクセス可能
- **システムトレイ**: トレイアイコンからバックグラウンドで制御
- **自動起動**: ログイン時の自動起動オプション
- **セルフアップデート**: `uti update`コマンドまたはトレイメニューから更新

## クイックスタート

ワンコマンドでインストール:

```bash
curl -fsSL https://raw.githubusercontent.com/noppomario/uti/main/install.sh | bash
```

インストール後、ログアウトして再ログインし、`uti`を実行してください。

## 使い方

### 基本操作

1. **Ctrlを素早く2回押す**とウィンドウの表示/非表示が切り替わります
2. クリップボード履歴をクリックするとコピーされます
3. 矢印キーで移動、Enterで選択

### システムトレイ

トレイアイコンを右クリックするとオプションが表示されます:

- **Show/Hide**: ウィンドウの表示/非表示を切り替え
- **Auto-start**: ログイン時の自動起動を有効/無効
- **Check for Updates...**: 新しいバージョンを確認
- **GitHub**: プロジェクトページを開く
- **Quit**: アプリケーションを終了

### アップデート

アップデートを確認してインストール:

```bash
uti update
```

確認のみ（インストールしない）:

```bash
uti update --check
```

## 設定

設定ファイル: `~/.config/uti/config.json`

```json
{
  "theme": "dark",
  "clipboardHistoryLimit": 50,
  "showTooltip": true,
  "tooltipDelay": 500
}
```

| オプション              | 型      | 初期値  | 説明                           |
| ----------------------- | ------- | ------- | ------------------------------ |
| `theme`                 | string  | `dark`  | UIテーマ: `dark` または `light` |
| `clipboardHistoryLimit` | number  | `50`    | 保存するクリップボード履歴の最大数 |
| `showTooltip`           | boolean | `true`  | ホバー時にツールチップを表示   |
| `tooltipDelay`          | number  | `500`   | ツールチップの遅延時間（ms）   |

## 技術スタック

| レイヤー | 技術                                |
| -------- | ----------------------------------- |
| Frontend | React 19 + TypeScript + Tailwind v4 |
| Backend  | Tauri 2 + Rust                      |
| Daemon   | Rust + evdev + D-Bus                |

## 動作環境

- **OS**: Linux（Fedora 43以上推奨）
- **デスクトップ**: GNOME (Wayland/X11)、KDE、XFCE
- **アーキテクチャ**: x86_64

### GNOMEユーザー向け

GNOME 43以降ではシステムトレイにAppIndicator拡張機能が必要です:

1. Extension Managerをインストール（未インストールの場合）:
   - GNOME Softwareを開く
   - "Extension Manager"を検索してインストール

2. AppIndicator拡張機能をインストール:
   - Extension Managerを開く
   - "AppIndicator and KStatusNotifierItem Support"を検索
   - インストールをクリック

3. ログアウトして再ログイン

<details>
<summary><strong>トラブルシューティング</strong></summary>

### デーモンが起動しない

inputグループに所属しているか確認:

```bash
groups  # "input"が含まれているはず
```

含まれていない場合、追加してログアウト/ログイン:

```bash
sudo usermod -aG input $USER
```

デーモンの状態を確認:

```bash
systemctl --user status double-ctrl.service
journalctl --user -u double-ctrl.service -n 50
```

### トレイアイコンが表示されない（GNOME）

Extension Manager経由でAppIndicator拡張機能をインストールしてください
（上記のGNOMEユーザー向けセクションを参照）。

</details>

## アンインストール

```bash
sudo dnf remove uti double-ctrl
```

これによりデーモンサービスも自動的に停止・無効化されます。

inputグループから自分を削除（オプション）:

```bash
sudo gpasswd -d $USER input
```

## 開発

開発環境のセットアップとコントリビューションガイドラインについては、
[DEVELOPMENT.md](docs/DEVELOPMENT.md)を参照してください。

## 既知の制限事項

- **Waylandでウィンドウがドックに表示される**: Waylandでは、ウィンドウが表示
  されているときドックに表示されます。これはTauriの制限です
  ([#9829](https://github.com/tauri-apps/tauri/issues/9829))。
- **ウィンドウの位置**: ウィンドウは常に画面中央に表示されます（Waylandでは
  カーソル相対位置がサポートされていません）。

## ライセンス

[MIT](LICENSE)

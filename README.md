# AtCoderNinja

サンプルケースの自動実行・ACであれば自動提出を可能にするCLIです.

設定の仕方によって, C++やPythonなど, 様々な言語での環境に対応することができます.

このプロジェクトは開発段階です.
気になる点があったら, [issue](https://github.com/UUGTech/AtCoderNinja/issues)や[pull request](https://github.com/UUGTech/AtCoderNinja/pulls)にお願いします!

## !!言語アップデート対応

言語アップデートに対応しました。最新のmasterブランチにしてください。また、config内のlang_idやlang_nameを[LANG_ID一覧](./LANG_ID.md)に記載されているものに合わせてください。
古いままだと提出が出来ません。

## インストール

以下のコマンドでインストールできます

```bash
cargo install --git https://github.com/UUGTech/AtCoderNinja
```

これで, `ac-ninja`コマンドが使えるようになります.

### **おススメ**

`.bashrc`などで,`acn`などのエイリアスを用意すると, コンテスト参加中のタイプ数が減って嬉しいです.

```bash
alias acn='ac-ninja'
```

## アンインストール

以下のコマンドでアンインストールできます

```bash
cargo uninstall ac-ninja
```

## 設定

`~/.config/ac-ninja/config.toml`に各種設定を記述します.
設定の詳しい内容は[CONFIG.md](./CONFIG.md)を参照してください.

## 使い方

- ログイン

AtCoderNinjaの機能を十分に使うためには, AtCoderにログインする必要があります. 以下のコマンドでログインできます.

```bash
ac-ninja login
```

usernameやpasswordは保存されません. セッション情報が`~/.ac-ninja/session.txt`に保存されます.

- ログアウト

```bash
ac-ninja logout
```

のようにすることで, 上記`~/.ac-ninija/session.txt`は削除され, ログアウトします.

- サンプルでACであればそのまま提出する場合

``` bash
ac-ninja a
```

`ac-ninja <problem_id>`のように, 問題を指定します.

- 提出はせずに、ローカルでのみ実行する場合

``` bash
ac-ninja a -l
```

のように`-l`オプションをつけることで, 提出は行いません.

- サンプルの結果に関わらず提出をする場合

``` bash
ac-ninja a -f
```

のように`-f`オプションをつけることで, サンプルの結果がACでなくても提出を行います.
これは, 正解が複数あり得る場合などに役立つオプションです.

- 手動の入力で確かめたい場合

```bash
ac-ninja a -i
```

のようにすると, サンプルケースではなく, 手動の標準入力で動作を確認することが出来ます.
もちろん提出は行われません.

また、

``` bash
ac-ninja a -i < ./input.txt
```

のようにすることで, 他のファイルを入力に使うこともできます.

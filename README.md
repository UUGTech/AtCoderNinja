# AtCoderNinja

サンプルケースの自動実行と、ACならソースをクリップボードにコピーするCLIです.
提出はブラウザで行います（Turnstileのため自動提出は不可）。

設定の仕方によって, C++やPythonなど, 様々な言語での環境に対応することができます.

このプロジェクトは開発段階です.
気になる点があったら, [issue](https://github.com/UUGTech/AtCoderNinja/issues)や[pull request](https://github.com/UUGTech/AtCoderNinja/pulls)にお願いします!

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

ブラウザでログインして、`REVEL_SESSION` を貼り付けます。
セッション情報が`~/.ac-ninja/session.txt`に保存されます.

- ログアウト

```bash
ac-ninja logout
```

のようにすることで, 上記`~/.ac-ninija/session.txt`は削除され, ログアウトします.

- ログイン状態の確認

```bash
ac-ninja login-check
```

ログインCookieが有効かどうかを確認できます.

- サンプルでACならソースをクリップボードにコピーする場合

``` bash
ac-ninja a
```

`ac-ninja <problem_id>`のように, 問題を指定します.

- クリップボードにコピーせず、ローカルでのみ実行する場合

``` bash
ac-ninja a -l
```

のように`-l`オプションをつけることで, クリップボードコピーは行いません.

- サンプルの結果に関わらずコピーする場合

``` bash
ac-ninja a -f
```

のように`-f`オプションをつけることで, サンプルの結果がACでなくてもコピーを行います.
これは, 正解が複数あり得る場合などに役立つオプションです.

- 手動の入力で確かめたい場合

```bash
ac-ninja a -i
```

のようにすると, サンプルケースではなく, 手動の標準入力で動作を確認することが出来ます.
もちろんコピーは行われません.

また、

``` bash
ac-ninja a -i < ./input.txt
```

のようにすることで, 他のファイルを入力に使うこともできます.

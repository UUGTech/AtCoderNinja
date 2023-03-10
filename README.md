# [WIP]AtCoderNinja

## 設定
`~/.config/ac-ninja/config.toml`に各種設定を記述します.
設定の詳しい内容は[CONFIG.md](./config.md)を参照してください.

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

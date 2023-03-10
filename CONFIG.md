# 設定

設定に必須な情報はcontest_dir, source_file_path, need_to_compile, execute_commandです.

```txt
contest_dir:          ac-ninjaを実行するディレクトリです.
                      {{contesty_type}},{{contest_id}}を特定できる必要があります
--------------------------------------------------------------------------------------
source_file_path:     ac-ninjaで提出するファイルのパスです.
--------------------------------------------------------------------------------------
need_to_compile:      プログラムの実行にコンパイルが必要かどうかを指定します.
                      trueの場合, {{compile_command}}を指定する必要があります.
--------------------------------------------------------------------------------------
execute_command:      プログラムを実行するためのコマンドです.
```

ファイルパスや, 実行コマンドには{{変数}}を含むことができます.

`{{contest_type}}`, `{{contest_id}}`, `{{problem_id}}`およびそれらの派生以外の変数は
config.toml内で解決可能である必要があります.

`{{CONTEST_TYPE}}`のように大文字で記述すると, `"ABC","ARC","AGC"`のように
contest_typeが大文字であることを表します.

また, contest_idに関しては`{{contest_id_0_pad}}`とすることで, `"009"`のように
AtCoderのURLに沿った0埋めを表すことが出来ます.

## <設定例>

AtCoderにC++で参戦している人の例です.

ディレクトリ構成が ~/CompetitiveProgramming/ABC/059/b.cpp のような場合には,
以下のような設定が考えられます

```toml
work_space = "~/CompetitiveProgramming"
need_to_compile = true
contest_dir = "{{work_space}}/{{CONTEST_TYPE}}/{{contest_id_0_pad}}"
output_file_path = "{{contest_dir}}/a.out"
source_file_path = "{{contest_dir}}/{{problem_id}}.cpp"
compile_command = "g++ {{source_file_path}} -std=c++17 -o {{output_file_path}}"
execute_command = "{{output_file_path}}"
```

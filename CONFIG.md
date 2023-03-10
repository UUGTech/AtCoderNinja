# 設定

設定に必須な情報はcontest_dir, source_file_path, need_to_compile, execute_command,
(language_id または language_name)です.

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
--------------------------------------------------------------------------------------
language_id:          ac-ninjaでの提出に用いる言語のidです.
                      AtCoderの提出セレクトボックスをディベロッパーツールから見ることで
                      確認できますが, [早見表](./LANG_ID.md)が便利です.
--------------------------------------------------------------------------------------
language_name:        language_idの代わりに, language_nameを指定することができます.
                      AtCoderの提出言語セレクトボックスの表示の通りに指定してください.
                      \"C++(GCC 9.2.1)\", \"Python (3.8.2)\", \"Rust (1.42.0)\"など.
                      こちらも, [早見表](./LANG_ID.md)の文字列をコピペすると便利です.
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
language_id = 4003      # language_nameの場合 "C++ (GCC 9.2.1)"
```

以下はPythonでの設定例です

```toml
work_space = "~/CompetitiveProgramming"
need_to_compile = false
contest_dir = "{{work_space}}/{{CONTEST_TYPE}}/{{contest_id_0_pad}}"
source_file_path = "{{contest_dir}}/{{problem_id}}/main.py"
execute_command = "python3 {{source_file_path}}"
language_name = "Python (3.8.2)"    # language_idの場合 4006
```


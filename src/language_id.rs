use anyhow::{anyhow, Result};

pub fn lang_to_id(lang_name: &str) -> Result<i64> {
    match lang_name {
        "C (GCC 9.2.1)" => Ok(4001),
        "C (Clang 10.0.0)" => Ok(4002),
        "C++ (GCC 9.2.1)" => Ok(4003),
        "C++ (Clang 10.0.0)" => Ok(4004),
        "Java (OpenJDK 11.0.6)" => Ok(4005),
        "Python (3.8.2)" => Ok(4006),
        "Bash (5.0.11)" => Ok(4007),
        "bc (1.07.1)" => Ok(4008),
        "Awk (GNU Awk 4.1.4)" => Ok(4009),
        "C# (.NET Core 3.1.201)" => Ok(4010),
        "C# (Mono-mcs 6.8.0.105)" => Ok(4011),
        "C# (Mono-csc 3.5.0)" => Ok(4012),
        "Clojure (1.10.1.536)" => Ok(4013),
        "Crystal (0.33.0)" => Ok(4014),
        "D (DMD 2.091.0)" => Ok(4015),
        "D (GDC 9.2.1)" => Ok(4016),
        "D (LDC 1.20.1)" => Ok(4017),
        "Dart (2.7.2)" => Ok(4018),
        "dc (1.4.1)" => Ok(4019),
        "Erlang (22.3)" => Ok(4020),
        "Elixir (1.10.2)" => Ok(4021),
        "F# (.NET Core 3.1.201)" => Ok(4022),
        "F# (Mono 10.2.3)" => Ok(4023),
        "Forth (gforth 0.7.3)" => Ok(4024),
        "Fortran (GNU Fortran 9.2.1)" => Ok(4025),
        "Go (1.14.1)" => Ok(4026),
        "Haskell (GHC 8.8.3)" => Ok(4027),
        "Haxe (4.0.3); js" => Ok(4028),
        "Haxe (4.0.3); Java" => Ok(4029),
        "JavaScript (Node.js 12.16.1)" => Ok(4030),
        "Julia (1.4.0)" => Ok(4031),
        "Kotlin (1.3.71)" => Ok(4032),
        "Lua (Lua 5.3.5)" => Ok(4033),
        "Lua (LuaJIT 2.1.0)" => Ok(4034),
        "Dash (0.5.8)" => Ok(4035),
        "Nim (1.0.6)" => Ok(4036),
        "Objective-C (Clang 10.0.0)" => Ok(4037),
        "Common Lisp (SBCL 2.0.3)" => Ok(4038),
        "OCaml (4.10.0)" => Ok(4039),
        "Octave (5.2.0)" => Ok(4040),
        "Pascal (FPC 3.0.4)" => Ok(4041),
        "Perl (5.26.1)" => Ok(4042),
        "Raku (Rakudo 2020.02.1)" => Ok(4043),
        "PHP (7.4.4)" => Ok(4044),
        "Prolog (SWI-Prolog 8.0.3)" => Ok(4045),
        "PyPy2 (7.3.0)" => Ok(4046),
        "PyPy3 (7.3.0)" => Ok(4047),
        "Racket (7.6)" => Ok(4048),
        "Ruby (2.7.1)" => Ok(4049),
        "Rust (1.42.0)" => Ok(4050),
        "Scala (2.13.1)" => Ok(4051),
        "Java (OpenJDK 1.8.0)" => Ok(4052),
        "Scheme (Gauche 0.9.9)" => Ok(4053),
        "Standard ML (MLton 20130715)" => Ok(4054),
        "Swift (5.2.1)" => Ok(4055),
        "Text (cat 8.28)" => Ok(4056),
        "TypeScript (3.8)" => Ok(4057),
        "Visual Basic (.NET Core 3.1.101)" => Ok(4058),
        "Zsh (5.4.2)" => Ok(4059),
        "COBOL - Fixed (OpenCOBOL 1.1.0)" => Ok(4060),
        "COBOL - Free (OpenCOBOL 1.1.0)" => Ok(4061),
        "Brainfuck (bf 20041219)" => Ok(4062),
        "Ada2012 (GNAT 9.2.1)" => Ok(4063),
        "Unlambda (2.0.0)" => Ok(4064),
        "Cython (0.29.16)" => Ok(4065),
        "Sed (4.4)" => Ok(4066),
        "Vim (8.2.0460)" => Ok(4067),
        _ => Err(anyhow!("Unknown language name error.")),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lang_to_id() {
        assert_eq!(4003, lang_to_id("C++ (GCC 9.2.1)").unwrap());
        assert_eq!(4067, lang_to_id("Vim (8.2.0460)").unwrap());
        assert!(lang_to_id("Vim").is_err());
    }
}


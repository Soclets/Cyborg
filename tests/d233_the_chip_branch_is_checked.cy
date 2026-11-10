// A chip's branch is checked, though the oracle doesn't follow it.
// trace: D233
@config(default);
import std;

fn.main [] -> void:
    let $a: i64 = 6;
    let $b: i64 = 7;
    on.target aarch64:
        let $wrong: i32 = $a;
    else:
        std::io::fmt_print("portable{newln}");
    end on.target
    return void;
end fn.main

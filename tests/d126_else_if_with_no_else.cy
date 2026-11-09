// trace: D126
import std;
@config(default);

fn.main [] -> void:
    let $n: i32 = 0;
    if $n > 0:
        std::io::fmt_print("positive{newln}");
    else if $n < 0:
        std::io::fmt_print("negative{newln}");
    end if
    std::io::fmt_print("carried on{newln}");
    return void;
end fn.main

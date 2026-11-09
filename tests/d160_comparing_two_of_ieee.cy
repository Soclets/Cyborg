// trace: D160, D159
import std;
@config(default);

fn.main [] -> void:
    let $a: f64.ieee = 1.5;
    let $b: f64.ieee = 2.5;
    if $a at_most $b:
        std::io::fmt_print("at most{newln}");
    end if
    return void;
end fn.main

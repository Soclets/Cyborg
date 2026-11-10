// trace: D131
import std;

fn.main [none] -> void:
    let $b: bool = true;
    let $c: bool = false == (not $b);
    if $c:
        std::io::fmt_print("grouped{newln}");
    end if
    return void;
end fn.main

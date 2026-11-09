// trace: D127, D75
import std;
@config(default);

fn true [] -> i32:
    return 1;
end fn true

fn.main [] -> void:
    let $a: i32 = my::true();
    let $b: bool = true;
    std::io::fmt_print("{}{newln}", $a);
    if $b:
        std::io::fmt_print("still a bool{newln}");
    end if
    return void;
end fn.main

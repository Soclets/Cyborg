// trace: D126
import std;

fn size [$n: i32] -> void:
    if $n > 100:
        std::io::fmt_print("huge{newln}");
    else if $n > 0:
        std::io::fmt_print("big{newln}");
    else:
        std::io::fmt_print("small{newln}");
    end if
    return void;
end fn size

fn.main [none] -> void:
    size(101);
    size(1);
    size(0);
    return void;
end fn.main

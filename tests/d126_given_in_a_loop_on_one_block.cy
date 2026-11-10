// trace: D126, D2
import std;

fn eat [$s: Given str] -> void:
    return void;
end fn eat

fn.main [none] -> void:
    let $xs: std::array<i32>[len: 2 items] = [1, 2];
    let $s: str = "a";
    for $x in $xs:
        if $x == 1:
            std::io::fmt_print("one{newln}");
        else:
            eat(Give $s);
        end if
    end for
    return void;
end fn.main

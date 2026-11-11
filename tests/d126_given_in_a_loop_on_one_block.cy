// trace: D126, D2
import std;
import os;

fn eat [$s: Given str] returns void:
    return void;
end fn eat

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $xs: std::array<i32>[len: 2 items] set_to [1, 2];
    let $s: str set_to "a";
    for $x in $xs:
        if $x == 1:
            std::io::fmt_print(Lend.mut $out, "one{newln}");
        else:
            eat(Give $s);
        end if
    end for
    return void;
end fn.main

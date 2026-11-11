// trace: D139, D20
import std;
import os;

fn f [none] returns i32[min: 0, max: 9] fails [a]:
    fail a;
end fn f

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $big: i32 = 50;
    std::io::fmt_print(Lend.mut $out, "before{newln}");
    let $n: i32[min: 0, max: 9] = f() otherwise:
        case a:
            instead $big;
    end otherwise
    std::io::fmt_print(Lend.mut $out, "after{newln}");
    return void;
end fn.main

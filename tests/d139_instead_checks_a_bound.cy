// trace: D139, D20
@config(default);
import std;

fn f [] -> i32[min: 0, max: 9] fails [a]:
    fail a;
end fn f

fn.main [] -> void:
    let $big: i32 = 50;
    std::io::fmt_print("before{newln}");
    let $n: i32[min: 0, max: 9] = f() otherwise:
        case a:
            instead $big;
    end otherwise
    std::io::fmt_print("after{newln}");
    return void;
end fn.main

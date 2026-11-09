// trace: D139, D20
import std;
@config(default);

fn f [] -> i32[min: 0, max: 9]:
    fails [a];

    fail a;
end fn f

fn.main [] -> void:
    let $big: i32 = 50;
    std::println("before");
    let $n: i32[min: 0, max: 9] = f() otherwise:
        case a:
            instead $big;
    end otherwise
    std::println("after");
    return void;
end fn.main

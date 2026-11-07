// Each `instead` belongs to the arm around it.
// trace: D139
import std;

fn f [] -> i32:
    fails [a];

    fail a;
end fn f

fn g [] -> i32:
    fails [b];

    fail b;
end fn g

fn.main [] -> void:
    let $n: i32 = f() otherwise:
        case a:
            let $m: i32 = g() otherwise:
                case b:
                    instead 2;
            end otherwise
            instead $m + 1;
    end otherwise
    std::println($n as str);
    return void;
end fn.main

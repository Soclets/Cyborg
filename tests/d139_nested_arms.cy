// Each `instead` belongs to the arm around it.
// trace: D139
import std;
import os;

fn f [none] -> i32 fails [a]:
    fail a;
end fn f

fn g [none] -> i32 fails [b]:
    fail b;
end fn g

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $n: i32 = f() otherwise:
        case a:
            let $m: i32 = g() otherwise:
                case b:
                    instead 2;
            end otherwise
            instead $m + 1;
    end otherwise
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    return void;
end fn.main

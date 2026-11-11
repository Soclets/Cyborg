// The name is for reading its data, and nothing else has its type.
// trace: D139
import std;
import os;

fn f [none] -> void fails [a[val: i32]]:
    return void;
end fn f

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    f() otherwise:
        case a $x:
            std::io::fmt_print(Lend.mut $out, "{}{newln}", $x);
    end otherwise
    return void;
end fn.main

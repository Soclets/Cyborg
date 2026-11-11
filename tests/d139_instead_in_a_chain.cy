// `instead` can be anywhere in its arm, and each way through has to end somewhere.
// trace: D139, D126
import std;
import os;

fn f [none] returns i32 fails [a[val: i32]]:
    fail a[val = 2];
end fn f

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $n: i32 = f() otherwise:
        case a $x:
            if $x.val above 1:
                instead 100;
            else:
                instead 200;
            end if
    end otherwise
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    return void;
end fn.main

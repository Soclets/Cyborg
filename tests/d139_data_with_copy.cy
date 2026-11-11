// trace: D139, D3
import std;
import os;

fn f [none] returns str fails [a[label: str]]:
    fail a[label = "from the failure"];
end fn f

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $s: str = f() otherwise:
        case a $x:
            instead Copy $x.label;
    end otherwise
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $s);
    return void;
end fn.main

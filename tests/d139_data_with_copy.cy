// trace: D139, D3
import std;
import os;

fn f [none] -> str[len.max: 32 bytes] fails [a[label: str[len.max: 32 bytes]]]:
    fail a[label = "from the failure"];
end fn f

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $s: str[len.max: 32 bytes] = f() otherwise:
        case a $x:
            instead Copy $x.label;
    end otherwise
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $s);
    return void;
end fn.main

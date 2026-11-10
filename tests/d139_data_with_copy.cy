// trace: D139, D3
import std;

fn f [none] -> str fails [a[label: str]]:
    fail a[label = "from the failure"];
end fn f

fn.main [none] -> void:
    let $s: str = f() otherwise:
        case a $x:
            instead Copy $x.label;
    end otherwise
    std::io::fmt_print("{}{newln}", $s);
    return void;
end fn.main

// trace: D139, D3
import std;

fn f [] -> str:
    fails [a[label: str]];

    fail a[label = "from the failure"];
end fn f

fn.main [] -> void:
    let $s: str = f() otherwise:
        case a $x:
            instead Copy $x.label;
    end otherwise
    std::println(Lend $s);
    return void;
end fn.main

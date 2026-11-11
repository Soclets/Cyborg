// trace: D139, D3

fn f [none] -> str fails [a[label: str]]:
    return "x";
end fn f

fn.main [none] -> void:
    let $s: str = f() otherwise:
        case a $x:
            instead $x.label;
    end otherwise
    return void;
end fn.main

// trace: D139, D3
fn f [] -> str:
    fails [a[label: str]];

    return "x";
end fn f

fn.main [] -> void:
    let $s: str = f() otherwise:
        case a $x:
            instead $x.label;
    end otherwise
    return void;
end fn.main

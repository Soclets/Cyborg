// trace: D139, D3

fn f [none] -> str[len.max: 32 bytes] fails [a[label: str[len.max: 32 bytes]]]:
    return "x";
end fn f

fn.main [none] -> void:
    let $s: str[len.max: 32 bytes] = f() otherwise:
        case a $x:
            instead $x.label;
    end otherwise
    return void;
end fn.main

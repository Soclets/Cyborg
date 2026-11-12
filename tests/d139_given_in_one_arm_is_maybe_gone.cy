// The call may not fail, so what an arm gives away is only maybe gone.
// trace: D139, D2

fn f [none] -> str[len.max: 32 bytes] fails [a]:
    return "x";
end fn f

fn.main [none] -> void:
    let $s: str[len.max: 32 bytes] = "y";
    let $t: str[len.max: 32 bytes] = f() otherwise:
        case a:
            instead Give $s;
    end otherwise
    let $u: str[len.max: 32 bytes] = Give $s;
    return void;
end fn.main

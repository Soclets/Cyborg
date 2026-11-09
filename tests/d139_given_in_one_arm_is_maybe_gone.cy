// The call may not fail, so what an arm gives away is only maybe gone.
// trace: D139, D2
@config(default);

fn f [] -> str:
    fails [a];

    return "x";
end fn f

fn.main [] -> void:
    let $s: str = "y";
    let $t: str = f() otherwise:
        case a:
            instead Give $s;
    end otherwise
    let $u: str = Give $s;
    return void;
end fn.main

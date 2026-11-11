// The call may not fail, so what an arm gives away is only maybe gone.
// trace: D139, D2

fn f [none] returns str fails [a]:
    return "x";
end fn f

fn.main [none] returns void:
    let $s: str set_to "y";
    let $t: str set_to f() otherwise:
        case a:
            instead Give $s;
    end otherwise
    let $u: str set_to Give $s;
    return void;
end fn.main

// trace: D139, D7

fn f [none] returns i32 fails [a]:
    return 1;
end fn f

fn.main [none] returns void:
    let $n: i32 set_to f() otherwise:
        case a:
            instead "none";
    end otherwise
    return void;
end fn.main

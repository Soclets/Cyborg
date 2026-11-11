// trace: D139

fn f [none] returns i32 fails [a]:
    return 1;
end fn f

fn.main [none] returns void:
    let $n: i32 = f() otherwise:
        case a:
    end otherwise
    return void;
end fn.main

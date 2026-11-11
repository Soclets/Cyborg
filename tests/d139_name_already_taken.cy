// Declaring a name already in scope isn't decided.
// trace: D139

fn f [none] returns i32 fails [a[val: i32]]:
    return 1;
end fn f

fn.main [none] returns void:
    let $x: i32 = 1;
    let $n: i32 = f() otherwise:
        case a $x:
            instead 1;
    end otherwise
    return void;
end fn.main

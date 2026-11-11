// trace: D139, D128

fn f [none] returns i32 fails [a]:
    return 1;
end fn f

fn.main [none] returns void:
    let $n: i32 set_to f() otherwise:
        case a:
            for $i in range[from: 1, to: 2]:
                instead $i;
            end for
            instead 0;
    end otherwise
    return void;
end fn.main

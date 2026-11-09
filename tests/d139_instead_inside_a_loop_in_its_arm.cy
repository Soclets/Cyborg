// trace: D139, D128
@config(default);

fn f [] -> i32 fails [a]:
    return 1;
end fn f

fn.main [] -> void:
    let $n: i32 = f() otherwise:
        case a:
            for $i in range[from: 1, to: 2]:
                instead $i;
            end for
            instead 0;
    end otherwise
    return void;
end fn.main

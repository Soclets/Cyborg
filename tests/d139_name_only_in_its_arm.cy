// trace: D139
@config(default);

fn f [] -> i32:
    fails [a[val: i32], b];

    return 1;
end fn f

fn.main [] -> void:
    let $n: i32 = f() otherwise:
        case a $x:
            instead 1;
        case b:
            instead $x.val;
    end otherwise
    return void;
end fn.main

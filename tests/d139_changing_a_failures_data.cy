// trace: D139, D125
@config(default);

fn f [] -> void:
    fails [a[val: i32]];

    return void;
end fn f

fn.main [] -> void:
    f() otherwise:
        case a $x:
            change $x.val = 2;
    end otherwise
    return void;
end fn.main

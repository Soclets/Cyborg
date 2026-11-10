// trace: D139, D125

fn f [none] -> void fails [a[val: i32]]:
    return void;
end fn f

fn.main [none] -> void:
    f() otherwise:
        case a $x:
            change $x.val = 2;
    end otherwise
    return void;
end fn.main

// trace: D139, D125

fn f [none] returns void fails [a[val: i32]]:
    return void;
end fn f

fn.main [none] returns void:
    f() otherwise:
        case a $x:
            change $x.val set_to 2;
    end otherwise
    return void;
end fn.main

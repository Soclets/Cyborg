// trace: D139

fn f [none] -> i32 fails [a]:
    return 1;
end fn f

fn.main [none] -> void:
    let $n: i32 = f() otherwise:
        case a:
    end otherwise
    return void;
end fn.main

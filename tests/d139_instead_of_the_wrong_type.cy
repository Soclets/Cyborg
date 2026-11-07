// trace: D139, D7
fn f [] -> i32:
    fails [a];

    return 1;
end fn f

fn.main [] -> void:
    let $n: i32 = f() otherwise:
        case a:
            instead "none";
    end otherwise
    return void;
end fn.main

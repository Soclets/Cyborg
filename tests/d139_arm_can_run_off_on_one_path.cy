// trace: D139
@config(default);

fn f [] -> i32:
    fails [a];

    return 1;
end fn f

fn.main [] -> void:
    let $n: i32 = f() otherwise:
        case a:
            if 1 < 2:
                instead 100;
            end if
    end otherwise
    return void;
end fn.main

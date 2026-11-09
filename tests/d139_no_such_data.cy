// trace: D139
@config(default);

fn f [] -> i32:
    fails [odd];

    return 1;
end fn f

fn.main [] -> void:
    let $n: i32 = f() otherwise:
        case odd $o:
            instead $o.val;
    end otherwise
    return void;
end fn.main

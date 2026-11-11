// trace: D139

fn f [none] returns i32 fails [odd]:
    return 1;
end fn f

fn.main [none] returns void:
    let $n: i32 set_to f() otherwise:
        case odd $o:
            instead $o.val;
    end otherwise
    return void;
end fn.main

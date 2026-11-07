// trace: D139, D2
struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn f [] -> i32:
    fails [a];

    return 1;
end fn f

fn.main [] -> void:
    let $n: i32 = f() otherwise:
        case a:
            let $g: open_file = open_file[fd = 1];
            instead 0;
    end otherwise
    return void;
end fn.main

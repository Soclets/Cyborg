// trace: D139, D2

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn f [none] returns i32 fails [a]:
    return 1;
end fn f

fn.main [none] returns void:
    let $n: i32 set_to f() otherwise:
        case a:
            let $g: open_file set_to open_file[fd set_to 1];
            instead 0;
    end otherwise
    return void;
end fn.main

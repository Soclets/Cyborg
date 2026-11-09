// trace: D2
@config(default);

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn peek [$f: open_file] -> i32:
    return $f.fd;
end fn peek

fn.main [] -> void:
    let $f: open_file = open_file[fd = 3];
    let $n: i32 = peek(Lend $f);
    return void;
end fn.main

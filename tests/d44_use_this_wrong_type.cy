// trace: D44, D7

fn read_port [none] -> i32 fails [no_port]:
    return 1;
end fn read_port

fn.main [none] -> void:
    let $a: i32 = read_port() ?? "none";
    return void;
end fn.main

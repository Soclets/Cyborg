// trace: D44, D7
@config(default);

fn read_port [] -> i32:
    fails [no_port];

    return 1;
end fn read_port

fn.main [] -> void:
    let $a: i32 = read_port() ?? "none";
    return void;
end fn.main

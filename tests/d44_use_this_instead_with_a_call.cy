// Whether `??` can take a call isn't decided.
// trace: D44

fn read_port [none] -> i32 fails [no_port]:
    return 1;
end fn read_port

fn other [none] -> i32:
    return 2;
end fn other

fn.main [none] -> void:
    let $a: i32 = read_port() ?? other();
    return void;
end fn.main

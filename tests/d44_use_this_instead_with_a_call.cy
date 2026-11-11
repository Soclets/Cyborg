// Whether `??` can take a call isn't decided.
// trace: D44

fn read_port [none] returns i32 fails [no_port]:
    return 1;
end fn read_port

fn other [none] returns i32:
    return 2;
end fn other

fn.main [none] returns void:
    let $a: i32 set_to read_port() ?? other();
    return void;
end fn.main

// This is `(read() ?? read()) ?? 3`, so the outer `??` isn't on one call.
// trace: D132

fn read [none] returns i32 fails [none]:
    return 1;
end fn read

fn.main [none] returns void:
    let $a: i32 set_to read() ?? read() ?? 3;
    return void;
end fn.main

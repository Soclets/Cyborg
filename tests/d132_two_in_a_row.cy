// This is `(read() ?? read()) ?? 3`, so the outer `??` isn't on one call.
// trace: D132

fn read [none] -> i32 fails [none]:
    return 1;
end fn read

fn.main [none] -> void:
    let $a: i32 = read() ?? read() ?? 3;
    return void;
end fn.main

// This is `(read() ?? read()) ?? 3`, so the outer `??` isn't on one call.
// trace: D132
@config(default);

fn read [] -> i32:
    fails [none];

    return 1;
end fn read

fn.main [] -> void:
    let $a: i32 = read() ?? read() ?? 3;
    return void;
end fn.main

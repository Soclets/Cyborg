// What `??` catches when its left side is more than one call isn't decided.
// trace: D132

fn read [none] returns i32 fails [none]:
    return 1;
end fn read

fn.main [none] returns void:
    let $a: i32 = (read() + read()) ?? 0;
    return void;
end fn.main

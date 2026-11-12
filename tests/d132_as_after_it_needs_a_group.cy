// trace: D132, D116

fn read [none] -> i32 fails [none]:
    return 1;
end fn read

fn.main [none] -> void:
    let $s: str[len.max: 20 bytes] = read() ?? 1 as str[len.max: 20 bytes];
    return void;
end fn.main

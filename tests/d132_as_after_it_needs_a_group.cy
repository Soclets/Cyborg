// trace: D132, D116

fn read [none] -> i32 fails [none]:
    return 1;
end fn read

fn.main [none] -> void:
    let $s: str = read() ?? 1 as str;
    return void;
end fn.main

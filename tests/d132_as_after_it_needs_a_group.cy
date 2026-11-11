// trace: D132, D116

fn read [none] returns i32 fails [none]:
    return 1;
end fn read

fn.main [none] returns void:
    let $s: str set_to read() ?? 1 as str;
    return void;
end fn.main

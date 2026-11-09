// trace: D132, D116
@config(default);

fn read [] -> i32:
    fails [none];

    return 1;
end fn read

fn.main [] -> void:
    let $s: str = read() ?? 1 as str;
    return void;
end fn.main

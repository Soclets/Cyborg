// trace: D83
@config(default);

fn read [] -> i32:
    return 1;
end fn read

const $a: i32 = read();

fn.main [] -> void:
    return void;
end fn.main

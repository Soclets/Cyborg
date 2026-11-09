// trace: D171
@config(default);

struct reading:
    celsius: i32;
end struct reading

struct view <`text`>:
    words: Lend from `text` reading;
end struct view

fn shown <`t`> [$v: view<`text`: `u`>] -> i32:
    return 0;
end fn shown

fn.main [] -> void:
    return void;
end fn.main

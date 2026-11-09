// trace: D171
@config(default);

struct reading:
    celsius: i32;
end struct reading

struct view <`text`, `text`>:
    words: Lend from `text` reading;
end struct view

fn.main [] -> void:
    return void;
end fn.main

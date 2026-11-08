// trace: D171
struct reading:
    celsius: i32;
end struct reading

struct view <`text`>:
    words: Lend from `txt` reading;
end struct view

fn.main [] -> void:
    return void;
end fn.main

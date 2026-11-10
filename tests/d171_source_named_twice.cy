// trace: D171

struct reading:
    celsius: i32;
end struct reading

struct view <`text`, `text`>:
    words: Lend from `text` reading;
end struct view

fn.main [none] -> void:
    return void;
end fn.main

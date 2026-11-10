// trace: D171

struct reading:
    celsius: i32;
end struct reading

struct editor <`text`>:
    words: Lend.mut from `text` reading;
end struct editor

fn.main [none] -> void:
    return void;
end fn.main

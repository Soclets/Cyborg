// trace: D171
struct reading:
    celsius: i32;
end struct reading

struct view <`text`, `table`: lasts_at_least `text`, `more`: lasts_at_least `text`>:
    words: Lend from `text` reading;
    lookup: Lend from `table` reading;
    other: Lend from `more` reading;
end struct view

fn build [$a: reading, $b: reading] -> void:
    requires $a lasts_at_least $b;
    requires $a lasts_at_least $b;
    return void;
end fn build

fn.main [] -> void:
    return void;
end fn.main

// trace: D171

struct reading:
    celsius: i32;
end struct reading

struct index <`text`, `table`: lasts_at_least `text`>:
    words: Lend from `text` reading;
    lookup: Lend from `table` reading;
end struct index

fn take [$i: index] returns void:
    return void;
end fn take

fn.main [none] returns void:
    return void;
end fn.main

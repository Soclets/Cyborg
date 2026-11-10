// trace: D171

struct reading:
    celsius: i32;
end struct reading

struct index <`text`, `table`: lasts_at_least `text`>:
    words: Lend from `text` reading;
    lookup: Lend from `table` reading;
end struct index

fn.main [none] -> void:
    let $a: reading = reading[celsius = 1];
    let $i: index<`table`: $a, `text`: $a> = index[words = Lend $a, lookup = Lend $a];
    return void;
end fn.main

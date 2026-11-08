// trace: D171
struct reading:
    celsius: i32;
end struct reading

struct index <`text`, `table`: lasts_at_least `text`>:
    words: Lend from `text` reading;
    lookup: Lend from `table` reading;
end struct index

fn.main [] -> void:
    let $article: reading = reading[celsius = 1];
    let $i: index<`text`: $article> = index[words = Lend $article, lookup = Lend $article];
    return void;
end fn.main

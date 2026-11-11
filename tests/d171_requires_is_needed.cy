// Two parameters, and nothing says which lasts longer.
// trace: D171

struct reading:
    celsius: i32;
end struct reading

struct index <`text`, `table`: lasts_at_least `text`>:
    words: Lend from `text` reading;
    lookup: Lend from `table` reading;
end struct index

fn build [$article: reading, $list: reading] returns index<`text`: $article, `table`: $list>:
    return index[words set_to Lend $article, lookup set_to Lend $list];
end fn build

fn.main [none] returns void:
    return void;
end fn.main

// trace: D171

struct reading:
    celsius: i32;
end struct reading

struct index <`text`, `table`: lasts_at_least `text`>:
    words: Lend from `text` reading;
    lookup: Lend from `table` reading;
end struct index

fn.main [none] returns void:
    let $article: reading set_to reading[celsius set_to 1];
    let $i: index<`text`: $article> set_to index[words set_to Lend $article, lookup set_to Lend $article];
    return void;
end fn.main

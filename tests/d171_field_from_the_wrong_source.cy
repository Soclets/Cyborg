// trace: D171

struct reading:
    celsius: i32;
end struct reading

struct index <`text`, `table`: lasts_at_least `text`>:
    words: Lend from `text` reading;
    lookup: Lend from `table` reading;
end struct index

fn.main [none] returns void:
    let $list: reading set_to reading[celsius set_to 2];
    let $article: reading set_to reading[celsius set_to 1];
    let $i: index<`text`: $article, `table`: $list> set_to index[words set_to Lend $list, lookup set_to Lend $list];
    return void;
end fn.main

// trace: D171
import std;
@config(default);

struct reading:
    celsius: i32;
end struct reading

struct index <`text`, `table`: lasts_at_least `text`>:
    words: Lend from `text` reading;
    lookup: Lend from `table` reading;
end struct index

fn.main [] -> void:
    let $list: reading = reading[celsius = 2];
    let $article: reading = reading[celsius = 1];
    let $i: index<`text`: $article, `table`: $list> = index[words = Lend $article, lookup = Lend $list];
    std::println($i.words.celsius as str, " ", $i.lookup.celsius as str);
    return void;
end fn.main

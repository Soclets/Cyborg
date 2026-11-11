// trace: D171, D170
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

struct index <`text`, `table`: lasts_at_least `text`>:
    words: Lend from `text` reading;
    lookup: Lend from `table` reading;
end struct index

fn build [$article: reading, $list: reading] returns index<`text`: $article, `table`: $list>:
    requires $list lasts_at_least $article;
    return index[words set_to Lend $article, lookup set_to Lend $list];
end fn build

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $list: reading set_to reading[celsius set_to 2];
    let $article: reading set_to reading[celsius set_to 1];
    let $i: index<`text`: $article, `table`: $list> set_to build(Lend $article, Lend $list);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $i.lookup.celsius);
    return void;
end fn.main

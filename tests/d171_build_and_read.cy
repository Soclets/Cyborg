// trace: D171
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

struct index <`text`, `table`: lasts_at_least `text`>:
    words: Lend from `text` reading;
    lookup: Lend from `table` reading;
end struct index

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $list: reading set_to reading[celsius set_to 2];
    let $article: reading set_to reading[celsius set_to 1];
    let $i: index<`text`: $article, `table`: $list> set_to index[words set_to Lend $article, lookup set_to Lend $list];
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $i.words.celsius, $i.lookup.celsius);
    return void;
end fn.main

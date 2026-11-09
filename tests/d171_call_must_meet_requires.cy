// `$list` is in an inner block, so it ends before `$article` does.
// trace: D171
@config(default);

struct reading:
    celsius: i32;
end struct reading

struct index <`text`, `table`: lasts_at_least `text`>:
    words: Lend from `text` reading;
    lookup: Lend from `table` reading;
end struct index

fn build [$article: reading, $list: reading] -> index<`text`: $article, `table`: $list>:
    requires $list lasts_at_least $article;
    return index[words = Lend $article, lookup = Lend $list];
end fn build

fn.main [] -> void:
    let $article: reading = reading[celsius = 1];
    if $article.celsius > 0:
        let $list: reading = reading[celsius = 2];
        let $i: index<`text`: $article, `table`: $list> = build(Lend $article, Lend $list);
    end if
    return void;
end fn.main

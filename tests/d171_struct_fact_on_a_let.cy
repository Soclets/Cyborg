// trace: D171

struct reading:
    celsius: i32;
end struct reading

struct index <`text`, `table`: lasts_at_least `text`>:
    words: Lend from `text` reading;
    lookup: Lend from `table` reading;
end struct index

fn.main [none] -> void:
    let $article: reading = reading[celsius = 1];
    if $article.celsius > 0:
        let $list: reading = reading[celsius = 2];
        let $i: index<`text`: $list, `table`: $article> = index[words = Lend $list, lookup = Lend $article];
    end if
    if $article.celsius > 0:
        let $list: reading = reading[celsius = 2];
        let $j: index<`text`: $article, `table`: $list> = index[words = Lend $article, lookup = Lend $list];
    end if
    return void;
end fn.main

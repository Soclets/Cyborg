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
    if $article.celsius above 0:
        let $list: reading set_to reading[celsius set_to 2];
        let $i: index<`text`: $list, `table`: $article> set_to index[words set_to Lend $list, lookup set_to Lend $article];
    end if
    if $article.celsius above 0:
        let $list: reading set_to reading[celsius set_to 2];
        let $j: index<`text`: $article, `table`: $list> set_to index[words set_to Lend $article, lookup set_to Lend $list];
    end if
    return void;
end fn.main

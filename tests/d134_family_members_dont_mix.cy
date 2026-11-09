// trace: D134, D135
@config(default);

fam.type.new ids:
    ids.user = i64;
    ids.order = i64;
end fam.type.new ids

fn f [$u: ids.user] -> void:
    let $o: ids.order = $u;
    return void;
end fn f

fn.main [] -> void:
    return void;
end fn.main

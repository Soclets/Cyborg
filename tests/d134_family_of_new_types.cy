// trace: D134, D135
fam.type.new ids:
    ids.user = i64;
    ids.order = i64;
end fam.type.new ids

fn f [$u: ids.user] -> ids.user:
    return $u;
end fn f

fn.main [] -> void:
    let $u: ids.user = 1;
    return void;
end fn.main

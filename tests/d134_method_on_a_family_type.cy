// trace: D134, D130
fam.type.new ids:
    ids.user = i64;
end fam.type.new ids

fn ids.user.%same [$u: ids.user] -> ids.user:
    return $u;
end fn ids.user.%same

fn.main [] -> void:
    return void;
end fn.main

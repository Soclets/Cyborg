// trace: D134, D135

fam.type.new ids:
    ids.user set_to i64;
    ids.order set_to i64;
end fam.type.new ids

fn f [$u: ids.user] returns void:
    let $o: ids.order set_to $u;
    return void;
end fn f

fn.main [none] returns void:
    return void;
end fn.main

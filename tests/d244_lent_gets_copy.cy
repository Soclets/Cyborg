// A lent `?item` can't be given away, so returning it bare is refused with `Copy` as the fix.
// trace: D244, D3

fn first<?item: any.type> [$a: ?item, $b: ?item] returns ?item:
    return $a;
end fn first

fn.main [none] returns void:
    return void;
end fn.main

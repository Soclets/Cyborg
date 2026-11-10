// A lent `?item` can't be given away, so returning it bare is refused with `Copy` as the fix.
// trace: D244, D3
@config(default);

fn first<?item: any.type> [$a: ?item, $b: ?item] -> ?item:
    return $a;
end fn first

fn.main [] -> void:
    return void;
end fn.main

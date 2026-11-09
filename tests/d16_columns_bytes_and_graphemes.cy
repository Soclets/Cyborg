// The place says bytes for tools and graphemes for people.
// trace: D16
@config(default);

struct box:
    n: i32;
end struct box

fn read [$b: box] -> i32:
    return $b.n;
end fn read

fn.main [] -> void:
    let $b: box = box[n = 1];
    let $s: str = "é🇹🇭"; let $n: i32 = read($b);
    return void;
end fn.main

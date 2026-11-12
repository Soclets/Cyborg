// A literal going into heap text needs heap memory, which takes an allocator.
// trace: D326, D321
fn.main [none] -> void:
    let $log: str[len.max: inf bytes] = "abc";
    return void;
end fn.main

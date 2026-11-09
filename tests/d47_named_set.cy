// trace: D47
import std;
@config(default);

fam.failure file_failed:
    file_failed.not_found;
    file_failed.denied;
end fam.failure file_failed

fn load [$which: i32] -> i32:
    fails [file_failed, bad_syntax];

    if $which == 1:
        fail file_failed.not_found;
    end if
    if $which == 2:
        fail denied;
    end if
    if $which == 3:
        fail bad_syntax;
    end if
    return 0;
end fn load

fn name_of [$which: i32] -> void:
    let $n: i32 = load($which) otherwise:
        case not_found:
            std::println("not found");
            return void;
        case file_failed.denied:
            std::println("denied");
            return void;
        case bad_syntax:
            std::println("bad syntax");
            return void;
    end otherwise
    std::println("fine ", $n as str);
    return void;
end fn name_of

fn.main [] -> void:
    name_of(1);
    name_of(2);
    name_of(3);
    name_of(4);
    return void;
end fn.main

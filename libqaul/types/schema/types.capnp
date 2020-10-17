struct UserAuth {
    id @0 :Text;
    token @1 :Text;
}

struct ContactQuery {
    union {
        nick @0 :Text;
        trust :group {
            val @1 :Int8;
            fuz @2 :Int8;
        }
        met @3 :Bool;
        location @4 :Text;
        notes @5 :Text;
    }
}

struct Mode {
    union {
        flood @0 :Void;
        std @1 :Text;
    }
}

struct IdType {
    union {
        unique @0 :Void;
        grouped @1 :Text;
    }
}

struct SeviceId {
    union {
        # It's one of the most common passwords after all
        god @0 :Void;
        id @1 :Text;
    }
}

struct Tag {
    key @0 :Text;
    val @1 :Data;
}

struct TagSet {
    tags @0 :List(Tag);
}

struct MsgQuery {
    id @0 :Text;
    sender @1 :Text;
    tags @2 :TagSet;
    skip @3 :UInt;
}

struct MetadataMap {
    name @0 :Text;
    map @1 :List(Entry);

    struct Entry {
        key @0 :Text;
        val @1 :Data;
    }
}

struct SetDiff(Val) {
    union {
        set @0 :Val;
        unset @0 :Val;
    }
}

struct UserUpdate {
    handle @0 :Text;
    disp_name @1 :Text;
    add_to_bio @2 :List(BioLine);
    rm_fr_bio @3 :List(Text);
    add_serv @4 :Text;
    rm_serv @5 :Text;
    avi_data @6 :SetDiff(Data);

    struct BioLine {
        key @0 :Text;
        val @1 :Text;
    }
}

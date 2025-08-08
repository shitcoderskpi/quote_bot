@0xc7280e4ca6f12d46;

struct Image {
    background @0 :Text;
    textEntries @1 :List(TextEntry);

    struct TextEntry {
        content @0 :Text;
        x @1 :Int32;
        y @2 :Int32;
        fontFamily @3 :Text;
        size @4 :Text;
        weight @5 :UInt16;
        alignment @6 :Alignment;
        color @7 :Text;
        wrapWidth @8 :Int32 = 0;

        enum Alignment {
            left @0;
            center @1;
            right @2;
        }
    }
}
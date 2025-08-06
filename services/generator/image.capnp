@0xc7280e4ca6f12d46;

struct Image {
    background @0 :Text;
    textEntries @1 :List(TextEntry);

    struct TextEntry {
        content @0 :Text;
        x @1 :Int32;
        y @2 :Int32;
        size @3 :Text;
        weight @4 :UInt16;
        alignment @5 :Alignment;
        color @6 :Text;
        wrapWidth @7 :Int32 = 0;

        enum Alignment {
            left @0;
            center @1;
            right @2;
        }
    }
}
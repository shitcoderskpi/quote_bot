//
// Created by mmatz on 8/6/25.
//

#ifndef IMAGE_SERIALIZER_H
#define IMAGE_SERIALIZER_H
#include "image.capnp.h"
#include "preprocessor.h"


class image_serializer {
    public:

    static void serialize_image(const templates::image &img, const std::filesystem::path &filename);
    static std::optional<templates::image> deserialize_image(const std::filesystem::path &filename);

    private:
    template<typename AlignmentT>
    static AlignmentT _pango_to_capnp_alignment(const PangoAlignment &alignment);

    static std::vector<pango::text> read_text(const Image::Reader &img_reader);

};

template<typename AlignmentT>
AlignmentT image_serializer::_pango_to_capnp_alignment(const PangoAlignment &alignment) {
    switch (alignment) {
        case PANGO_ALIGN_CENTER: return Image::TextEntry::Alignment::CENTER;
        case PANGO_ALIGN_RIGHT: return Image::TextEntry::Alignment::RIGHT;
        default: return Image::TextEntry::Alignment::LEFT;
    }
}


#endif //IMAGE_SERIALIZER_H

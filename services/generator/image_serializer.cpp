//
// Created by mmatz on 8/6/25.
//

#include "image_serializer.h"

#include <fstream>
#include <capnp/message.h>
#include <kj/std/iostream.h>
#include <capnp/serialize.h>

void image_serializer::serialize_image(const templates::image &img, const std::filesystem::path &filename) {
    ::capnp::MallocMessageBuilder message;
    auto img_builder = message.initRoot<Image>();
    img_builder.setBackground(img.background);

    auto text_builder = img_builder.initTextEntries(img.text_entries.size());
    for (size_t i = 0; i < img.text_entries.size(); ++i) {
        auto builder = text_builder[i];
        const auto&[content, x, y, font_family, size, weight, alignment, color, wrap_width] = img.text_entries[i];

        builder.setContent(content);
        builder.setX(x);
        builder.setY(y);
        builder.setFontFamily(font_family);
        builder.setSize(size);
        builder.setWeight(weight);
        builder.setAlignment(_pango_to_capnp_alignment<capnp::schemas::Alignment_cda29255ea1e0512>(alignment));
        builder.setColor(color);
        builder.setWrapWidth(wrap_width);
    }

    std::ofstream file(filename, std::ios::binary);
    kj::std::StdOutputStream out {file};

    capnp::writeMessage(out, message);
}

std::optional<templates::image> image_serializer::deserialize_image(const std::filesystem::path &filename) {
    if (!std::filesystem::exists(filename) || !std::filesystem::is_regular_file(filename)) {
        return std::nullopt;
    }

    std::ifstream file { filename, std::ios::binary };
    kj::std::StdInputStream in {file};
    ::capnp::InputStreamMessageReader message { in };

    Image::Reader image_reader = message.getRoot<Image>();

    return templates::image {image_reader.getBackground(), read_text(image_reader)};
}

std::vector<pango::text> image_serializer::read_text(const Image::Reader &img_reader) {
    const auto text_reader = img_reader.getTextEntries();
    std::vector<pango::text> result {text_reader.size()};

    size_t i {};
    for (const auto &entry : text_reader) {
        const auto text = pango::text_factory::from_capnp_reader(entry);
        result[i++] = text;
    }

    return result;
}

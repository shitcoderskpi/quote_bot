//
// Created by mmatz on 8/4/25.
//

#ifndef IMAGE_H
#define IMAGE_H
#include <filesystem>
#include <string>
#include <vector>

#include "text.h"

namespace templates {

    struct image {
        const std::string background;
        const std::vector<pango::text> text_entries;
    };

} // templates

#endif //IMAGE_H

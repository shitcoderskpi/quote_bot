//
// Created by mmatz on 8/10/25.
//

#include "response.h"

#include <sstream>

std::string response::to_json() const {
    std::stringstream ss;
    ss << "{" << R"("chat_id": )" << chat_id << ", " << R"("image": ")" << base64_img << R"("})";
    return ss.str();
}

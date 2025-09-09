//
// Created by mmatz on 8/10/25.
//

#ifndef RESPOSE_H
#define RESPOSE_H
#include <string_view>


struct response {
    long long chat_id;
    std::string_view base64_img;

    [[nodiscard]] std::string to_json() const;
};



#endif //RESPOSE_H

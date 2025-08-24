//
// Created by mmatz on 7/16/25.
//

#ifndef REDIS_REPLY_H
#define REDIS_REPLY_H
#include <cstddef>
#include <hiredis/hiredis.h>
#include <iterator>
#include <optional>
#include <stdexcept>

class redis_array;

class empty_reply_error final : public std::runtime_error {
public:
  empty_reply_error() noexcept : std::runtime_error("Redis reply is empty.") {}

  explicit empty_reply_error(const std::string &_arg)
      : std::runtime_error(_arg) {}

  explicit empty_reply_error(const char *string) : std::runtime_error(string) {}

  explicit empty_reply_error(std::runtime_error &&runtime_error)
      : std::runtime_error(runtime_error) {}

  explicit empty_reply_error(const std::runtime_error &runtime_error)
      : std::runtime_error(runtime_error) {}
};

class redis_reply {
public:
  explicit redis_reply(redisReply *reply, bool can_free = true) noexcept;
  explicit redis_reply(void *reply, bool can_free = true);
  ~redis_reply();

  redis_reply(const redis_reply &) = default;
  redis_reply &operator=(const redis_reply &) = delete;

  redis_reply(redis_reply &&other) noexcept
      : _reply(other._reply), can_free(true) {
    other._reply = nullptr;
  }

  redis_reply &operator=(redis_reply &&other) noexcept {
    if (this != &other) {
      if (_reply) {
        freeReplyObject(_reply);
      }
      _reply = other._reply;
      other._reply = nullptr;
    }
    return *this;
  }

  [[deprecated(
      "Use get_<redisReply's field> functions instead ( elements and element "
      "being get_array() )")]] [[nodiscard]] std::optional<redisReply *>
  get_reply() const noexcept;

  [[nodiscard]] int get_type() const;
  [[nodiscard]] long long get_integer() const;
  [[nodiscard]] double get_dval() const;
  [[nodiscard]] size_t get_len() const;
  [[nodiscard]] std::string get_str() const;
  [[nodiscard]] std::array<char, 4> get_vtype() const;
  [[nodiscard]] redis_array get_array() const;
  [[nodiscard]] bool has_value() const noexcept;

private:
  redisReply *_reply;
  const bool can_free;
};

class redis_array {
public:
  struct iterator {
    using iterator_category = std::input_iterator_tag;
    using difference_type = std::ptrdiff_t;
    using value_type = redis_reply;
    using pointer = redis_reply *;
    using reference = const redis_reply &;

    explicit iterator(redisReply **ptr) noexcept : _ptr(ptr) {}

    value_type operator*() const { return redis_reply{*_ptr}; }

    iterator &operator++() {
      ++_ptr;

      return *this;
    }

    iterator &operator++(int) {
      iterator &tmp = *this;
      ++_ptr;

      return tmp;
    }

    friend bool operator==(const iterator &a, const iterator &b) {
      return a._ptr == b._ptr;
    }
    friend bool operator!=(const iterator &a, const iterator &b) {
      return a._ptr != b._ptr;
    }

  private:
    redisReply **_ptr;
  };

  redis_array(redisReply **ptr, const size_t len) noexcept
      : _ptr(ptr), len(len) {}

  [[nodiscard]] iterator begin() const noexcept { return iterator{_ptr}; }
  [[nodiscard]] iterator end() const noexcept { return iterator{_ptr + len}; }

  [[nodiscard]] size_t size() const noexcept { return len; }

  redis_reply operator[](const size_t idx) const {
    if (idx >= len) {
      throw std::out_of_range("index out of range");
    }
    return redis_reply{_ptr[idx], false};
  }

private:
  redisReply **_ptr;
  const size_t len;
};

#endif // REDIS_REPLY_H

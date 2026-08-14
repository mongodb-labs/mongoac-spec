#pragma once

#include <mongoac/string.h>

#include <cstring>
#include <string>
#include <string_view>
#include <utility>

namespace mongoac
{
namespace test_util
{

inline mongoac_string_view_t
to_mongoac(char const *v)
{
   if (v == nullptr) {
      return {};
   }

   return {v, std::strlen(v)};
}

inline mongoac_string_view_t
to_mongoac(std::string_view v)
{
   return {v.data(), v.size()};
}

inline std::string_view
from_mongoac(mongoac_string_view_t v)
{
   if (v.data == nullptr) {
      return {};
   }

   return std::string_view{v.data, v.len};
}

inline std::string
to_string(mongoac_string_view_t v)
{
   if (v.data == nullptr) {
      return {};
   }

   return std::string(v.data, v.len);
}

class owning_string
{
 private:
   mongoac_string_t _str;

 public:
   ~owning_string()
   {
      mongoac_string_destroy(_str);
   }

   owning_string(owning_string &&other) noexcept : _str{other._str}
   {
      other._str = mongoac_string_t{nullptr, 0};
   }

   owning_string(owning_string const &other) = delete;
   owning_string &
   operator=(owning_string const &other) = delete;

   owning_string &
   operator=(owning_string &&other) noexcept
   {
      auto tmp = std::move(other);
      tmp.swap(*this);
      return *this;
   }

   explicit owning_string(mongoac_string_t str) : _str{str}
   {
   }

   explicit
   operator bool() const
   {
      return _str.data != nullptr;
   }

   std::string_view
   view() const
   {
      if (_str.data == nullptr) {
         return {};
      }
      return {_str.data, _str.len};
   }

   std::string
   value() const
   {
      if (_str.data == nullptr) {
         return {};
      }

      return {_str.data, _str.len};
   }

   /* explicit(false) */
   operator std::string() const
   {
      return this->value();
   }

   void
   swap(owning_string &other) noexcept
   {
      std::swap(_str, other._str);
   }
};

} // namespace test_util
} // namespace mongoac

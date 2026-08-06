#pragma once

#include <mongoac/string.h>

#include <string>
#include <string_view>
#include <utility>

namespace mongoac
{
namespace test_util
{

inline std::string_view
to_string_view(mongoac_string_view_t v)
{
   if (v.data == nullptr) {
      return {};
   }
   return std::string_view{v.data, v.len};
}

inline std::string
to_string(mongoac_string_view_t s)
{
   std::string result;
   if (s.data != nullptr) {
      result.assign(s.data, s.len);
   }
   return result;
}

inline std::string
to_string(mongoac_string_t s)
{
   std::string result;
   if (s.data != nullptr) {
      result.assign(s.data, s.len);
   }
   mongoac_string_destroy(s);
   return result;
}

class owning_string
{
 private:
   mongoac_string_t _owner;

 public:
   ~owning_string()
   {
      mongoac_string_destroy(_owner);
   }

   owning_string(owning_string &&other) noexcept : _owner{other._owner}
   {
      other._owner = mongoac_string_t{nullptr, 0};
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

   explicit owning_string(mongoac_string_t s) : _owner{s}
   {
   }

   explicit
   operator bool() const
   {
      return _owner.data != nullptr;
   }

   std::string_view
   view() const
   {
      if (_owner.data == nullptr) {
         return {};
      }
      return {_owner.data, _owner.len};
   }

   void
   swap(owning_string &other) noexcept
   {
      std::swap(_owner, other._owner);
   }
};

} // namespace test_util
} // namespace mongoac
